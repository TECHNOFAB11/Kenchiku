use crate::server::KenchikuMcpServer;
use rmcp::{
    RoleClient, ServiceExt,
    model::{
        ClientInfo, ClientResult, ErrorCode, ErrorData, Implementation, ListToolsResult,
        ServerNotification, ServerRequest,
    },
    service::{NotificationContext, RequestContext, Service},
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

#[tokio::test]
async fn test_mcp_server_initialize() {
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

    let client = rmcp::serve_client(client_service, (client_read, client_write))
        .await
        .unwrap();

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

    let client = rmcp::serve_client(client_service, (client_read, client_write))
        .await
        .unwrap();

    client
        .notify_initialized()
        .await
        .expect("Failed to notify initialized");

    let tools: ListToolsResult = client.list_tools(None).await.expect("Failed to list tools");
    assert!(tools.tools.len() >= 3);

    let tool_names: Vec<_> = tools.tools.iter().map(|t| t.name.as_ref()).collect();
    assert!(tool_names.contains(&"construct"));
    assert!(tool_names.contains(&"patch"));
    assert!(tool_names.contains(&"list"));
}
