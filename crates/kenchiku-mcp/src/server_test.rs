use crate::server::KenchikuMcpServer;
use rmcp::{
    RoleClient, ServiceExt,
    model::{
        ClientInfo, ClientResult, ErrorCode, ErrorData, Implementation, ListToolsResult,
        ServerNotification, ServerRequest,
    },
    service::{NotificationContext, RequestContext, RunningService, Service},
};
use std::sync::Mutex;
use tokio::io::duplex;

static SEQUENTIAL_MUTEX: Mutex<()> = Mutex::new(());

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
    assert!(tool_names.contains(&"show"));
}

#[tokio::test]
async fn test_mcp_server_call_list_tool() {
    let _lock = SEQUENTIAL_MUTEX.lock().unwrap();
    use rmcp::model::{CallToolRequestParam, CallToolResult};
    use std::env;
    use std::path::Path;
    use tempfile::tempdir;
    use tokio::fs;

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

#[tokio::test]
async fn test_mcp_server_session_flow() {
    let _lock = SEQUENTIAL_MUTEX.lock().unwrap();
    use rmcp::model::{CallToolRequestParam, CallToolResult};
    use serde_json::json;
    use std::{collections::HashMap, env, path::Path};
    use tempfile::tempdir;
    use tokio::fs;

    let temp_dir = tempdir().unwrap();
    let temp_dir_path = temp_dir.path().to_string_lossy().to_string();
    let scaffold_name = "session-test-scaffold";
    let scaffold_dir = Path::new(&temp_dir_path).join(scaffold_name);
    fs::create_dir_all(&scaffold_dir).await.unwrap();
    let scaffold_content = r#"
        return {
            description = "A test scaffold for session",
            values = {
                name = {
                    type = "string",
                    description = "Name of the project",
                },
            },
            construct = function()
                local name = values.get("name")
            end,
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

    // 1. Call construct without values
    let mut args = HashMap::new();
    args.insert("scaffold_name".to_string(), json!(scaffold_name));

    let result: CallToolResult = client
        .call_tool(CallToolRequestParam {
            name: "construct".into(),
            arguments: Some(serde_json::Map::from_iter(args.into_iter())),
        })
        .await
        .expect("Failed to call construct tool");

    let output = &result.content[0].as_text().unwrap().text;
    assert!(output.contains("Missing value: name"));
    assert!(output.contains("Description:"));

    // 2. Try to start another session (should fail)
    let mut args = HashMap::new();
    args.insert("scaffold_name".to_string(), json!(scaffold_name));

    let result: CallToolResult = client
        .call_tool(CallToolRequestParam {
            name: "construct".into(),
            arguments: Some(serde_json::Map::from_iter(args.into_iter())),
        })
        .await
        .expect("Failed to call construct tool");

    let output = &result.content[0].as_text().unwrap().text;
    assert!(output.contains("A session is already active"));

    // 3. Provide values
    let mut values = HashMap::new();
    values.insert("name".to_string(), json!("my-project"));
    let mut args = HashMap::new();
    args.insert("values".to_string(), json!(values));

    let result: CallToolResult = client
        .call_tool(CallToolRequestParam {
            name: "provide_values".into(),
            arguments: Some(serde_json::Map::from_iter(args.into_iter())),
        })
        .await
        .expect("Failed to call provide_values tool");

    let output = &result.content[0].as_text().unwrap().text;
    println!("Output: {}", output);
    assert!(output.contains("constructed successfully"));

    // 4. Verify session is cleared (can start new one)
    let mut args = HashMap::new();
    args.insert("scaffold_name".to_string(), json!(scaffold_name));

    let result: CallToolResult = client
        .call_tool(CallToolRequestParam {
            name: "construct".into(),
            arguments: Some(serde_json::Map::from_iter(args.into_iter())),
        })
        .await
        .expect("Failed to call construct tool");

    let output = &result.content[0].as_text().unwrap().text;
    assert!(output.contains("Missing value: name"));

    // 5. Cancel session
    let result: CallToolResult = client
        .call_tool(CallToolRequestParam {
            name: "cancel_session".into(),
            arguments: None,
        })
        .await
        .expect("Failed to call cancel_session tool");

    let output = &result.content[0].as_text().unwrap().text;
    assert!(output.contains("Session cancelled"));

    // 6. Verify session is gone by trying to provide values
    let provide_result = client
        .call_tool(rmcp::model::CallToolRequestParam {
            name: "provide_values".into(),
            arguments: Some(
                rmcp::serde_json::json!({
                    "values": { "name": "test" }
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        })
        .await
        .expect("Failed to call provide_values");

    assert_eq!(
        provide_result.content[0].as_text().unwrap().text,
        "No active session."
    );
}
