/// Global configuration constants for the simulation
pub struct Config {
    pub grid_size: usize,
    pub initial_population: usize,
    pub lifespan_ticks: u32,
    pub tick_interval_ms: u64,
    pub seed: u64,
    pub reproduction_radius: usize,
    pub stagnation_threshold: usize,
    pub stagnation_check_ticks: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            grid_size: 1000,
            initial_population: 10_000,
            lifespan_ticks: 500,
            tick_interval_ms: 500,
            seed: 12345,
            reproduction_radius: 2,
            stagnation_threshold: 100, // 1% of 10k
            stagnation_check_ticks: 20,
        }
    }
}
