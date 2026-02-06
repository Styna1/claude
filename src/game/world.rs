use std::collections::HashMap;
use crate::config::*;
use crate::game::player::{Player, Cell};
use crate::game::food::{Food, Virus, EjectedMass};
use crate::game::physics;
use rand::Rng;

pub struct World {
    pub players: HashMap<u64, Player>,
    pub food: Vec<Food>,
    pub viruses: Vec<Virus>,
    pub ejected: Vec<EjectedMass>,
    next_player_id: u64,
}

impl World {
    pub fn new() -> Self {
        let mut food = Vec::with_capacity(FOOD_COUNT);
        for _ in 0..FOOD_COUNT {
            food.push(Food::random());
        }
        let mut viruses = Vec::with_capacity(VIRUS_COUNT);
        for _ in 0..VIRUS_COUNT {
            viruses.push(Virus::random());
        }
        World {
            players: HashMap::new(),
            food,
            viruses,
            ejected: Vec::new(),
            next_player_id: 1,
        }
    }

    pub fn add_player(&mut self, name: String, user_id: Option<i64>) -> u64 {
        let id = self.next_player_id;
        self.next_player_id += 1;

        let mut rng = rand::thread_rng();
        let margin = 200.0;
        let x = rng.gen_range(margin..WORLD_SIZE - margin);
        let y = rng.gen_range(margin..WORLD_SIZE - margin);

        let player = Player::new(id, name, user_id, x, y);
        self.players.insert(id, player);
        id
    }

    pub fn remove_player(&mut self, id: u64) {
        self.players.remove(&id);
    }

    pub fn tick(&mut self, dt: f64) {
        self.move_players(dt);
        self.move_ejected(dt);
        self.check_food_eating();
        self.check_ejected_eating();
        self.check_player_eating();
        self.check_virus_eating();
        self.update_merge_timers(dt);
        self.merge_cells();
        self.decay_mass(dt);
        self.replenish_food();
        self.replenish_viruses();
        self.push_apart_own_cells();

        for player in self.players.values_mut() {
            player.update_score();
        }
    }

    fn move_players(&mut self, dt: f64) {
        for player in self.players.values_mut() {
            if !player.alive {
                continue;
            }
            for cell in &mut player.cells {
                // Apply velocity from split/eject
                if cell.vx.abs() > 1.0 || cell.vy.abs() > 1.0 {
                    cell.x += cell.vx * dt;
                    cell.y += cell.vy * dt;
                    cell.vx *= SPLIT_DECEL;
                    cell.vy *= SPLIT_DECEL;
                } else {
                    cell.vx = 0.0;
                    cell.vy = 0.0;
                    // Normal movement toward target
                    let dx = player.target_x - cell.x;
                    let dy = player.target_y - cell.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist > 5.0 {
                        let speed = speed_for_mass(cell.mass) * dt;
                        let (nx, ny) = physics::normalize(dx, dy);
                        cell.x += nx * speed;
                        cell.y += ny * speed;
                    }
                }
                let r = cell.radius();
                let (cx, cy) = physics::clamp_to_world(cell.x, cell.y, r);
                cell.x = cx;
                cell.y = cy;
            }
        }
    }

    fn move_ejected(&mut self, dt: f64) {
        for ej in &mut self.ejected {
            ej.x += ej.vx * dt;
            ej.y += ej.vy * dt;
            ej.vx *= EJECT_DECEL;
            ej.vy *= EJECT_DECEL;
            let r = mass_to_radius(ej.mass);
            let (cx, cy) = physics::clamp_to_world(ej.x, ej.y, r);
            ej.x = cx;
            ej.y = cy;
        }
    }

    fn check_food_eating(&mut self) {
        let mut eaten_indices = Vec::new();
        for player in self.players.values_mut() {
            if !player.alive {
                continue;
            }
            for cell in &mut player.cells {
                for (fi, food) in self.food.iter().enumerate() {
                    if physics::can_eat_food(cell.x, cell.y, cell.mass, food.x, food.y) {
                        cell.mass += FOOD_MASS;
                        if !eaten_indices.contains(&fi) {
                            eaten_indices.push(fi);
                        }
                    }
                }
            }
        }
        eaten_indices.sort_unstable_by(|a, b| b.cmp(a));
        for i in eaten_indices {
            self.food.swap_remove(i);
        }
    }

    fn check_ejected_eating(&mut self) {
        let mut eaten_indices = Vec::new();
        for player in self.players.values_mut() {
            if !player.alive {
                continue;
            }
            for cell in &mut player.cells {
                for (ei, ej) in self.ejected.iter().enumerate() {
                    if ej.vx.abs() < 5.0 && ej.vy.abs() < 5.0 {
                        if physics::can_eat_food(cell.x, cell.y, cell.mass, ej.x, ej.y) {
                            cell.mass += ej.mass;
                            if !eaten_indices.contains(&ei) {
                                eaten_indices.push(ei);
                            }
                        }
                    }
                }
            }
        }
        eaten_indices.sort_unstable_by(|a, b| b.cmp(a));
        for i in eaten_indices {
            self.ejected.swap_remove(i);
        }
    }

    fn check_player_eating(&mut self) {
        let ids: Vec<u64> = self.players.keys().cloned().collect();
        let mut kills: Vec<(u64, u64, usize, f64)> = Vec::new(); // (eater_id, victim_id, victim_cell_idx, mass)

        for &id1 in &ids {
            for &id2 in &ids {
                if id1 == id2 {
                    continue;
                }
                let p1 = match self.players.get(&id1) {
                    Some(p) if p.alive => p,
                    _ => continue,
                };
                let p2 = match self.players.get(&id2) {
                    Some(p) if p.alive => p,
                    _ => continue,
                };

                for c1 in &p1.cells {
                    for (ci2, c2) in p2.cells.iter().enumerate() {
                        if physics::can_eat(c1.x, c1.y, c1.mass, c2.x, c2.y, c2.mass) {
                            kills.push((id1, id2, ci2, c2.mass));
                        }
                    }
                }
            }
        }

        // Process kills
        let mut dead_cells: HashMap<u64, Vec<usize>> = HashMap::new();
        let mut mass_gains: HashMap<u64, f64> = HashMap::new();

        for (eater_id, victim_id, cell_idx, mass) in &kills {
            dead_cells.entry(*victim_id).or_default().push(*cell_idx);
            *mass_gains.entry(*eater_id).or_insert(0.0) += *mass;
        }

        // Add mass to eaters (distribute to largest cell)
        for (eater_id, gained) in &mass_gains {
            if let Some(player) = self.players.get_mut(eater_id) {
                if let Some(cell) = player.cells.iter_mut().max_by(|a, b| a.mass.partial_cmp(&b.mass).unwrap()) {
                    cell.mass += gained;
                }
            }
        }

        // Remove dead cells
        for (victim_id, mut indices) in dead_cells {
            indices.sort_unstable_by(|a, b| b.cmp(a));
            indices.dedup();
            if let Some(player) = self.players.get_mut(&victim_id) {
                for idx in indices {
                    if idx < player.cells.len() {
                        player.cells.remove(idx);
                    }
                }
                if player.cells.is_empty() {
                    player.alive = false;
                }
            }
        }
    }

    fn check_virus_eating(&mut self) {
        let mut virus_eaten = Vec::new();
        for player in self.players.values_mut() {
            if !player.alive {
                continue;
            }
            for (vi, virus) in self.viruses.iter().enumerate() {
                for ci in 0..player.cells.len() {
                    let cell = &player.cells[ci];
                    if cell.mass >= VIRUS_SPLIT_MIN_MASS
                        && physics::can_eat_food(cell.x, cell.y, cell.mass, virus.x, virus.y)
                    {
                        // Virus pop: split cell into many pieces
                        if !virus_eaten.contains(&vi) {
                            virus_eaten.push(vi);
                        }
                        let split_count = (MAX_CELLS_PER_PLAYER - player.cells.len()).min(8);
                        if split_count > 0 {
                            let mass_per = cell.mass / (split_count as f64 + 1.0);
                            player.cells[ci].mass = mass_per;
                            let cx = player.cells[ci].x;
                            let cy = player.cells[ci].y;
                            for i in 0..split_count {
                                let angle = (i as f64 / split_count as f64) * std::f64::consts::TAU;
                                let mut new_cell = Cell::new(cx, cy, mass_per);
                                new_cell.vx = angle.cos() * SPLIT_LAUNCH_SPEED;
                                new_cell.vy = angle.sin() * SPLIT_LAUNCH_SPEED;
                                new_cell.merge_time = MERGE_TIME_SECS;
                                player.cells.push(new_cell);
                            }
                        }
                        break;
                    }
                }
            }
        }
        virus_eaten.sort_unstable_by(|a, b| b.cmp(a));
        for i in virus_eaten {
            self.viruses.swap_remove(i);
        }
    }

    fn update_merge_timers(&mut self, dt: f64) {
        for player in self.players.values_mut() {
            for cell in &mut player.cells {
                if cell.merge_time > 0.0 {
                    cell.merge_time -= dt;
                }
            }
        }
    }

    fn merge_cells(&mut self) {
        for player in self.players.values_mut() {
            if player.cells.len() <= 1 {
                continue;
            }
            let mut i = 0;
            while i < player.cells.len() {
                let mut j = i + 1;
                while j < player.cells.len() {
                    if player.cells[i].merge_time <= 0.0 && player.cells[j].merge_time <= 0.0 {
                        let dist = physics::distance(
                            player.cells[i].x, player.cells[i].y,
                            player.cells[j].x, player.cells[j].y,
                        );
                        let r1 = player.cells[i].radius();
                        let r2 = player.cells[j].radius();
                        if dist < r1.max(r2) {
                            // Merge j into i
                            player.cells[i].mass += player.cells[j].mass;
                            player.cells.remove(j);
                            continue;
                        }
                    }
                    j += 1;
                }
                i += 1;
            }
        }
    }

    fn decay_mass(&mut self, dt: f64) {
        for player in self.players.values_mut() {
            for cell in &mut player.cells {
                if cell.mass > DECAY_MIN_MASS {
                    cell.mass -= cell.mass * DECAY_RATE * dt;
                    if cell.mass < MIN_MASS {
                        cell.mass = MIN_MASS;
                    }
                }
            }
        }
    }

    fn push_apart_own_cells(&mut self) {
        for player in self.players.values_mut() {
            let len = player.cells.len();
            if len <= 1 {
                continue;
            }
            for i in 0..len {
                for j in (i + 1)..len {
                    // Only push apart cells that can't merge yet
                    if player.cells[i].merge_time > 0.0 || player.cells[j].merge_time > 0.0 {
                        let dist = physics::distance(
                            player.cells[i].x, player.cells[i].y,
                            player.cells[j].x, player.cells[j].y,
                        );
                        let r1 = player.cells[i].radius();
                        let r2 = player.cells[j].radius();
                        let min_dist = r1 + r2;
                        if dist < min_dist && dist > 0.01 {
                            let overlap = min_dist - dist;
                            let (nx, ny) = physics::normalize(
                                player.cells[j].x - player.cells[i].x,
                                player.cells[j].y - player.cells[i].y,
                            );
                            let push = overlap * 0.5;
                            player.cells[i].x -= nx * push;
                            player.cells[i].y -= ny * push;
                            player.cells[j].x += nx * push;
                            player.cells[j].y += ny * push;
                        }
                    }
                }
            }
        }
    }

    fn replenish_food(&mut self) {
        while self.food.len() < FOOD_COUNT {
            self.food.push(Food::random());
        }
    }

    fn replenish_viruses(&mut self) {
        while self.viruses.len() < VIRUS_COUNT {
            self.viruses.push(Virus::random());
        }
    }

    pub fn split_player(&mut self, player_id: u64) {
        let player = match self.players.get_mut(&player_id) {
            Some(p) if p.alive => p,
            _ => return,
        };
        if !player.can_split() {
            return;
        }

        let (tx, ty) = (player.target_x, player.target_y);
        let mut new_cells = Vec::new();
        let current_count = player.cells.len();

        for cell in player.cells.iter_mut() {
            if cell.mass >= SPLIT_MIN_MASS && new_cells.len() + current_count < MAX_CELLS_PER_PLAYER {
                let half = cell.mass / 2.0;
                cell.mass = half;
                cell.merge_time = MERGE_TIME_SECS;

                let (nx, ny) = physics::normalize(tx - cell.x, ty - cell.y);
                let mut new_cell = Cell::new(cell.x, cell.y, half);
                new_cell.vx = nx * SPLIT_LAUNCH_SPEED;
                new_cell.vy = ny * SPLIT_LAUNCH_SPEED;
                new_cell.merge_time = MERGE_TIME_SECS;
                new_cells.push(new_cell);
            }
        }
        player.cells.extend(new_cells);
    }

    pub fn eject_mass(&mut self, player_id: u64) {
        let player = match self.players.get_mut(&player_id) {
            Some(p) if p.alive => p,
            _ => return,
        };
        if !player.can_eject() {
            return;
        }

        let (tx, ty) = (player.target_x, player.target_y);
        let color = player.color.clone();

        for cell in player.cells.iter_mut() {
            if cell.mass >= EJECT_MIN_MASS {
                cell.mass -= EJECT_MASS;
                let (nx, ny) = physics::normalize(tx - cell.x, ty - cell.y);
                let r = cell.radius();
                self.ejected.push(EjectedMass {
                    x: cell.x + nx * r,
                    y: cell.y + ny * r,
                    mass: EJECT_MASS,
                    vx: nx * EJECT_SPEED,
                    vy: ny * EJECT_SPEED,
                    color: color.clone(),
                });
                break; // Only eject from one cell
            }
        }
    }

    pub fn get_leaderboard(&self) -> Vec<(String, u64)> {
        let mut entries: Vec<(String, u64)> = self
            .players
            .values()
            .filter(|p| p.alive)
            .map(|p| (p.name.clone(), p.total_mass() as u64))
            .collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.truncate(10);
        entries
    }

    /// Get which player killed this player (checks who recently ate them)
    pub fn get_killer_name(&self, _victim_id: u64) -> String {
        "".to_string()
    }
}


