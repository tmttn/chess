//! WebSocket bridge server for UCI chess bots.
//!
//! This server accepts WebSocket connections from the browser and routes
//! UCI commands to/from bot processes via stdin/stdout.
//! Supports multiple concurrent bot sessions per connection.

mod config;
mod session;

use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::Message;

use config::Config;
use session::BotSession;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::load().await?;
    let config = Arc::new(config);

    let addr: SocketAddr = format!("127.0.0.1:{}", config.port).parse()?;
    let listener = TcpListener::bind(&addr).await?;

    println!("Bot bridge listening on ws://{}", addr);
    println!(
        "Available bots: {:?}",
        config.bots.keys().collect::<Vec<_>>()
    );

    while let Ok((stream, peer)) = listener.accept().await {
        let config = Arc::clone(&config);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, peer, config).await {
                eprintln!("Connection error from {}: {}", peer, e);
            }
        });
    }

    Ok(())
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    peer: SocketAddr,
    config: Arc<Config>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("New connection from {}", peer);

    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Multiple bot sessions keyed by bot name
    let sessions: Arc<RwLock<HashMap<String, BotSession>>> = Arc::new(RwLock::new(HashMap::new()));

    // Channel for bot output -> websocket
    let (bot_tx, mut bot_rx) = tokio::sync::mpsc::channel::<String>(100);

    // Task to forward bot output to websocket
    let sessions_clone = Arc::clone(&sessions);
    let forward_task = tokio::spawn(async move {
        while let Some(line) = bot_rx.recv().await {
            // Check if this is already a control message (JSON with "type" field)
            let msg_str = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                if json.get("type").is_some() {
                    // Already a control message, pass through directly
                    line
                } else {
                    // Wrap as UCI output
                    serde_json::json!({ "type": "uci", "line": line }).to_string()
                }
            } else {
                // Not JSON, wrap as UCI output
                serde_json::json!({ "type": "uci", "line": line }).to_string()
            };

            if ws_sender.send(Message::Text(msg_str.into())).await.is_err() {
                break;
            }
        }
        // Clean up all sessions on disconnect
        let mut sessions = sessions_clone.write().await;
        for (_, sess) in sessions.drain() {
            sess.stop().await;
        }
    });

    // Handle incoming websocket messages
    while let Some(msg) = ws_receiver.next().await {
        let msg = match msg {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_)) => break,
            Ok(_) => continue,
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        };

        // Parse JSON message
        let json: serde_json::Value = match serde_json::from_str(&msg) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let msg_type = json["type"].as_str().unwrap_or("");

        match msg_type {
            "list" => {
                // List available bots
                let bots: Vec<&String> = config.bots.keys().collect();
                let response = serde_json::json!({
                    "type": "bots",
                    "bots": bots
                });
                bot_tx.send(response.to_string()).await.ok();
            }

            "connect" => {
                let bot_name = json["bot"].as_str().unwrap_or("");

                // Check if already connected to this bot
                if sessions.read().await.contains_key(bot_name) {
                    let response = serde_json::json!({
                        "type": "connected",
                        "bot": bot_name,
                        "session": "existing"
                    });
                    bot_tx.send(response.to_string()).await.ok();
                    continue;
                }

                // Look up bot config
                if let Some(bot_config) = config.bots.get(bot_name) {
                    match BotSession::spawn(&bot_config.command, bot_tx.clone()).await {
                        Ok(sess) => {
                            let session_id = sess.id.clone();
                            sessions.write().await.insert(bot_name.to_string(), sess);

                            let response = serde_json::json!({
                                "type": "connected",
                                "bot": bot_name,
                                "session": session_id
                            });
                            bot_tx.send(response.to_string()).await.ok();
                        }
                        Err(e) => {
                            let response = serde_json::json!({
                                "type": "error",
                                "message": format!("Failed to spawn bot: {}", e)
                            });
                            bot_tx.send(response.to_string()).await.ok();
                        }
                    }
                } else {
                    let response = serde_json::json!({
                        "type": "error",
                        "message": format!("Unknown bot: {}", bot_name)
                    });
                    bot_tx.send(response.to_string()).await.ok();
                }
            }

            "uci" => {
                let cmd = json["cmd"].as_str().unwrap_or("");
                // Bot name is optional - if not provided, send to all active bots
                // (useful for simple single-bot scenarios)
                let bot_name = json["bot"].as_str();

                let sessions_read = sessions.read().await;
                if let Some(name) = bot_name {
                    // Send to specific bot
                    if let Some(sess) = sessions_read.get(name) {
                        sess.send(cmd).await.ok();
                    }
                } else {
                    // Send to most recently connected bot (last in iteration)
                    // For backwards compatibility
                    if let Some((_, sess)) = sessions_read.iter().last() {
                        sess.send(cmd).await.ok();
                    }
                }
            }

            "disconnect" => {
                let bot_name = json["bot"].as_str();

                if let Some(name) = bot_name {
                    // Disconnect specific bot
                    if let Some(sess) = sessions.write().await.remove(name) {
                        sess.stop().await;
                        let response = serde_json::json!({
                            "type": "disconnected",
                            "bot": name,
                            "reason": "user requested"
                        });
                        bot_tx.send(response.to_string()).await.ok();
                    }
                } else {
                    // Disconnect all bots
                    let mut sessions = sessions.write().await;
                    for (name, sess) in sessions.drain() {
                        sess.stop().await;
                        let response = serde_json::json!({
                            "type": "disconnected",
                            "bot": name,
                            "reason": "user requested"
                        });
                        bot_tx.send(response.to_string()).await.ok();
                    }
                }
            }

            _ => {}
        }
    }

    // Clean up
    forward_task.abort();
    let mut sessions = sessions.write().await;
    for (_, sess) in sessions.drain() {
        sess.stop().await;
    }

    println!("Connection closed from {}", peer);
    Ok(())
}
