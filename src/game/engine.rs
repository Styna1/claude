use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use crate::config::*;
use crate::game::world::World;
use crate::protocol::messages::*;

pub type SharedWorld = Arc<RwLock<World>>;

pub fn create_world() -> SharedWorld {
    Arc::new(RwLock::new(World::new()))
}

pub async fn game_loop(world: SharedWorld) {
    let mut tick_interval = interval(Duration::from_millis(TICK_DURATION_MS));
    let dt = 1.0 / TICK_RATE as f64;

    loop {
        tick_interval.tick().await;
        let mut w = world.write().await;
        w.tick(dt);
    }
}

pub fn build_state_for_player(world: &World, player_id: u64) -> Option<ServerMessage> {
    let player = world.players.get(&player_id)?;
    if !player.alive {
        return None;
    }

    let (cx, cy) = player.center();
    let scale = player.viewport_scale();
    let view_size = BASE_VIEWPORT_SIZE * scale;

    let view_left = cx - view_size;
    let view_right = cx + view_size;
    let view_top = cy - view_size;
    let view_bottom = cy + view_size;

    // Collect visible players
    let players: Vec<PlayerState> = world
        .players
        .values()
        .filter(|p| p.alive)
        .filter(|p| {
            p.cells.iter().any(|c| {
                c.x + c.radius() > view_left
                    && c.x - c.radius() < view_right
                    && c.y + c.radius() > view_top
                    && c.y - c.radius() < view_bottom
            })
        })
        .map(|p| PlayerState {
            id: p.id,
            name: p.name.clone(),
            skin: p.skin_url(),
            cells: p
                .cells
                .iter()
                .map(|c| CellState {
                    x: c.x,
                    y: c.y,
                    radius: c.radius(),
                })
                .collect(),
        })
        .collect();

    // Collect visible food
    let food: Vec<FoodState> = world
        .food
        .iter()
        .filter(|f| f.x > view_left && f.x < view_right && f.y > view_top && f.y < view_bottom)
        .map(|f| FoodState {
            x: f.x,
            y: f.y,
            color: f.color.clone(),
        })
        .collect();

    // Collect visible viruses
    let viruses: Vec<VirusState> = world
        .viruses
        .iter()
        .filter(|v| {
            v.x + VIRUS_RADIUS > view_left
                && v.x - VIRUS_RADIUS < view_right
                && v.y + VIRUS_RADIUS > view_top
                && v.y - VIRUS_RADIUS < view_bottom
        })
        .map(|v| VirusState {
            x: v.x,
            y: v.y,
            radius: VIRUS_RADIUS,
        })
        .collect();

    let leaderboard: Vec<LeaderboardEntry> = world
        .get_leaderboard()
        .into_iter()
        .map(|(name, score)| LeaderboardEntry { name, score })
        .collect();

    Some(ServerMessage::State {
        players,
        food,
        viruses,
        leaderboard,
    })
}
