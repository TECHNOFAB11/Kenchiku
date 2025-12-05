use crate::server::KenchikuMcpServer;
use rmcp::{
    RoleClient, ServiceExt,
    model::{
        ClientInfo, ClientResult, ErrorCode, ErrorData, Implementation, ListToolsResult,
        ServerNotification, ServerRequest,
    },
    service::{NotificationContext, RequestContext, RunningService, Service},
};
use tokio::io::duplex;

struct TestClient {
    info: ClientInfo,
}

impl Service<RoleClient> for TestClient {
    fn get_info(&self) -> ClientInfo {
        self.info.clone()
    }

    async fn handle_request(
        &self,
        _request: ServerRequest,
        _context: RequestContext<RoleClient>,
    ) -> Result<ClientResult, ErrorData> {
        Err(ErrorData {
            code: ErrorCode::METHOD_NOT_FOUND,
            message: "Method not found".into(),
            data: None,
        })
    }

    async fn handle_notification(
        &self,
        _notification: ServerNotification,
        _context: NotificationContext<RoleClient>,
    ) -> Result<(), ErrorData> {
        Ok(())
    }
}

async fn setup_client() -> RunningService<RoleClient, TestClient> {
    let (client_stream, server_stream) = duplex(1024);
    let (client_read, client_write) = tokio::io::split(client_stream);
    let (server_read, server_write) = tokio::io::split(server_stream);

    let server = KenchikuMcpServer::new();

    tokio::spawn(async move {
        server
            .serve((server_read, server_write))
            .await
            .unwrap()
            .waiting()
            .await
            .unwrap();
    });

    let client_service = TestClient {
        info: ClientInfo {
            client_info: Implementation {
                name: "test-client".into(),
                version: "1.0.0".into(),
                ..Default::default()
            },
            ..Default::default()
        },
    };

    rmcp::serve_client(client_service, (client_read, client_write))
        .await
        .unwrap()
}

#[tokio::test]
async fn test_mcp_server_initialize() {
    let client = setup_client().await;
    let init_result = client.peer_info().expect("Peer info should be set");

    assert_eq!(init_result.server_info.name, "kenchiku");
    assert!(init_result.capabilities.tools.is_some());

    client
        .notify_initialized()
        .await
        .expect("Failed to notify initialized");
}

#[tokio::test]
async fn test_mcp_server_list_tools() {
    let client = setup_client().await;
    client
        .notify_initialized()
        .await
        .expect("Failed to notify initialized");

    let tools: ListToolsResult = client.list_tools(None).await.expect("Failed to list tools");
    assert!(tools.tools.len() >= 4);

    let tool_names: Vec<_> = tools.tools.iter().map(|t| t.name.as_ref()).collect();
    assert!(tool_names.contains(&"construct"));
    assert!(tool_names.contains(&"patch"));
    assert!(tool_names.contains(&"list"));
    assert!(tool_names.contains(&"read"));
}

#[tokio::test]
async fn test_mcp_server_read_tool() {
    use rmcp::model::{CallToolRequestParam, CallToolResult};
    use std::env;
    use std::path::Path;
    use tempfile::tempdir;
    use tokio::fs;

    // Setup scaffold
    let temp_dir = tempdir().unwrap();
    let temp_dir_path = temp_dir.path().to_string_lossy().to_string();
    let scaffold_name = "test-scaffold";
    let scaffold_dir = Path::new(&temp_dir_path).join(scaffold_name);
    fs::create_dir_all(&scaffold_dir).await.unwrap();
    let scaffold_content = "print('Hello World')";
    fs::write(scaffold_dir.join("scaffold.lua"), scaffold_content)
        .await
        .unwrap();

    env::set_var("KENCHIKU_PATH", temp_dir_path.clone());

    let client = setup_client().await;

    client
        .notify_initialized()
        .await
        .expect("Failed to notify initialized");

    // Test valid read
    let result: CallToolResult = client
        .call_tool(CallToolRequestParam {
            name: "read".into(),
            arguments: Some(
                serde_json::json!({ "scaffold_name": scaffold_name })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        })
        .await
        .expect("Failed to call read tool");

    assert!(!result.is_error.unwrap_or(false));
    assert_eq!(result.content.len(), 1);
    assert_eq!(result.content[0].as_text().unwrap().text, scaffold_content);

    // Test invalid read
    let result: CallToolResult = client
        .call_tool(CallToolRequestParam {
            name: "read".into(),
            arguments: Some(
                serde_json::json!({ "scaffold_name": "non-existent" })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        })
        .await
        .expect("Failed to call read tool");

    assert!(!result.is_error.unwrap_or(false));
    assert_eq!(
        result.content[0].as_text().unwrap().text,
        "No scaffold with name 'non-existent' found"
    );
}

#[tokio::test]
async fn test_mcp_server_call_list_tool() {
    use rmcp::model::{CallToolRequestParam, CallToolResult};
    use std::env;
    use std::path::Path;
    use tempfile::tempdir;
    use tokio::fs;

    // Setup scaffold
    let temp_dir = tempdir().unwrap();
    let temp_dir_path = temp_dir.path().to_string_lossy().to_string();
    let scaffold_name = "list-test-scaffold";
    let scaffold_dir = Path::new(&temp_dir_path).join(scaffold_name);
    fs::create_dir_all(&scaffold_dir).await.unwrap();
    let scaffold_content = r#"
        return {
            description = "A test scaffold for listing",
            construct = function() end,
        }
    "#;
    fs::write(scaffold_dir.join("scaffold.lua"), scaffold_content)
        .await
        .unwrap();

    env::set_var("KENCHIKU_PATH", temp_dir_path.clone());

    let client = setup_client().await;

    client
        .notify_initialized()
        .await
        .expect("Failed to notify initialized");

    let result: CallToolResult = client
        .call_tool(CallToolRequestParam {
            name: "list".into(),
            arguments: None,
        })
        .await
        .expect("Failed to call list tool");

    assert!(!result.is_error.unwrap_or(false));
    let output = &result.content[0].as_text().unwrap().text;
    assert!(output.contains("Found scaffolds:"));
    assert!(output.contains(&format!("Name: {}", scaffold_name)));
    assert!(output.contains("Description: A test scaffold for listing"));
}
