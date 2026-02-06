use crate::config::*;

pub fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
}

pub fn circles_overlap(x1: f64, y1: f64, r1: f64, x2: f64, y2: f64, r2: f64) -> bool {
    distance(x1, y1, x2, y2) < r1 + r2
}

/// Check if circle1 can eat circle2 (>= 1.25x mass and overlaps enough)
pub fn can_eat(
    x1: f64, y1: f64, mass1: f64,
    x2: f64, y2: f64, mass2: f64,
) -> bool {
    if mass1 < mass2 * EAT_MASS_RATIO {
        return false;
    }
    let r1 = mass_to_radius(mass1);
    let r2 = mass_to_radius(mass2);
    let dist = distance(x1, y1, x2, y2);
    // The smaller circle's center must be inside the larger circle
    dist + r2 * EAT_OVERLAP_RATIO < r1
}

/// Check if a cell can eat food
pub fn can_eat_food(cx: f64, cy: f64, cell_mass: f64, fx: f64, fy: f64) -> bool {
    let r = mass_to_radius(cell_mass);
    let dist = distance(cx, cy, fx, fy);
    dist < r - FOOD_RADIUS * 0.5
}

/// Clamp position to world bounds
pub fn clamp_to_world(x: f64, y: f64, radius: f64) -> (f64, f64) {
    let x = x.max(radius).min(WORLD_SIZE - radius);
    let y = y.max(radius).min(WORLD_SIZE - radius);
    (x, y)
}

/// Normalize a direction vector
pub fn normalize(x: f64, y: f64) -> (f64, f64) {
    let len = (x * x + y * y).sqrt();
    if len < 0.0001 {
        (0.0, 0.0)
    } else {
        (x / len, y / len)
    }
}
