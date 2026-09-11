# Color Evolution Sim

Agent-Based Model (ABM) simulation showcasing evolutionary color dynamics on a 1000×1000 grid. Features spatial partitioning via Radix Sort, decoupled logic/render loops, and deterministic PRNG.

## Features

- **Spatial Sorting**: Radix Sort for O(n) collision detection instead of quadtrees
- **Decoupled Ticks**: Logic runs at 500ms intervals, rendering at 60 FPS with smooth interpolation
- **Deterministic**: Xoshiro256** PRNG ensures reproducible simulations
- **Genetic Algorithm**: Agents reproduce via HSL color blending with angular hue interpolation
- **Toroidal World**: Agents wrap at grid edges for seamless boundaries
- **Stagnation Detection**: Auto-reset when population stabilizes

## Architecture

### Data Layout: Structure of Arrays (SoA)

All agent properties stored in parallel vectors for cache efficiency:
- Position: `pos_x`, `pos_y` (rendering), `target_x`, `target_y` (logic)
- Genetics: `hue`, `saturation`, `lightness` (HSL color space)
- Lifecycle: `birth_tick`, `is_alive`

### Main Loop

1. **Aging & Death** — Remove agents exceeding lifespan
2. **Movement** — Random walk (8-directional)
3. **Spatial Sorting** — Radix Sort by grid index
4. **Collisions & Births** — Detect colocated pairs, spawn offspring
5. **Stagnation Check** — Reset if population stabilizes

Render thread runs independently, interpolating agent positions between ticks.

## Configuration

Edit `src/config.rs`:

```rust
pub struct Config {
    pub grid_size: usize,              // 1000
    pub initial_population: usize,     // 10,000
    pub lifespan_ticks: u32,           // 500
    pub tick_interval_ms: u64,         // 500
    pub seed: u64,                     // 12345 (determinism)
    pub reproduction_radius: usize,    // 2 pixels
    pub stagnation_threshold: usize,   // 1% of initial pop
    pub stagnation_check_ticks: u32,   // 20
}
```

## Building & Running

```bash
cargo build --release
cargo run --release
```

The simulation runs in headless mode, printing tick statistics. Each tick is 500ms; rendering happens at 60 FPS internally.

## Performance Tuning

- **Radix Sort**: O(n) instead of O(n log n) for spatial partitioning
- **Cache Locality**: SoA layout ensures tight packing of hot data
- **Deterministic PRNG**: No locks, reproducible results
- **Lazy Deallocation**: Dead agents reused without heap churn

## Future: Graphical Rendering

Replace headless loop with wgpu texture streaming for real-time visualization at 60 FPS with physics-based smooth movement.

## License

MIT
