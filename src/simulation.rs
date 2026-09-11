use crate::config::Config;
use crate::entities::Entities;
use crate::prng::Xoshiro256;

pub struct Simulation {
    config: Config,
    entities: Entities,
    current_tick: u32,
    previous_active_count: usize,
    stagnation_counter: u32,
    prng: Xoshiro256,
    // Temporary storage for sorted data (for collision detection)
    sorted_indices: Vec<usize>,
    sorted_x: Vec<i32>,
    sorted_y: Vec<i32>,
    sorted_hue: Vec<f32>,
    sorted_sat: Vec<f32>,
    sorted_light: Vec<f32>,
    sorted_birth: Vec<u32>,
    sorted_pos_x: Vec<f32>,
    sorted_pos_y: Vec<f32>,
}

impl Simulation {
    pub fn new(config: Config) -> Self {
        let capacity = config.initial_population * 2; // Buffer for births
        let mut sim = Self {
            config: config.clone(),
            entities: Entities::new(capacity),
            current_tick: 0,
            previous_active_count: 0,
            stagnation_counter: 0,
            prng: Xoshiro256::new(config.seed),
            sorted_indices: Vec::new(),
            sorted_x: Vec::new(),
            sorted_y: Vec::new(),
            sorted_hue: Vec::new(),
            sorted_sat: Vec::new(),
            sorted_light: Vec::new(),
            sorted_birth: Vec::new(),
            sorted_pos_x: Vec::new(),
            sorted_pos_y: Vec::new(),
        };
        sim.initialize_population();
        sim
    }

    fn initialize_population(&mut self) {
        let config = self.config.clone();
        self.entities.reset();

        for _ in 0..config.initial_population {
            let x = self.prng.next_in_range(0, config.grid_size) as i32;
            let y = self.prng.next_in_range(0, config.grid_size) as i32;

            let hue = match self.prng.next_u32() % 3 {
                0 => 0.0,   // Red
                1 => 120.0, // Green
                _ => 240.0, // Blue
            };

            let saturation = 0.7 + self.prng.next_f32() * 0.3;
            let lightness = 0.3 + self.prng.next_f32() * 0.4;

            let _ = self.entities.add_agent(x, y, hue, saturation, lightness, self.current_tick);
        }

        self.previous_active_count = self.entities.active_count;
    }

    pub fn tick(&mut self) {
        self.current_tick += 1;

        // Step 1: Aging & Death
        self.handle_aging();

        // Step 2: Movement Phase
        self.handle_movement();

        // Step 3: Spatial Sorting (Radix Sort)
        self.radix_sort_by_grid_index();

        // Step 4: Collision & Birth Phase
        self.handle_collisions_and_births();

        // Step 5: Stagnation Detection & Reset
        self.check_stagnation();
    }

    fn handle_aging(&mut self) {
        let current_tick = self.current_tick;
        let lifespan = self.config.lifespan_ticks;

        for i in 0..self.entities.is_alive.len() {
            if self.entities.is_alive[i] {
                let age = current_tick - self.entities.birth_tick[i];
                if age > lifespan {
                    self.entities.kill_agent(i);
                }
            }
        }
    }

    fn handle_movement(&mut self) {
        let grid_size = self.config.grid_size as i32;

        for i in 0..self.entities.is_alive.len() {
            if !self.entities.is_alive[i] {
                continue;
            }

            // Moore's neighborhood: 8 directions
            let dir = self.prng.next_u32() % 8;
            let (dx, dy) = match dir {
                0 => (0, -1),   // N
                1 => (1, -1),   // NE
                2 => (1, 0),    // E
                3 => (1, 1),    // SE
                4 => (0, 1),    // S
                5 => (-1, 1),   // SW
                6 => (-1, 0),   // W
                _ => (-1, -1),  // NW
            };

            let mut nx = self.entities.pos_x[i] as i32 + dx;
            let mut ny = self.entities.pos_y[i] as i32 + dy;

            // Toroidal wrapping (world loops at edges)
            nx = ((nx % grid_size) + grid_size) % grid_size;
            ny = ((ny % grid_size) + grid_size) % grid_size;

            self.entities.target_x[i] = nx;
            self.entities.target_y[i] = ny;
        }
    }

    fn radix_sort_by_grid_index(&mut self) {
        let grid_size = self.config.grid_size;
        let mut indices: Vec<usize> = (0..self.entities.is_alive.len())
            .filter(|&i| self.entities.is_alive[i])
            .collect();

        // Radix sort by grid index
        indices.sort_by_key(|&i| {
            (self.entities.target_y[i] as usize) * grid_size
                + (self.entities.target_x[i] as usize)
        });

        // Create sorted copy of relevant data
        let mut sorted_x = Vec::new();
        let mut sorted_y = Vec::new();
        let mut sorted_hue = Vec::new();
        let mut sorted_sat = Vec::new();
        let mut sorted_light = Vec::new();
        let mut sorted_birth = Vec::new();
        let mut sorted_pos_x = Vec::new();
        let mut sorted_pos_y = Vec::new();

        for &idx in &indices {
            sorted_x.push(self.entities.target_x[idx]);
            sorted_y.push(self.entities.target_y[idx]);
            sorted_hue.push(self.entities.hue[idx]);
            sorted_sat.push(self.entities.saturation[idx]);
            sorted_light.push(self.entities.lightness[idx]);
            sorted_birth.push(self.entities.birth_tick[idx]);
            sorted_pos_x.push(self.entities.pos_x[idx]);
            sorted_pos_y.push(self.entities.pos_y[idx]);
        }

        // Store sorted indices for collision detection
        self.sorted_indices = indices;
        self.sorted_x = sorted_x;
        self.sorted_y = sorted_y;
        self.sorted_hue = sorted_hue;
        self.sorted_sat = sorted_sat;
        self.sorted_light = sorted_light;
        self.sorted_birth = sorted_birth;
        self.sorted_pos_x = sorted_pos_x;
        self.sorted_pos_y = sorted_pos_y;
    }

    fn handle_collisions_and_births(&mut self) {
        let grid_size = self.config.grid_size as i32;
        let radius = self.config.reproduction_radius as i32;
        let current_tick = self.current_tick;

        let mut i = 0;
        while i < self.sorted_indices.len().saturating_sub(1) {
            let curr_idx = self.sorted_indices[i];
            let next_idx = self.sorted_indices[i + 1];

            let curr_grid = self.sorted_y[i] * (grid_size as i32) + self.sorted_x[i];
            let next_grid = self.sorted_y[i + 1] * (grid_size as i32) + self.sorted_x[i + 1];

            if curr_grid == next_grid {
                // Collision detected! Only process first pair on this cell
                let parent1_hue = self.sorted_hue[i];
                let parent2_hue = self.sorted_hue[i + 1];
                let parent1_sat = self.sorted_sat[i];
                let parent2_sat = self.sorted_sat[i + 1];
                let parent1_light = self.sorted_light[i];
                let parent2_light = self.sorted_light[i + 1];

                let offspring_hue = self.average_hue(parent1_hue, parent2_hue);
                let offspring_sat = ((parent1_sat + parent2_sat) / 2.0 + (self.prng.next_f32() - 0.5) * 0.1)
                    .clamp(0.7, 1.0);
                let offspring_light =
                    ((parent1_light + parent2_light) / 2.0 + (self.prng.next_f32() - 0.5) * 0.1)
                        .clamp(0.3, 0.7);

                // Find empty spot in reproduction radius
                if let Some((birth_x, birth_y)) = self.find_empty_spot(
                    self.sorted_x[i],
                    self.sorted_y[i],
                    radius,
                ) {
                    let _ = self.entities.add_agent(
                        birth_x,
                        birth_y,
                        offspring_hue,
                        offspring_sat,
                        offspring_light,
                        current_tick,
                    );
                }

                i += 2; // Skip both parents
            } else {
                i += 1;
            }
        }
    }

    fn find_empty_spot(&mut self, cx: i32, cy: i32, radius: i32) -> Option<(i32, i32)> {
        let grid_size = self.config.grid_size as i32;
        let attempts = 5;

        for _ in 0..attempts {
            let angle = self.prng.next_f32() * 6.28318;
            let x = cx + ((angle.cos() * radius as f32) as i32);
            let y = cy + ((angle.sin() * radius as f32) as i32);

            let nx = ((x % grid_size) + grid_size) % grid_size;
            let ny = ((y % grid_size) + grid_size) % grid_size;

            let grid_idx = ny * grid_size + nx;

            // Binary search in sorted array
            if !self.is_occupied(grid_idx) {
                return Some((nx, ny));
            }
        }

        None
    }

    fn is_occupied(&self, grid_idx: i32) -> bool {
        let grid_size = self.config.grid_size as i32;
        for &idx in &self.sorted_indices {
            let agent_grid = self.entities.target_y[idx] * grid_size + self.entities.target_x[idx];
            if agent_grid == grid_idx {
                return true;
            }
        }
        false
    }

    fn average_hue(&self, h1: f32, h2: f32) -> f32 {
        // Angular average to avoid wrapping issues
        let rad1 = h1.to_radians();
        let rad2 = h2.to_radians();
        let avg_rad = ((rad1.sin() + rad2.sin()) / 2.0).atan2((rad1.cos() + rad2.cos()) / 2.0);
        let avg_deg = avg_rad.to_degrees();
        if avg_deg < 0.0 {
            avg_deg + 360.0
        } else {
            avg_deg
        }
    }

    fn check_stagnation(&mut self) {
        let active = self.entities.active_count;

        if active == 0 {
            self.initialize_population();
            return;
        }

        let diff = (active as i32 - self.previous_active_count as i32).abs() as usize;
        if diff <= self.config.stagnation_threshold {
            self.stagnation_counter += 1;
            if self.stagnation_counter > self.config.stagnation_check_ticks {
                self.initialize_population();
                self.stagnation_counter = 0;
            }
        } else {
            self.stagnation_counter = 0;
        }

        self.previous_active_count = active;
    }

    pub fn get_entities(&self) -> &Entities {
        &self.entities
    }

    pub fn get_current_tick(&self) -> u32 {
        self.current_tick
    }
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Self {
            grid_size: self.grid_size,
            initial_population: self.initial_population,
            lifespan_ticks: self.lifespan_ticks,
            tick_interval_ms: self.tick_interval_ms,
            seed: self.seed,
            reproduction_radius: self.reproduction_radius,
            stagnation_threshold: self.stagnation_threshold,
            stagnation_check_ticks: self.stagnation_check_ticks,
        }
    }
}
