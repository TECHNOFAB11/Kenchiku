use eyre::{Result, WrapErr};
use kenchiku_scaffold::{Scaffold, discovery::find_all_scaffolds};
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
    scaffold_name: String,
    allow_level: i32,
}

#[tool_router(router = tool_router)]
impl KenchikuMcpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Scaffold a new project")]
    pub async fn construct(
        &self,
        Parameters(ConstructArgs {
            scaffold_name,
            allow_level,
        }): Parameters<ConstructArgs>,
    ) -> String {
        tokio::task::spawn_blocking(move || {
            format!(
                "Construct tool called with scaffold_name: {}, allow_level: {}",
                scaffold_name, allow_level
            )
        })
        .await
        .unwrap_or_else(|e| e.to_string())
    }

    #[tool(description = "Patch an existing project")]
    pub async fn patch(&self) -> String {
        tokio::task::spawn_blocking(|| "Patch tool called".to_string())
            .await
            .unwrap_or_else(|e| e.to_string())
    }

    #[tool(description = "List all available scaffolds and patches")]
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
                    scaffold.print(Some(&mut writer))?;
                    writeln!(writer, "======")?;
                }
            }
            Ok(String::from_utf8(output)?)
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
