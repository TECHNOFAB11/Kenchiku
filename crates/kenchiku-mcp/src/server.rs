use std::collections::HashMap;

use eyre::{Result, WrapErr};
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
use tokio::io::{stdin, stdout};
use tracing::info;

#[derive(Clone)]
pub struct KenchikuMcpServer {
    tool_router: ToolRouter<Self>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct ConstructArgs {
    /// Which scaffold to construct.
    scaffold_name: String,
    /// Values to pass to the scaffold. Always use the `show` tool to see which values are required
    /// and what types they are etc.
    /// A simple dictionary of keys being the value names and values being their values.
    values: Option<HashMap<String, serde_json::Value>>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct PatchArgs {
    /// Which patch to run. Use '<scaffold>:<patch>' to select the correct patch.
    name: String,
    /// Values to pass to the patch. Always use the `show` tool to see which values are required
    /// and what types they are etc.
    /// A simple dictionary of keys being the value names and values being their values.
    values: Option<HashMap<String, serde_json::Value>>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct ShowArgs {
    /// Name of the thing to show. Use a normal string to show a scaffold,
    /// and '<scaffold>:<patch>' to show a patch from a scaffold.
    name: String,
}

#[tool_router(router = tool_router)]
impl KenchikuMcpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = r#"
        Scaffold a new project. Specify values using the `values` parameter (required).
        Use the `show` tool to find out what values the scaffold wants.
        If the user wanted you to run this, ask them for any values you can't provide yourself.
    "#)]
    pub async fn construct(
        &self,
        Parameters(ConstructArgs {
            scaffold_name,
            values,
        }): Parameters<ConstructArgs>,
    ) -> String {
        tokio::task::spawn_blocking(move || {
            format!(
                "Construct tool called with scaffold_name: {} and values: {:?}",
                scaffold_name, values
            )
        })
        .await
        .unwrap_or_else(|e| e.to_string())
    }

    #[tool(description = r#"
        Patch an existing project. Specify values using the `values` parameter (required).
        Use the `show` tool to find out what values the patch wants.
        If the user wanted you to run this, ask them for any values you can't provide yourself.
    "#)]
    pub async fn patch(
        &self,
        Parameters(PatchArgs { name, values }): Parameters<PatchArgs>,
    ) -> String {
        tokio::task::spawn_blocking(move || {
            format!(
                "Patch tool called with name: {} and values: {:?}",
                name, values
            )
        })
        .await
        .unwrap_or_else(|e| e.to_string())
    }

    #[tool(description = r#"
        List all available scaffolds and patches. This only gives an overview,
        use the `show` tool to get details & values.
    "#)]
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

    #[tool(description = r#"
        Show details & values of a scaffold or patch.
        To get details of a patch, use '<scaffold>:<patch>' for the name.
        Always use this tool first, before constructing or patching, to see which values are required.
    "#)]
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
