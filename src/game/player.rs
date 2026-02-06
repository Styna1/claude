use crate::config::*;

#[derive(Debug, Clone)]
pub struct Cell {
    pub x: f64,
    pub y: f64,
    pub mass: f64,
    pub vx: f64, // velocity for split/eject momentum
    pub vy: f64,
    pub merge_time: f64, // seconds until this cell can merge
}

impl Cell {
    pub fn new(x: f64, y: f64, mass: f64) -> Self {
        Cell {
            x,
            y,
            mass,
            vx: 0.0,
            vy: 0.0,
            merge_time: 0.0,
        }
    }

    pub fn radius(&self) -> f64 {
        mass_to_radius(self.mass)
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub user_id: Option<i64>, // database user id if logged in
    pub cells: Vec<Cell>,
    pub target_x: f64,
    pub target_y: f64,
    pub color: String,
    pub alive: bool,
    pub score: u64,
}

impl Player {
    pub fn new(id: u64, name: String, user_id: Option<i64>, x: f64, y: f64) -> Self {
        let color = random_color();
        Player {
            id,
            name,
            user_id,
            cells: vec![Cell::new(x, y, STARTING_MASS)],
            target_x: x,
            target_y: y,
            color,
            alive: true,
            score: 0,
        }
    }

    pub fn total_mass(&self) -> f64 {
        self.cells.iter().map(|c| c.mass).sum()
    }

    pub fn center(&self) -> (f64, f64) {
        if self.cells.is_empty() {
            return (0.0, 0.0);
        }
        let total_mass = self.total_mass();
        if total_mass == 0.0 {
            return (self.cells[0].x, self.cells[0].y);
        }
        let cx = self.cells.iter().map(|c| c.x * c.mass).sum::<f64>() / total_mass;
        let cy = self.cells.iter().map(|c| c.y * c.mass).sum::<f64>() / total_mass;
        (cx, cy)
    }

    pub fn viewport_scale(&self) -> f64 {
        let total = self.total_mass();
        (total / STARTING_MASS).sqrt().max(1.0)
    }

    pub fn update_score(&mut self) {
        let mass = self.total_mass() as u64;
        if mass > self.score {
            self.score = mass;
        }
    }

    pub fn skin_url(&self) -> Option<String> {
        self.user_id.map(|uid| format!("/api/skin/{}", uid))
    }

    pub fn can_split(&self) -> bool {
        self.cells.len() < MAX_CELLS_PER_PLAYER
            && self.cells.iter().any(|c| c.mass >= SPLIT_MIN_MASS)
    }

    pub fn can_eject(&self) -> bool {
        self.cells.iter().any(|c| c.mass >= EJECT_MIN_MASS)
    }
}

fn random_color() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let colors = [
        "#FF4136", "#FF6B35", "#FFDC00", "#2ECC40", "#0074D9",
        "#7FDBFF", "#B10DC9", "#F012BE", "#FF69B4", "#01FF70",
        "#3D9970", "#39CCCC", "#E65100", "#00BCD4", "#8BC34A",
    ];
    colors[rng.gen_range(0..colors.len())].to_string()
}
