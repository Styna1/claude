use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

use crate::config::*;
use crate::db::Database;
use crate::game::engine::{build_state_for_player, SharedWorld};
use crate::protocol::messages::{ClientMessage, ServerMessage};

#[derive(Clone)]
pub struct WsState {
    pub world: SharedWorld,
    pub db: Arc<Database>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<WsState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: WsState) {
    let (mut sender, mut receiver) = socket.split();
    let player_id = Arc::new(RwLock::new(None::<u64>));

    // Wait for the Join message first
    let join_msg = match receiver.next().await {
        Some(Ok(Message::Text(text))) => {
            match serde_json::from_str::<ClientMessage>(&text) {
                Ok(ClientMessage::Join { name, token }) => Some((name, token)),
                _ => None,
            }
        }
        _ => None,
    };

    let (name, token) = match join_msg {
        Some(j) => j,
        None => {
            let _ = sender
                .send(Message::Text(
                    serde_json::to_string(&ServerMessage::Error {
                        message: "Expected join message".into(),
                    })
                    .unwrap()
                    .into(),
                ))
                .await;
            return;
        }
    };

    // Resolve user_id from token
    let user_id = token
        .as_deref()
        .and_then(|t| state.db.validate_session(t))
        .map(|u| u.id);

    let display_name = if name.trim().is_empty() {
        "Unnamed".to_string()
    } else {
        name.chars().take(20).collect()
    };

    // Add player to world
    let id = {
        let mut world = state.world.write().await;
        world.add_player(display_name, user_id)
    };
    *player_id.write().await = Some(id);

    // Send joined confirmation
    let joined_msg = serde_json::to_string(&ServerMessage::Joined {
        id,
        world_size: WORLD_SIZE,
    })
    .unwrap();
    if sender.send(Message::Text(joined_msg.into())).await.is_err() {
        cleanup(&state.world, id).await;
        return;
    }

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    // Task: send game state to client at tick rate
    let world_clone = state.world.clone();
    let tx_clone = tx.clone();
    let send_task = tokio::spawn(async move {
        let mut tick = interval(Duration::from_millis(TICK_DURATION_MS));
        loop {
            tick.tick().await;
            let world = world_clone.read().await;

            // Check if player is dead
            if let Some(player) = world.players.get(&id) {
                if !player.alive {
                    let dead_msg = serde_json::to_string(&ServerMessage::Dead {
                        killer: world.get_killer_name(id),
                        score: player.score,
                    })
                    .unwrap();
                    let _ = tx_clone.send(dead_msg);
                    break;
                }
            } else {
                break;
            }

            if let Some(state_msg) = build_state_for_player(&world, id) {
                let json = serde_json::to_string(&state_msg).unwrap();
                if tx_clone.send(json).is_err() {
                    break;
                }
            }
        }
    });

    // Task: forward messages from channel to websocket
    let forward_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Main loop: receive input from client
    let world_clone = state.world.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        match client_msg {
                            ClientMessage::Move { x, y } => {
                                let mut world = world_clone.write().await;
                                if let Some(player) = world.players.get_mut(&id) {
                                    player.target_x = x;
                                    player.target_y = y;
                                }
                            }
                            ClientMessage::Split => {
                                let mut world = world_clone.write().await;
                                world.split_player(id);
                            }
                            ClientMessage::Eject => {
                                let mut world = world_clone.write().await;
                                world.eject_mass(id);
                            }
                            _ => {}
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for any task to finish
    tokio::select! {
        _ = send_task => {},
        _ = forward_task => {},
        _ = recv_task => {},
    }

    cleanup(&state.world, id).await;
}

async fn cleanup(world: &SharedWorld, player_id: u64) {
    let mut w = world.write().await;
    w.remove_player(player_id);
}
