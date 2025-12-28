use std::{collections::HashMap, path::PathBuf, sync::Arc};

use eyre::{Result, WrapErr};
use kenchiku_common::Context;
use kenchiku_scaffold::{
    Scaffold,
    discovery::{discover_scaffold, find_all_scaffolds},
};
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ProtocolVersion, ServerCapabilities, ServerInfo, ToolsCapability},
    schemars, tool, tool_handler, tool_router,
};
use tokio::{
    io::{stdin, stdout},
    sync::Mutex,
};
use tracing::{info, warn};

use crate::session::{MissingValueError, Session, Status};

#[derive(Clone)]
pub struct KenchikuMcpServer {
    tool_router: ToolRouter<Self>,
    session: Arc<tokio::sync::Mutex<Option<Session>>>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct ConstructArgs {
    /// Which scaffold to construct.
    scaffold_name: String,
    /// Values to pass to the scaffold. Always use the `show` tool to see which values are required
    /// and what types they are etc.
    /// A simple dictionary of keys being the value names and values being their values.
    values: Option<HashMap<String, serde_json::Value>>,
    /// Output/target path to construct to. Optional, defaults to working directory.
    output: Option<String>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct PatchArgs {
    /// Which patch to run. Use '<scaffold>:<patch>' to select the correct patch.
    name: String,
    /// Values to pass to the patch. Always use the `show` tool to see which values are required
    /// and what types they are etc.
    /// A simple dictionary of keys being the value names and values being their values.
    values: Option<HashMap<String, serde_json::Value>>,
    /// Output/target path to run patch in. Optional, defaults to working directory.
    output: Option<String>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct ShowArgs {
    /// Name of the thing to show. Use a normal string to show a scaffold,
    /// and '<scaffold>:<patch>' to show a patch from a scaffold.
    name: String,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct ProvideValuesArgs {
    /// Values to provide to the current session.
    values: HashMap<String, serde_json::Value>,
}

#[tool_router(router = tool_router)]
impl KenchikuMcpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            session: Arc::new(Mutex::new(None)),
        }
    }

    async fn start_session<F>(
        &self,
        scaffold_name: String,
        values: Option<HashMap<String, serde_json::Value>>,
        output: Option<String>,
        setup_operation: F,
    ) -> String
    where
        F: FnOnce(
                Scaffold,
            ) -> eyre::Result<(
                HashMap<String, kenchiku_common::meta::ValueMeta>,
                Box<dyn FnOnce(Context) -> eyre::Result<String> + Send>,
            )> + Send
            + 'static,
    {
        let session = self.session.clone();

        // Make sure only one session runs at a time
        let mut session_guard = session.lock().await;
        if session_guard.is_some() {
            return "A session is already active. Please complete or cancel it first.".to_string();
        }

        let output_path = if let Some(o) = output {
            PathBuf::from(o)
        } else {
            match std::env::current_dir() {
                Ok(p) => p,
                Err(e) => return format!("Failed to get current directory: {}", e),
            }
        };

        let scaffold_name_clone = scaffold_name.clone();
        let scaffold_result = tokio::task::spawn_blocking(move || {
            discover_scaffold(scaffold_name_clone).map(Scaffold::load)
        })
        .await
        .unwrap_or_else(|e| Some(Err(eyre::eyre!(e))));

        if let Some(Ok(scaffold)) = scaffold_result {
            let provided_values: HashMap<String, serde_json::Value> =
                kenchiku_common::get_env_values()
                    .into_iter()
                    .map(|(k, v)| (k, serde_json::Value::String(v)))
                    .chain(values.unwrap_or_default().into_iter())
                    .collect();

            let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<HashMap<String, serde_json::Value>>();
            let (status_tx, mut status_rx) =
                tokio::sync::mpsc::channel::<crate::session::Status>(1);

            let output_path_clone = output_path.clone();
            let provided_values_clone = provided_values.clone();

            let mut join_handle = tokio::task::spawn_blocking(move || -> eyre::Result<String> {
                let scaffold_path = scaffold.path.clone();
                let (values_meta, operation) = setup_operation(scaffold)?;

                let cmd_rx = std::sync::Mutex::new(cmd_rx);
                let current_values = Arc::new(std::sync::Mutex::new(provided_values_clone));

                let current_values_clone = current_values.clone();

                let prompt_value = Arc::new(
                    move |name: String,
                          r#type: String,
                          description: String,
                          choices: Option<Vec<String>>,
                          _default: Option<String>,
                          validator: Option<
                        Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>,
                    >|
                          -> eyre::Result<String> {
                        loop {
                            let error_msg = &mut None;
                            {
                                let mut values = current_values_clone.lock().unwrap();
                                if let Some(val) = values.get(&name) {
                                    let val_str = val.to_string().trim_matches('"').to_string();
                                    if let Some(validator) = &validator {
                                        if let Err(e) = validator(&val_str) {
                                            *error_msg = Some(e.to_string());
                                            values.remove(&name);
                                        }
                                    }
                                    return Ok(val_str);
                                }
                            }

                            // Request value from model
                            let _ =
                                status_tx.blocking_send(Status::MissingValue(MissingValueError {
                                    name: name.clone(),
                                    r#type: r#type.clone(),
                                    description: description.clone(),
                                    choices: choices.clone(),
                                    error: error_msg.clone(),
                                }));

                            // Wait for new values from the model
                            let rx = cmd_rx.lock().unwrap();
                            if let Ok(new_values) = rx.recv() {
                                let mut values = current_values_clone.lock().unwrap();
                                values.extend(new_values);
                            } else {
                                return Err(eyre::eyre!("Session cancelled"));
                            }
                        }
                    },
                );

                let mut temp_dir = tempfile::tempdir()?;

                let context = Context {
                    working_dir: temp_dir.path().to_path_buf(),
                    scaffold_dir: scaffold_path,
                    output: output_path_clone,
                    values: current_values
                        .lock()
                        .unwrap()
                        .iter()
                        .map(|(k, v)| (k.clone(), v.to_string().trim_matches('"').to_string()))
                        .collect(),
                    values_meta,
                    prompt_value,
                    ..Default::default()
                };

                let msg = operation(context)?;
                // only disable cleanup if we constructed successfully
                temp_dir.disable_cleanup(true);
                Ok(msg)
            });
            tokio::select! {
                Some(crate::session::Status::MissingValue(missing)) = status_rx.recv() => {
                    *session_guard = Some(Session {
                        values: provided_values,
                        missing_values: vec![missing.name.clone()],
                        value_sender: cmd_tx,
                        status_receiver: Some(status_rx),
                        join_handle: Some(join_handle),
                    });
                    format!(
                        "Missing value: {}. Description: {}. Type: {}. Please use `provide_values` to supply it.",
                        missing.name, missing.description, missing.r#type
                    )
                }
                result = &mut join_handle => {
                    match result {
                        Ok(Ok(msg)) => msg,
                        // Makes it easier to distinguish lol
                        Ok(Err(e)) => format!("Operation errored: {:?}", e),
                        Err(e) => format!("Operation failed: {:?}", e),
                    }
                }
            }
        } else {
            format!("Scaffold '{}' not found.", scaffold_name)
        }
    }

    #[tool(description = "
        Scaffold a new project. Specify values using the `values` parameter.
        Use the `show` tool to find out what values the scaffold wants.
        If you are unsure about some values, ask the user.
    ")]
    pub async fn construct(
        &self,
        Parameters(ConstructArgs {
            scaffold_name,
            values,
            output,
        }): Parameters<ConstructArgs>,
    ) -> String {
        let scaffold_name_clone = scaffold_name.clone();
        self.start_session(scaffold_name, values, output, move |scaffold| {
            let meta = scaffold.meta.values.clone();
            let op = Box::new(move |ctx| {
                scaffold.construct(ctx)?;
                Ok(format!(
                    "Scaffold '{}' constructed successfully.",
                    scaffold_name_clone
                ))
            });
            Ok((meta, op))
        })
        .await
    }

    #[tool(description = "
        Patch an existing project. Specify values using the `values` parameter.
        Use the `show` tool to find out what values the patch wants.
        If you are unsure about some values, ask the user.
    ")]
    pub async fn patch(
        &self,
        Parameters(PatchArgs {
            name,
            values,
            output,
        }): Parameters<PatchArgs>,
    ) -> String {
        let (scaffold_name, patch_name) = match name.split_once(':') {
            Some((s, p)) => (s.to_string(), p.to_string()),
            None => return "Invalid patch name format. Use '<scaffold>:<patch>'.".to_string(),
        };

        let scaffold_name_clone = scaffold_name.clone();
        let patch_name_clone = patch_name.clone();

        self.start_session(scaffold_name, values, output, move |scaffold| {
            let patch_meta = match scaffold.meta.patches.get(&patch_name_clone) {
                Some(meta) => meta,
                None => {
                    return Err(eyre::eyre!(
                        "Patch '{}' not found in scaffold '{}'.",
                        patch_name_clone,
                        scaffold_name_clone
                    ));
                }
            };
            let meta = patch_meta.values.clone();
            let op = Box::new(move |ctx| {
                scaffold.call_patch(&patch_name_clone, ctx)?;
                Ok(format!(
                    "Patch '{}:{}' executed successfully.",
                    scaffold_name_clone, patch_name_clone
                ))
            });
            Ok((meta, op))
        })
        .await
    }

    #[tool(description = "
        Provide extra values for the current session. The `values` param is required!
        Values are saved, so no need to repeat them across tool calls, you can just add one
        value after another or multiple at once if you know all missing ones.
    ")]
    pub async fn provide_values(
        &self,
        Parameters(ProvideValuesArgs { values }): Parameters<ProvideValuesArgs>,
    ) -> String {
        let session = self.session.clone();

        // Lock session and send values
        let (status_rx, join_handle) = {
            let mut session_guard = session.lock().await;
            if let Some(session) = session_guard.as_mut() {
                session.values.extend(values.clone());

                // Send values to execution thread
                if let Err(_) = session.value_sender.send(values) {
                    return "Execution thread died, that's unfortunate.".to_string();
                }

                (session.status_receiver.take(), session.join_handle.take())
            } else {
                return "No active session.".to_string();
            }
        };

        if let (Some(mut rx), Some(mut handle)) = (status_rx, join_handle) {
            tokio::select! {
                // A value is missing, provide it
                Some(crate::session::Status::MissingValue(missing)) = rx.recv() => {
                    let mut session_guard = session.lock().await;

                    // Check if session still exists (might have been cancelled)
                    if let Some(session) = session_guard.as_mut() {
                        session.missing_values = vec![missing.name.clone()];
                        session.status_receiver = Some(rx);
                        session.join_handle = Some(handle);
                        format!("Missing value: {}. Description: {}. Type: {}. Choices (if enum): {:?}
                                 Please use `provide_values` to supply it.",
                            missing.name, missing.description, missing.r#type, missing.choices,
                        )
                    } else {
                        "Session was cancelled while processing.".to_string()
                    }
                }
                result = &mut handle => {
                    // Finished, maybe successful, maybe with error
                    let mut session_guard = session.lock().await;
                    *session_guard = None;

                    match result {
                        Ok(Ok(msg)) => msg,
                        // Makes it easier to distinguish lol
                        Ok(Err(e)) => format!("Construction errored: {:?}", e),
                        Err(e) => format!("Construction failed: {:?}", e),
                    }
                }
            }
        } else {
            "Session in invalid state (no status receiver or join handle).".to_string()
        }
    }

    #[tool(description = "
        List all available scaffolds and patches. This only gives an overview,
        use the `show` tool to get details & values.
    ")]
    pub async fn list(&self) -> String {
        tokio::task::spawn_blocking(|| -> eyre::Result<String> {
            let found_scaffolds = find_all_scaffolds()
                .iter()
                .map(|path| Scaffold::load(path.to_path_buf()))
                .collect::<eyre::Result<Vec<Scaffold>>>()?;
            let mut output = Vec::new();
            {
                let mut writer = std::io::Cursor::new(&mut output);
                use std::io::Write;
                writeln!(writer, "Found scaffolds:")?;
                for scaffold in found_scaffolds {
                    scaffold.print(&mut writer, false)?;
                    writeln!(writer, "======")?;
                }
            }
            Ok(String::from_utf8(output)?)
        })
        .await
        .unwrap_or_else(|e| Err(e.into()))
        .unwrap_or_else(|e| e.to_string())
    }

    #[tool(description = "
        Show details & values of a scaffold or patch.
        To get details of a patch, use '<scaffold>:<patch>' for the name.
        Always use this tool first, before constructing or patching, to see which values are required.
    ")]
    async fn show(&self, Parameters(ShowArgs { name }): Parameters<ShowArgs>) -> String {
        tokio::task::spawn_blocking(move || -> eyre::Result<String> {
            let asking_for_patch = name.contains(":");
            let scaffold_name = if asking_for_patch {
                name.split_once(":").unwrap().0.to_string()
            } else {
                name.clone()
            };
            let scaffold = discover_scaffold(scaffold_name).map(Scaffold::load);
            if let Some(Ok(scaffold)) = scaffold {
                let mut output = Vec::new();
                let mut writer = std::io::Cursor::new(&mut output);
                if asking_for_patch {
                    scaffold.print_patch(
                        name.split_once(":").unwrap().1,
                        &mut writer,
                        true,
                        false,
                    )?;
                } else {
                    scaffold.print(&mut writer, true)?;
                }
                Ok(String::from_utf8(output)?)
            } else {
                Ok("No such scaffold or patch found".to_string())
            }
        })
        .await
        .unwrap_or_else(|e| Err(e.into()))
        .unwrap_or_else(|e| e.to_string())
    }

    #[tool(description = "Cancel the current session.")]
    pub async fn cancel_session(&self) -> String {
        let session = self.session.clone();
        let mut session_guard = session.lock().await;
        if let Some(mut session) = session_guard.take() {
            let join_handle = session.join_handle.take();

            drop(session);
            drop(session_guard);

            // Wait for the thread to finish
            if let Some(handle) = join_handle {
                match handle.await {
                    Ok(Ok(_)) => info!("Session cancelled. Background task finished successfully"),
                    Ok(Err(err)) => {
                        // since cancelling means throwing an error, the lua execution will error
                        // too
                        info!(?err, "Session cancelled. Background task failed (expected)")
                    }
                    Err(err) => warn!(?err, "Session cancelled. Background task panicked"),
                }
            }
            "Session cancelled.".to_string()
        } else {
            "No active session.".to_string()
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for KenchikuMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
                ..Default::default()
            },
            instructions: Some("Kenchiku MCP server to scaffold projects. Supports constructing new projects and patching existing projects.".into()),
            server_info: Implementation {
                name: "kenchiku".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                ..Default::default()
            },
        }
    }
}

pub async fn run() -> Result<()> {
    info!("Starting MCP server");
    let server = KenchikuMcpServer::new();

    server
        .serve((stdin(), stdout()))
        .await
        .wrap_err("Failed to serve MCP server")?
        .waiting()
        .await
        .wrap_err("Error while waiting for server shutdown")?;
    Ok(())
}

pub fn run_blocking() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { run().await })?;
    Ok(())
}
