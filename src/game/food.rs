use crate::config::*;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Food {
    pub x: f64,
    pub y: f64,
    pub color: String,
}

impl Food {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let colors = [
            "#FF6384", "#36A2EB", "#FFCE56", "#4BC0C0", "#9966FF",
            "#FF9F40", "#E7E9ED", "#7CB342", "#F06292", "#4DD0E1",
        ];
        Food {
            x: rng.gen_range(0.0..WORLD_SIZE),
            y: rng.gen_range(0.0..WORLD_SIZE),
            color: colors[rng.gen_range(0..colors.len())].to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EjectedMass {
    pub x: f64,
    pub y: f64,
    pub mass: f64,
    pub vx: f64,
    pub vy: f64,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct Virus {
    pub x: f64,
    pub y: f64,
}

impl Virus {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        Virus {
            x: rng.gen_range(VIRUS_RADIUS..WORLD_SIZE - VIRUS_RADIUS),
            y: rng.gen_range(VIRUS_RADIUS..WORLD_SIZE - VIRUS_RADIUS),
        }
    }
}
