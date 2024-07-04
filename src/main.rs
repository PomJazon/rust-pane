mod test;

use std::sync::Arc;
use futures::executor;
use winit::application::ApplicationHandler;
use winit::event::{WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Arc<Window>,
}

fn main() {
    struct Application {
        state: Option<State>,
    }

    impl ApplicationHandler for Application {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            let window_attributes = Window::default_attributes().with_title("A fantastic window!");
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

            let size = window.inner_size();

            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                ..Default::default()
            });

            let surface = instance.create_surface(window.clone()).unwrap();

            let adapter = executor::block_on(
                instance.request_adapter(
                &wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::default(),
                        compatible_surface: Some(&surface),
                        force_fallback_adapter: false,
                    },
                )
            ).unwrap();

            let (device, queue) = executor::block_on(
                adapter.request_device(
                    &wgpu::DeviceDescriptor {
                        required_features: wgpu::Features::empty(),
                        // WebGL doesn't support all of wgpu's features, so if
                        // we're building for the web, we'll have to disable some.
                        required_limits: if cfg!(target_arch = "wasm32") {
                            wgpu::Limits::downlevel_webgl2_defaults()
                        } else {
                            wgpu::Limits::default()
                        },
                        label: None,
                    },
                    None, // Trace path
                )
            ).unwrap();

            let surface_caps = surface.get_capabilities(&adapter);
            let surface_format = surface_caps.formats.iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0]);
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width,
                height: size.height,
                present_mode: surface_caps.present_modes[0],
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            self.state = Some(State {
                surface,
                device,
                queue,
                config,
                size,
                window,
            });
        }

        fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                },
                WindowEvent::Resized(size) => {
                    if let Some(ref mut state) = self.state {
                        state.size = size;
                        state.config.width = size.width;
                        state.config.height = size.height;
                        state.surface.configure(&state.device, &state.config);
                        state.window.request_redraw();
                    }
                },
                WindowEvent::RedrawRequested if window_id == self.state.as_ref().unwrap().window.id() => {
                    if let Some(ref state) = self.state {
                        let output = state.surface.get_current_texture().unwrap();
                        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                        let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });
                        {
                            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("Render Pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                occlusion_query_set: None,
                                timestamp_writes: None,
                            });
                        }
                        state.queue.submit(std::iter::once(encoder.finish()));
                        output.present();
                    }
                },
                _ => {}
            }
        }
    }

    let event_loop = EventLoop::new().unwrap();

    let mut application = Application { state: None };
    event_loop.run_app(& mut application).unwrap();
}
