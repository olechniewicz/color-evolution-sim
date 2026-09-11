/// Structure of Arrays (SoA) for cache-efficient storage
pub struct Entities {
    // Position data
    pub pos_x: Vec<f32>,
    pub pos_y: Vec<f32>,
    pub target_x: Vec<i32>,
    pub target_y: Vec<i32>,

    // Genetics (HSL color space)
    pub hue: Vec<f32>,        // 0.0 - 360.0
    pub saturation: Vec<f32>, // 0.7 - 1.0
    pub lightness: Vec<f32>,  // 0.3 - 0.7

    // Lifecycle
    pub birth_tick: Vec<u32>,
    pub is_alive: Vec<bool>,

    // Active agents count
    pub active_count: usize,
}

impl Entities {
    pub fn new(capacity: usize) -> Self {
        Self {
            pos_x: vec![0.0; capacity],
            pos_y: vec![0.0; capacity],
            target_x: vec![0; capacity],
            target_y: vec![0; capacity],
            hue: vec![0.0; capacity],
            saturation: vec![0.7; capacity],
            lightness: vec![0.5; capacity],
            birth_tick: vec![0; capacity],
            is_alive: vec![false; capacity],
            active_count: 0,
        }
    }

    pub fn add_agent(
        &mut self,
        x: i32,
        y: i32,
        hue: f32,
        saturation: f32,
        lightness: f32,
        birth_tick: u32,
    ) -> Option<usize> {
        // Find first dead agent slot or extend if needed
        let mut index = None;
        for i in 0..self.pos_x.len() {
            if !self.is_alive[i] {
                index = Some(i);
                break;
            }
        }

        let idx = match index {
            Some(i) => i,
            None => {
                let i = self.pos_x.len();
                self.pos_x.push(0.0);
                self.pos_y.push(0.0);
                self.target_x.push(0);
                self.target_y.push(0);
                self.hue.push(0.0);
                self.saturation.push(0.7);
                self.lightness.push(0.5);
                self.birth_tick.push(0);
                self.is_alive.push(false);
                i
            }
        };

        self.pos_x[idx] = x as f32;
        self.pos_y[idx] = y as f32;
        self.target_x[idx] = x;
        self.target_y[idx] = y;
        self.hue[idx] = hue;
        self.saturation[idx] = saturation;
        self.lightness[idx] = lightness;
        self.birth_tick[idx] = birth_tick;
        self.is_alive[idx] = true;
        self.active_count += 1;

        Some(idx)
    }

    pub fn kill_agent(&mut self, index: usize) {
        if index < self.is_alive.len() && self.is_alive[index] {
            self.is_alive[index] = false;
            self.active_count = self.active_count.saturating_sub(1);
        }
    }

    pub fn reset(&mut self) {
        for i in 0..self.is_alive.len() {
            self.is_alive[i] = false;
        }
        self.active_count = 0;
    }
}
