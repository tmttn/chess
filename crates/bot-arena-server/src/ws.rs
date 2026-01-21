//! WebSocket handler for live match updates.
//!
//! This module provides WebSocket functionality for real-time streaming of
//! match events to connected clients. Clients can subscribe to specific
//! matches to receive move updates, game endings, and match results.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

/// WebSocket messages for match updates.
///
/// All messages use snake_case tag names for JSON serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// Client requests to subscribe to a match.
    Subscribe {
        /// The match ID to subscribe to.
        match_id: String,
    },
    /// Client requests to unsubscribe from a match.
    Unsubscribe {
        /// The match ID to unsubscribe from.
        match_id: String,
    },
    /// A move was made in a game.
    Move {
        /// The match ID.
        match_id: String,
        /// The move in UCI notation (e.g., "e2e4").
        uci: String,
        /// Optional centipawn evaluation of the position.
        centipawns: Option<i32>,
    },
    /// A game in the match has ended.
    GameEnd {
        /// The match ID.
        match_id: String,
        /// The game result (e.g., "1-0", "0-1", "1/2-1/2").
        result: String,
        /// The game number in the match.
        game_num: i32,
    },
    /// The entire match has ended.
    MatchEnd {
        /// The match ID.
        match_id: String,
        /// The final score (e.g., "5.5-4.5").
        score: String,
    },
    /// A new match has started.
    MatchStarted {
        /// The match ID.
        match_id: String,
        /// The name of the white player/bot.
        white: String,
        /// The name of the black player/bot.
        black: String,
    },
}

/// Broadcast channel sender for WebSocket messages.
pub type WsBroadcast = broadcast::Sender<WsMessage>;

/// Creates a new broadcast channel for WebSocket messages.
///
/// Returns the sender half of the channel. The channel has a capacity of 100
/// messages before older messages are dropped.
pub fn create_broadcast() -> WsBroadcast {
    let (tx, _) = broadcast::channel(100);
    tx
}

/// Axum handler for WebSocket upgrade requests.
///
/// Upgrades the HTTP connection to a WebSocket connection and spawns the
/// handler task.
pub async fn ws_handler(ws: WebSocketUpgrade, State(broadcast): State<WsBroadcast>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, broadcast))
}

/// Handles an established WebSocket connection.
///
/// This function manages bidirectional communication:
/// - Receives subscription/unsubscription requests from the client
/// - Forwards relevant broadcast messages to the client based on subscriptions
async fn handle_socket(socket: WebSocket, broadcast: WsBroadcast) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = broadcast.subscribe();

    // Subscribed match IDs
    let subscriptions = Arc::new(tokio::sync::RwLock::new(Vec::<String>::new()));
    let subs_clone = subscriptions.clone();

    // Task to forward broadcast messages to client
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let match_id = match &msg {
                WsMessage::Move { match_id, .. } => match_id,
                WsMessage::GameEnd { match_id, .. } => match_id,
                WsMessage::MatchEnd { match_id, .. } => match_id,
                WsMessage::MatchStarted { match_id, .. } => match_id,
                // Subscribe/Unsubscribe are client-to-server only
                WsMessage::Subscribe { .. } | WsMessage::Unsubscribe { .. } => continue,
            };

            let subs = subs_clone.read().await;
            if subs.contains(match_id) {
                let json = serde_json::to_string(&msg).unwrap();
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages from client
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                match ws_msg {
                    WsMessage::Subscribe { match_id } => {
                        let mut subs = subscriptions.write().await;
                        if !subs.contains(&match_id) {
                            subs.push(match_id);
                        }
                    }
                    WsMessage::Unsubscribe { match_id } => {
                        let mut subs = subscriptions.write().await;
                        subs.retain(|id| id != &match_id);
                    }
                    // Ignore other message types from client
                    _ => {}
                }
            }
        }
    }

    send_task.abort();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_move_serialization() {
        let msg = WsMessage::Move {
            match_id: "123".to_string(),
            uci: "e2e4".to_string(),
            centipawns: Some(30),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"move\""));
        assert!(json.contains("\"match_id\":\"123\""));
        assert!(json.contains("\"uci\":\"e2e4\""));
        assert!(json.contains("\"centipawns\":30"));
    }

    #[test]
    fn test_ws_message_move_without_centipawns() {
        let msg = WsMessage::Move {
            match_id: "456".to_string(),
            uci: "d7d5".to_string(),
            centipawns: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"move\""));
        assert!(json.contains("\"centipawns\":null"));
    }

    #[test]
    fn test_ws_message_game_end_serialization() {
        let msg = WsMessage::GameEnd {
            match_id: "match-1".to_string(),
            result: "1-0".to_string(),
            game_num: 3,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"game_end\""));
        assert!(json.contains("\"result\":\"1-0\""));
        assert!(json.contains("\"game_num\":3"));
    }

    #[test]
    fn test_ws_message_match_end_serialization() {
        let msg = WsMessage::MatchEnd {
            match_id: "match-1".to_string(),
            score: "5.5-4.5".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"match_end\""));
        assert!(json.contains("\"score\":\"5.5-4.5\""));
    }

    #[test]
    fn test_ws_message_match_started_serialization() {
        let msg = WsMessage::MatchStarted {
            match_id: "match-2".to_string(),
            white: "Stockfish".to_string(),
            black: "Random Bot".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"match_started\""));
        assert!(json.contains("\"white\":\"Stockfish\""));
        assert!(json.contains("\"black\":\"Random Bot\""));
    }

    #[test]
    fn test_ws_message_subscribe_serialization() {
        let msg = WsMessage::Subscribe {
            match_id: "test-match".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"subscribe\""));
        assert!(json.contains("\"match_id\":\"test-match\""));
    }

    #[test]
    fn test_ws_message_unsubscribe_serialization() {
        let msg = WsMessage::Unsubscribe {
            match_id: "test-match".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"unsubscribe\""));
    }

    #[test]
    fn test_ws_message_deserialization() {
        let json = r#"{"type":"subscribe","match_id":"abc-123"}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();

        match msg {
            WsMessage::Subscribe { match_id } => {
                assert_eq!(match_id, "abc-123");
            }
            _ => panic!("Expected Subscribe message"),
        }
    }

    #[test]
    fn test_ws_message_move_deserialization() {
        let json = r#"{"type":"move","match_id":"m1","uci":"e2e4","centipawns":15}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();

        match msg {
            WsMessage::Move {
                match_id,
                uci,
                centipawns,
            } => {
                assert_eq!(match_id, "m1");
                assert_eq!(uci, "e2e4");
                assert_eq!(centipawns, Some(15));
            }
            _ => panic!("Expected Move message"),
        }
    }

    #[test]
    fn test_create_broadcast() {
        let tx = create_broadcast();
        // Verify we can subscribe
        let _rx = tx.subscribe();
        // Verify we can send (even if no receivers)
        let _ = tx.send(WsMessage::Subscribe {
            match_id: "test".to_string(),
        });
    }
}
