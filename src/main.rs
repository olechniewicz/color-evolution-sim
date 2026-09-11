use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop},
    window::WindowBuilder,
};
use wgpu::*;
use std::time::{Instant, Duration};

mod config;
mod entities;
mod prng;
mod renderer;
mod simulation;
mod graphics;

use config::Config;
use renderer::Renderer;
use simulation::Simulation;
use graphics::RenderPipeline;

pub struct GraphicsState {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: Option<RenderPipeline>,
    texture: Option<Texture>,
}

impl GraphicsState {
    async fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });
        
        let surface = unsafe {
            instance.create_surface_unsafe(
                SurfaceTargetUnsafe::from_window(window).unwrap()
            )
        }.unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: Features::empty(),
                    required_limits: Limits::downlevel_webgl2_defaults(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline: None,
            texture: None,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

fn main() {
    env_logger::init();

    let config = Config::default();
    let mut simulation = Simulation::new(config.clone());
    let mut sim_renderer = Renderer::new(config.grid_size);

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Color Evolution Sim - 1000x1000")
        .with_inner_size(winit::dpi::LogicalSize::new(1000.0, 1000.0))
        .build(&event_loop)
        .unwrap();

    let window = std::sync::Arc::new(window);
    let tick_interval = Duration::from_millis(config.tick_interval_ms);
    let mut last_tick_time = Instant::now();
    let mut last_frame_time = Instant::now();
    let mut frame_count = 0u32;
    let mut fps_timer = Instant::now();

    let config_clone = config.clone();
    let window_clone = window.clone();

    let _result = pollster::block_on(async {
        let mut graphics = GraphicsState::new(&window).await;

        // Create texture for simulation framebuffer
        let texture = graphics.device.create_texture(&TextureDescriptor {
            label: Some("sim_framebuffer"),
            size: Extent3d {
                width: config_clone.grid_size as u32,
                height: config_clone.grid_size as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        // Create render pipeline
        let render_pipeline = RenderPipeline::new(&graphics.device, &graphics.queue, graphics.config.format, &texture);
        graphics.render_pipeline = Some(render_pipeline);
        graphics.texture = Some(texture);

        event_loop.run(move |event, target| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => target.exit(),
                Event::WindowEvent {
                    event: WindowEvent::Resized(physical_size),
                    ..
                } => graphics.resize(physical_size),
                Event::AboutToWait => {
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

                    // Render frame (60 FPS)
                    if now.duration_since(last_frame_time) >= Duration::from_millis(16) {
                        let elapsed_since_tick = now.duration_since(last_tick_time).as_secs_f32();
                        let alpha = (elapsed_since_tick / (config_clone.tick_interval_ms as f32 / 1000.0)).min(1.0);

                        sim_renderer.render(simulation.get_entities(), alpha, simulation.get_current_tick());

                        // Upload framebuffer to GPU
                        if let Some(ref texture) = graphics.texture {
                            let framebuffer = sim_renderer.get_framebuffer();
                            let mut rgba_data = Vec::with_capacity(framebuffer.len() * 4);
                            for &rgb in framebuffer {
                                let r = (rgb >> 16) as u8;
                                let g = (rgb >> 8) as u8;
                                let b = rgb as u8;
                                rgba_data.push(r);
                                rgba_data.push(g);
                                rgba_data.push(b);
                                rgba_data.push(255);
                            }

                            graphics.queue.write_texture(
                                ImageCopyTexture {
                                    texture: &texture,
                                    mip_level: 0,
                                    origin: Origin3d::ZERO,
                                    aspect: TextureAspect::All,
                                },
                                &rgba_data,
                                ImageDataLayout {
                                    offset: 0,
                                    bytes_per_row: Some(config_clone.grid_size as u32 * 4),
                                    rows_per_image: Some(config_clone.grid_size as u32),
                                },
                                Extent3d {
                                    width: config_clone.grid_size as u32,
                                    height: config_clone.grid_size as u32,
                                    depth_or_array_layers: 1,
                                },
                            );
                        }

                        // Render to screen
                        match graphics.surface.get_current_texture() {
                            Ok(output) => {
                                let view = output.texture.create_view(&TextureViewDescriptor::default());
                                let mut encoder = graphics.device.create_command_encoder(
                                    &CommandEncoderDescriptor { label: Some("render_encoder") },
                                );

                                {
                                    let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                                        label: Some("render_pass"),
                                        color_attachments: &[Some(RenderPassColorAttachment {
                                            view: &view,
                                            resolve_target: None,
                                            ops: Operations {
                                                load: LoadOp::Clear(Color::BLACK),
                                                store: StoreOp::Store,
                                            },
                                        })],
                                        depth_stencil_attachment: None,
                                        occlusion_query_set: None,
                                        timestamp_writes: None,
                                    });

                                    if let Some(ref pipeline) = graphics.render_pipeline {
                                        render_pass.set_pipeline(&pipeline.pipeline);
                                        render_pass.set_bind_group(0, &pipeline.bind_group, &[]);
                                        render_pass.draw(0..6, 0..1);
                                    }
                                }

                                graphics.queue.submit(std::iter::once(encoder.finish()));
                                output.present();
                            }
                            Err(_) => {}
                        }

                        last_frame_time = now;
                        frame_count += 1;

                        if fps_timer.elapsed() >= Duration::from_secs(1) {
                            println!("FPS: {}", frame_count);
                            frame_count = 0;
                            fps_timer = Instant::now();
                        }

                        window_clone.request_redraw();
                    }
                }
                _ => {}
            }
        });
    });
}
