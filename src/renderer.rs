use crate::entities::Entities;

pub struct Renderer {
    framebuffer: Vec<u32>,
    grid_size: usize,
}

impl Renderer {
    pub fn new(grid_size: usize) -> Self {
        Self {
            framebuffer: vec![0; grid_size * grid_size],
            grid_size,
        }
    }

    pub fn render(&mut self, entities: &Entities, alpha: f32, _tick: u32) {
        // Clear framebuffer
        self.framebuffer.fill(0);

        // Render each active agent
        for i in 0..entities.is_alive.len() {
            if !entities.is_alive[i] {
                continue;
            }

            // Interpolate position
            let prev_x = entities.pos_x[i];
            let prev_y = entities.pos_y[i];
            let target_x = entities.target_x[i] as f32;
            let target_y = entities.target_y[i] as f32;

            let render_x = prev_x + (target_x - prev_x) * alpha;
            let render_y = prev_y + (target_y - prev_y) * alpha;

            let px = render_x.clamp(0.0, (self.grid_size - 1) as f32) as usize;
            let py = render_y.clamp(0.0, (self.grid_size - 1) as f32) as usize;

            // Convert HSL to RGB
            let rgb = self.hsl_to_rgb(
                entities.hue[i],
                entities.saturation[i],
                entities.lightness[i],
            );

            // Plot pixel
            let idx = py * self.grid_size + px;
            if idx < self.framebuffer.len() {
                self.framebuffer[idx] = rgb;
            }
        }
    }

    fn hsl_to_rgb(&self, h: f32, s: f32, l: f32) -> u32 {
        let h = h % 360.0;
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let h_prime = h / 60.0;
        let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());

        let (r_base, g_base, b_base) = match h_prime as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        let m = l - c / 2.0;
        let r = ((r_base + m) * 255.0) as u8;
        let g = ((g_base + m) * 255.0) as u8;
        let b = ((b_base + m) * 255.0) as u8;

        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }

    pub fn get_framebuffer(&self) -> &[u32] {
        &self.framebuffer
    }
}
