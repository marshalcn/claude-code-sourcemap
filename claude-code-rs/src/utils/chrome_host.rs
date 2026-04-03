use anyhow::{Context, Result};
use bytes::{Buf, BufMut, BytesMut};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use serde_json::{json, Value};

const MAX_MESSAGE_SIZE: u32 = 1024 * 1024; // 1MB
const VERSION: &str = "1.0.0";

/// Send a message to stdout using the Chrome Native Messaging Protocol
/// (4-byte little-endian length prefix followed by JSON string).
pub async fn send_chrome_message(message: &str) -> Result<()> {
    let json_bytes = message.as_bytes();
    let length = json_bytes.len() as u32;
    
    let mut stdout = tokio::io::stdout();
    let mut len_buf = [0u8; 4];
    len_buf.copy_from_slice(&length.to_le_bytes());
    
    stdout.write_all(&len_buf).await?;
    stdout.write_all(json_bytes).await?;
    stdout.flush().await?;
    
    Ok(())
}

struct McpClient {
    #[allow(dead_code)]
    id: usize,
    // For a full implementation, this would hold the stream writer half
    // stream: tokio::net::unix::OwnedWriteHalf,
}

pub struct ChromeNativeHost {
    mcp_clients: Arc<Mutex<HashMap<usize, McpClient>>>,
    next_client_id: Arc<Mutex<usize>>,
    socket_path: String,
}

impl ChromeNativeHost {
    pub fn new(socket_path: String) -> Self {
        Self {
            mcp_clients: Arc::new(Mutex::new(HashMap::new())),
            next_client_id: Arc::new(Mutex::new(1)),
            socket_path,
        }
    }

    pub async fn start(&self) -> Result<()> {
        // Clean up stale socket if it exists
        let _ = tokio::fs::remove_file(&self.socket_path).await;
        
        let listener = UnixListener::bind(&self.socket_path)
            .context("Failed to bind Unix socket for Chrome Native Host")?;
            
        // Set secure permissions (0600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(&self.socket_path).await?.permissions();
            perms.set_mode(0o600);
            tokio::fs::set_permissions(&self.socket_path, perms).await?;
        }
        
        let clients = self.mcp_clients.clone();
        let next_id = self.next_client_id.clone();
        
        // Spawn a task to accept MCP client connections
        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                Self::handle_mcp_client(stream, clients.clone(), next_id.clone()).await;
            }
        });
        
        Ok(())
    }
    
    async fn handle_mcp_client(
        mut stream: UnixStream, 
        clients: Arc<Mutex<HashMap<usize, McpClient>>>,
        next_id: Arc<Mutex<usize>>
    ) {
        let mut id_guard = next_id.lock().await;
        let client_id = *id_guard;
        *id_guard += 1;
        drop(id_guard);
        
        clients.lock().await.insert(client_id, McpClient { id: client_id });
        
        // Notify Chrome of new connection
        let _ = send_chrome_message(&json!({ "type": "mcp_connected" }).to_string()).await;
        
        // Read loop for MCP client
        tokio::spawn(async move {
            let mut buffer = BytesMut::new();
            let mut read_buf = [0u8; 4096];
            
            loop {
                match stream.read(&mut read_buf).await {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        buffer.put_slice(&read_buf[..n]);
                        
                        // Parse length-prefixed messages
                        while buffer.len() >= 4 {
                            let mut len_bytes = [0u8; 4];
                            len_bytes.copy_from_slice(&buffer[0..4]);
                            let length = u32::from_le_bytes(len_bytes) as usize;
                            
                            if length == 0 || length > MAX_MESSAGE_SIZE as usize {
                                break; // Invalid message
                            }
                            
                            if buffer.len() < 4 + length {
                                break; // Need more data
                            }
                            
                            // Extract message
                            let msg_bytes = buffer[4..4+length].to_vec();
                            buffer.advance(4 + length);
                            
                            if let Ok(msg_str) = String::from_utf8(msg_bytes) {
                                if let Ok(parsed) = serde_json::from_str::<Value>(&msg_str) {
                                    // Forward to Chrome
                                    let _ = send_chrome_message(&json!({
                                        "type": "tool_request",
                                        "method": parsed["method"],
                                        "params": parsed["params"]
                                    }).to_string()).await;
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
            
            // Client disconnected
            clients.lock().await.remove(&client_id);
            let _ = send_chrome_message(&json!({ "type": "mcp_disconnected" }).to_string()).await;
        });
    }

    pub async fn handle_chrome_message(&self, message_str: &str) -> Result<()> {
        let parsed: Value = serde_json::from_str(message_str)?;
        
        let msg_type = parsed.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
        
        match msg_type {
            "ping" => {
                send_chrome_message(&json!({
                    "type": "pong",
                    "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
                }).to_string()).await?;
            }
            "get_status" => {
                send_chrome_message(&json!({
                    "type": "status_response",
                    "native_host_version": VERSION
                }).to_string()).await?;
            }
            "tool_response" | "notification" => {
                // In a full implementation, we would extract the payload and write it
                // back to the `tokio::net::unix::OwnedWriteHalf` of each connected MCP client.
                let clients = self.mcp_clients.lock().await;
                if !clients.is_empty() {
                    // let payload = ... format length-prefixed ...
                    // for client in clients.values() { client.stream.write_all(&payload).await?; }
                }
            }
            _ => {
                send_chrome_message(&json!({
                    "type": "error",
                    "error": format!("Unknown message type: {}", msg_type)
                }).to_string()).await?;
            }
        }
        
        Ok(())
    }
}

/// Reads Chrome Native Messaging messages from stdin asynchronously.
pub async fn read_chrome_messages(host: Arc<ChromeNativeHost>) -> Result<()> {
    let mut stdin = tokio::io::stdin();
    let mut buffer = BytesMut::new();
    let mut read_buf = [0u8; 4096];

    loop {
        match stdin.read(&mut read_buf).await {
            Ok(0) => break, // Stdin closed, Chrome disconnected
            Ok(n) => {
                buffer.put_slice(&read_buf[..n]);

                while buffer.len() >= 4 {
                    let mut len_bytes = [0u8; 4];
                    len_bytes.copy_from_slice(&buffer[0..4]);
                    let length = u32::from_le_bytes(len_bytes) as usize;

                    if length == 0 || length > MAX_MESSAGE_SIZE as usize {
                        return Err(anyhow::anyhow!("Invalid message length from Chrome: {}", length));
                    }

                    if buffer.len() < 4 + length {
                        break; // Wait for more data
                    }

                    let msg_bytes = buffer[4..4+length].to_vec();
                    buffer.advance(4 + length);

                    if let Ok(msg_str) = String::from_utf8(msg_bytes) {
                        if let Err(e) = host.handle_chrome_message(&msg_str).await {
                            eprintln!("Error handling Chrome message: {}", e);
                        }
                    }
                }
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}
