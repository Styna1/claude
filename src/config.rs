// Game world constants
pub const WORLD_SIZE: f64 = 4000.0;
pub const TICK_RATE: u64 = 30; // ticks per second (30 TPS for network sanity)
pub const TICK_DURATION_MS: u64 = 1000 / TICK_RATE;

// Player constants
pub const STARTING_MASS: f64 = 10.0;
pub const MIN_MASS: f64 = 10.0;
pub const BASE_SPEED: f64 = 300.0; // pixels per second at mass=10
pub const EAT_OVERLAP_RATIO: f64 = 0.5;
pub const EAT_MASS_RATIO: f64 = 1.25;
pub const MAX_CELLS_PER_PLAYER: usize = 16;
pub const MERGE_TIME_SECS: f64 = 30.0;
pub const DECAY_RATE: f64 = 0.002; // mass lost per tick for large cells
pub const DECAY_MIN_MASS: f64 = 50.0;

// Split constants
pub const SPLIT_MIN_MASS: f64 = 36.0;
pub const SPLIT_LAUNCH_SPEED: f64 = 800.0;
pub const SPLIT_DECEL: f64 = 0.9; // friction per tick

// Eject mass constants
pub const EJECT_MASS: f64 = 14.0;
pub const EJECT_MIN_MASS: f64 = 32.0;
pub const EJECT_SPEED: f64 = 600.0;
pub const EJECT_DECEL: f64 = 0.88;

// Food constants
pub const FOOD_COUNT: usize = 500;
pub const FOOD_MASS: f64 = 1.0;
pub const FOOD_RADIUS: f64 = 5.0;

// Virus constants
pub const VIRUS_COUNT: usize = 15;
pub const VIRUS_MASS: f64 = 100.0;
pub const VIRUS_RADIUS: f64 = 40.0;
pub const VIRUS_SPLIT_MIN_MASS: f64 = 130.0;

// Viewport
pub const BASE_VIEWPORT_SIZE: f64 = 800.0;

// Server
pub const SERVER_PORT: u16 = 63012;
pub const SESSION_EXPIRY_HOURS: i64 = 24 * 7; // 1 week
pub const MAX_SKIN_SIZE: usize = 256 * 1024; // 256KB

// Helper: mass to radius
pub fn mass_to_radius(mass: f64) -> f64 {
    (mass).sqrt() * 4.0
}

// Helper: speed based on mass
pub fn speed_for_mass(mass: f64) -> f64 {
    BASE_SPEED / (mass).sqrt()
}
