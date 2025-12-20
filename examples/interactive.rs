use raytracing::app::App;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

const WIDTH: u32 = 540;
const HEIGHT: u32 = 360;

struct AppState {
    window: Option<Arc<Window>>,
    pixels: Option<pixels::Pixels<'static>>,
    app: App,
    last_frame: Instant,
    cursor_grabbed: bool,
}

impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Interactive Raytracer")
                .with_inner_size(winit::dpi::LogicalSize::new(WIDTH, HEIGHT));

            let window = event_loop
                .create_window(window_attributes)
                .expect("Failed to create window");

            let arc_window = Arc::new(window);
            self.window = Some(arc_window);
        }

        if self.pixels.is_some() {
            return;
        }

        if let Some(window) = &self.window {
            let window_size = window.inner_size();
            // Leak the Arc to get a 'static reference for pixels
            let window_ref: &'static Window = Box::leak(Box::new(Arc::clone(window)));
            let surface_texture =
                pixels::SurfaceTexture::new(window_size.width, window_size.height, window_ref);
            let pixels = pixels::PixelsBuilder::new(WIDTH, HEIGHT, surface_texture)
                .build()
                .expect("Failed to create pixels buffer");
            self.pixels = Some(pixels);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Close requested, exiting...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Calculate delta time
                let now = Instant::now();
                let delta_time = now.duration_since(self.last_frame).as_secs_f64();
                self.last_frame = now;

                // Update app state
                self.app.update(delta_time);

                // Render frame
                self.app.render_frame();

                // Copy to pixels buffer
                if let Some(pixels) = &mut self.pixels {
                    let frame = pixels.frame_mut();
                    frame.copy_from_slice(&self.app.pixel_buffer);

                    if let Err(e) = pixels.render() {
                        eprintln!("pixels.render() failed: {}", e);
                        event_loop.exit();
                    }
                }

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }

                // Update window title with FPS and sample count
                let fps = 1.0 / delta_time;
                let samples = self.app.accumulator.sample_count();
                let target = self.app.accumulator.target_samples();
                if let Some(window) = &self.window {
                    window.set_title(&format!(
                        "Interactive Raytracer - {:.1} FPS | Samples: {}/{}",
                        fps, samples, target
                    ));
                }
            }
            WindowEvent::Resized(new_size) => {
                if let Some(pixels) = &mut self.pixels {
                    if let Err(e) = pixels.resize_surface(new_size.width, new_size.height) {
                        eprintln!("pixels.resize_surface() failed: {}", e);
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                use winit::keyboard::{KeyCode, PhysicalKey};

                if let PhysicalKey::Code(key_code) = event.physical_key {
                    if !event.state.is_pressed() {
                        // Pass key releases to camera controller
                        self.app.process_keyboard(key_code, false);
                        return;
                    }

                    // Handle Escape key for cursor release
                    if key_code == KeyCode::Escape {
                        self.cursor_grabbed = !self.cursor_grabbed;
                        if let Some(window) = &self.window {
                            window
                                .set_cursor_grab(if self.cursor_grabbed {
                                    winit::window::CursorGrabMode::Confined
                                } else {
                                    winit::window::CursorGrabMode::None
                                })
                                .ok();
                            window.set_cursor_visible(!self.cursor_grabbed);
                        }
                        return;
                    }

                    // Scene editing
                    match key_code {
                        KeyCode::KeyN => {
                            self.app.add_random_sphere();
                            println!("Added sphere (total: {})", self.app.scene.len());
                        }
                        KeyCode::KeyC => {
                            self.app.clear_scene();
                            println!("Cleared scene");
                        }
                        KeyCode::KeyR => {
                            self.app.reset_scene();
                            println!("Reset scene to default");
                        }
                        // Quality presets
                        KeyCode::F1 => {
                            self.app.set_quality_fast();
                            println!("Quality: Fast (1 SPP, 16 target)");
                        }
                        KeyCode::F2 => {
                            self.app.set_quality_medium();
                            println!("Quality: Medium (4 SPP, 64 target)");
                        }
                        KeyCode::F3 => {
                            self.app.set_quality_high();
                            println!("Quality: High (8 SPP, 256 target)");
                        }
                        KeyCode::KeyH => {
                            println!("\n=== Controls ===");
                            println!("ESC        - Toggle mouse capture");
                            println!("WASD       - Move camera");
                            println!("Space      - Move up");
                            println!("Left Shift - Move down");
                            println!("Mouse      - Look around (when captured)");
                            println!("\n=== Scene ===");
                            println!("N          - Add random sphere");
                            println!("C          - Clear scene");
                            println!("R          - Reset to default scene");
                            println!("\n=== Quality ===");
                            println!("F1         - Fast (1 SPP, 16 target)");
                            println!("F2         - Medium (4 SPP, 64 target)");
                            println!("F3         - High (8 SPP, 256 target)");
                            println!("H          - Show this help\n");
                        }
                        _ => {
                            // Pass to camera controller
                            self.app.process_keyboard(key_code, true);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        use winit::event::DeviceEvent;

        // Only process mouse motion when cursor is grabbed
        if self.cursor_grabbed {
            if let DeviceEvent::MouseMotion { delta } = event {
                self.app.process_mouse_motion(delta.0, delta.1);
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    println!("=== Real-Time Interactive Raytracer ===");
    println!("Window size: {}x{}", WIDTH, HEIGHT);
    println!("\n=== Controls ===");
    println!("ESC        - Toggle mouse capture");
    println!("WASD       - Move camera");
    println!("Space      - Move up");
    println!("Left Shift - Move down");
    println!("Mouse      - Look around (when captured)");
    println!("\n=== Scene ===");
    println!("N          - Add random sphere");
    println!("C          - Clear scene");
    println!("R          - Reset to default scene");
    println!("\n=== Quality ===");
    println!("F1         - Fast (1 SPP, 16 target)");
    println!("F2         - Medium (4 SPP, 64 target)");
    println!("F3         - High (8 SPP, 256 target)");
    println!("H          - Show help");
    println!();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app_state = AppState {
        window: None,
        pixels: None,
        app: App::new(WIDTH as usize, HEIGHT as usize),
        last_frame: Instant::now(),
        cursor_grabbed: false,
    };

    println!("Starting event loop...");
    println!("Press ESC to capture mouse for camera control");
    event_loop
        .run_app(&mut app_state)
        .expect("Event loop failed");
}
