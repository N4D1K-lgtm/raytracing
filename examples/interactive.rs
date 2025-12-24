use pixels::{Pixels, SurfaceTexture};
use raytracing::app::App;
use std::sync::Arc;
use std::time::Instant;
use winit::{
    dpi::LogicalSize,
    event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
};

fn main() {
    println!("=== Interactive Raytracer ===\n");
    println!("Controls:");
    println!("  WASD       - Move camera");
    println!("  Space      - Move up");
    println!("  Shift      - Move down");
    println!("  Mouse      - Look around");
    println!("  R          - Add random sphere");
    println!("  C          - Clear scene");
    println!("  T          - Reset to default scene");
    println!("  1/2/3      - Quality: Fast/Medium/High");
    println!("  ESC        - Exit\n");

    // Window dimensions
    let width = 960;
    let height = 540;

    // Create event loop and window
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new({
        let size = LogicalSize::new(width as f64, height as f64);
        let window_attributes = winit::window::Window::default_attributes()
            .with_title("Interactive Raytracer - Path Tracing with MIS & Russian Roulette")
            .with_inner_size(size)
            .with_min_inner_size(size);
        event_loop.create_window(window_attributes).unwrap()
    });

    // Hide cursor and grab it for FPS-style controls
    window.set_cursor_visible(false);
    let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Confined);

    // Create pixel buffer for rendering
    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
    let mut pixels = Pixels::new(width as u32, height as u32, surface_texture).unwrap();

    // Create application
    let mut app = App::new(width, height);

    // Set initial quality to fast for real-time
    app.set_quality_fast();

    println!("Scene initialized. Starting render loop...\n");
    println!("Samples: 0/{}\n", app.target_samples);

    let mut last_frame_time = Instant::now();
    let mut last_sample_print = Instant::now();

    let window_clone = window.clone();
    let _ = event_loop.run(move |event, elwt| {
            let window = &window_clone;
            elwt.set_control_flow(ControlFlow::Poll);

            match event {
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    app.process_mouse_motion(delta.0, delta.1);
                }

                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }

                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(key),
                                state,
                                ..
                            },
                        ..
                    } => {
                        let pressed = state == ElementState::Pressed;

                        match key {
                            KeyCode::Escape if pressed => {
                                elwt.exit();
                            }
                            KeyCode::KeyR if pressed => {
                                println!("Adding random sphere...");
                                app.add_random_sphere();
                            }
                            KeyCode::KeyC if pressed => {
                                println!("Clearing scene...");
                                app.clear_scene();
                            }
                            KeyCode::KeyT if pressed => {
                                println!("Resetting to default scene...");
                                app.reset_scene();
                            }
                            KeyCode::Digit1 if pressed => {
                                println!("Quality: Fast (16 samples)");
                                app.set_quality_fast();
                            }
                            KeyCode::Digit2 if pressed => {
                                println!("Quality: Medium (64 samples)");
                                app.set_quality_medium();
                            }
                            KeyCode::Digit3 if pressed => {
                                println!("Quality: High (256 samples)");
                                app.set_quality_high();
                            }
                            _ => {
                                app.process_keyboard(key, pressed);
                            }
                        }
                    }

                    WindowEvent::Resized(new_size) => {
                        let _ = pixels.resize_surface(new_size.width, new_size.height);
                        // Note: We're not resizing the render buffer to maintain consistent performance
                        // The image will be stretched to fit the window
                    }

                    WindowEvent::RedrawRequested => {
                        // Calculate delta time
                        let now = Instant::now();
                        let delta_time = now.duration_since(last_frame_time).as_secs_f64();
                        last_frame_time = now;

                        // Update camera
                        app.update(delta_time);

                        // Render a frame (adds one sample if not converged)
                        app.render_frame();

                        // Copy to pixel buffer
                        let frame = pixels.frame_mut();
                        frame.copy_from_slice(&app.pixel_buffer);

                        // Display to window
                        if let Err(err) = pixels.render() {
                            eprintln!("pixels.render() failed: {}", err);
                            elwt.exit();
                            return;
                        }

                        // Print sample count periodically
                        if now.duration_since(last_sample_print).as_secs() >= 1 {
                            let sample_count = app.accumulator.sample_count();
                            let target = app.accumulator.target_samples();
                            if sample_count < target {
                                println!("Samples: {}/{}", sample_count, target);
                            } else {
                                println!("Converged at {} samples", sample_count);
                            }
                            last_sample_print = now;
                        }
                    }

                    _ => {}
                },

                Event::AboutToWait => {
                    window.request_redraw();
                }

                _ => {}
            }
        });
}
