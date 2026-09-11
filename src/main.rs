mod config;
mod entities;
mod prng;
mod renderer;
mod simulation;

use config::Config;
use renderer::Renderer;
use simulation::Simulation;
use std::time::{Duration, Instant};

fn main() {
    // Initialize configuration
    let config = Config::default();

    // Initialize simulation and renderer
    let mut simulation = Simulation::new(config.clone());
    let mut renderer = Renderer::new(config.grid_size);

    // Timing
    let tick_interval = Duration::from_millis(config.tick_interval_ms);
    let frame_interval = Duration::from_millis(16); // ~60 FPS

    let mut last_tick_time = Instant::now();
    let mut last_frame_time = Instant::now();

    println!("[COLOR EVOLUTION SIM] Starting...");
    println!("Grid: {}x{}", config.grid_size, config.grid_size);
    println!("Initial Population: {}", config.initial_population);
    println!("Tick Interval: {} ms", config.tick_interval_ms);

    // Main loop
    let mut running = true;
    while running {
        let now = Instant::now();

        // Logic tick (every 500ms)
        if now.duration_since(last_tick_time) >= tick_interval {
            simulation.tick();
            last_tick_time = now;

            if simulation.get_current_tick() % 10 == 0 {
                println!(
                    "Tick {}: {} active agents",
                    simulation.get_current_tick(),
                    simulation.get_entities().active_count
                );
            }
        }

        // Render frame (every ~16ms)
        if now.duration_since(last_frame_time) >= frame_interval {
            let elapsed_since_tick = now.duration_since(last_tick_time).as_secs_f32();
            let alpha = (elapsed_since_tick / (config.tick_interval_ms as f32 / 1000.0)).min(1.0);

            renderer.render(simulation.get_entities(), alpha, simulation.get_current_tick());
            last_frame_time = now;

            // For headless testing, just print framebuffer stats
            let fb = renderer.get_framebuffer();
            let active_pixels = fb.iter().filter(|&&p| p != 0).count();
            if simulation.get_current_tick() % 20 == 0 {
                println!("  Frame rendered: {} active pixels", active_pixels);
            }
        }

        // Stop after 100 ticks for testing
        if simulation.get_current_tick() > 100 {
            running = false;
        }
    }

    println!("\n[COLOR EVOLUTION SIM] Finished after {} ticks", simulation.get_current_tick());
}
