use serde::{Deserialize, Serialize};

// ── Client → Server ──

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Join {
        name: String,
        #[serde(default)]
        token: Option<String>,
    },
    Move {
        x: f64,
        y: f64,
    },
    Split,
    Eject,
}

// ── Server → Client ──

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Joined {
        id: u64,
        world_size: f64,
    },
    State {
        players: Vec<PlayerState>,
        food: Vec<FoodState>,
        viruses: Vec<VirusState>,
        leaderboard: Vec<LeaderboardEntry>,
    },
    Dead {
        killer: String,
        score: u64,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Serialize, Clone)]
pub struct CellState {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct PlayerState {
    pub id: u64,
    pub name: String,
    pub skin: Option<String>,
    pub cells: Vec<CellState>,
}

#[derive(Debug, Serialize, Clone)]
pub struct FoodState {
    pub x: f64,
    pub y: f64,
    pub color: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct VirusState {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct LeaderboardEntry {
    pub name: String,
    pub score: u64,
}
