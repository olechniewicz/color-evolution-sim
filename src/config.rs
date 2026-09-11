use crate::prng::Xoshiro256;

#[derive(Clone)]
pub struct Config {
    pub grid_size: usize,
    pub initial_population: usize,
    pub lifespan_ticks: u32,
    pub tick_interval_ms: u64,
    pub seed: u64,
    pub reproduction_radius: i32,
    pub stagnation_threshold: usize,
    pub stagnation_check_ticks: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            grid_size: 100,           // Changed from 1000 to 100 (100x100 grid)
            initial_population: 1000, // Changed from 10000 to 1000
            lifespan_ticks: 500,
            tick_interval_ms: 100,    // Changed from 500 to 100 (5x faster)
            seed: 12345,
            reproduction_radius: 2,
            stagnation_threshold: 50,
            stagnation_check_ticks: 20,
        }
    }
}
