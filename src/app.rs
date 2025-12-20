use crate::accumulator::Accumulator;
use crate::camera::Camera;
use crate::camera_controller::CameraController;
use crate::renderer::Renderer;
use crate::scene::Scene;
use crate::vec3::Vec3;

pub struct App {
    pub renderer: Renderer,
    pub scene: Scene,
    pub pixel_buffer: Vec<u8>,
    pub camera_controller: CameraController,
    pub camera_dirty: bool,
    pub accumulator: Accumulator,
    pub target_samples: usize,
    samples_per_pixel: usize,
    max_depth: i32,
    render_width: usize,
    render_height: usize,
}

impl App {
    pub fn new(width: usize, height: usize) -> Self {
        // Create a default scene similar to materials.rs example
        let mut scene = Self::create_default_scene();
        scene.build_bvh();

        // Initialize camera position and orientation
        let camera_position = Vec3::new(0.0, 1.0, 3.0);
        let camera_target = Vec3::new(0.0, 0.0, -1.0);
        let camera_up = Vec3::new(0.0, 1.0, 0.0);
        let aspect_ratio = width as f64 / height as f64;

        let camera = Camera::new_look_at(
            camera_position,
            camera_target,
            camera_up,
            60.0, // FOV in degrees
            aspect_ratio,
        );

        let renderer = Renderer::new(width, height)
            .with_samples(4) // Lower samples for real-time
            .with_max_depth(16) // Lower depth for real-time
            .with_camera(camera);

        let pixel_buffer = vec![0u8; width * height * 4]; // RGBA

        // Initialize camera controller
        // Start looking slightly down toward the scene
        let camera_controller = CameraController::new(
            camera_position,
            0.0,                         // yaw (looking forward along -Z)
            -15.0_f64.to_radians(),      // pitch (looking slightly down)
        );

        let target_samples = 64;
        let accumulator = Accumulator::new(width, height, target_samples);

        App {
            renderer,
            scene,
            pixel_buffer,
            camera_controller,
            camera_dirty: false,
            accumulator,
            target_samples,
            samples_per_pixel: 4,
            max_depth: 16,
            render_width: width,
            render_height: height,
        }
    }

    fn create_default_scene() -> Scene {
        use crate::material::{Lambertian, Metal};
        use crate::objects::Sphere;
        use crate::vec3::Vec3;
        use std::sync::Arc;

        let mut scene = Scene::new();

        // Ground
        let ground_material = Arc::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5)));
        scene.add(Sphere::new(
            Vec3::new(0.0, -100.5, -1.0),
            100.0,
            ground_material,
        ));

        // Center sphere
        let center_material = Arc::new(Lambertian::new(Vec3::new(0.7, 0.3, 0.3)));
        scene.add(Sphere::new(
            Vec3::new(0.0, 0.0, -1.0),
            0.5,
            center_material,
        ));

        // Left sphere (metal)
        let left_material = Arc::new(Metal::new(Vec3::new(0.8, 0.8, 0.8), 0.3));
        scene.add(Sphere::new(
            Vec3::new(-1.0, 0.0, -1.0),
            0.5,
            left_material,
        ));

        // Right sphere (metal)
        let right_material = Arc::new(Metal::new(Vec3::new(0.8, 0.6, 0.2), 1.0));
        scene.add(Sphere::new(
            Vec3::new(1.0, 0.0, -1.0),
            0.5,
            right_material,
        ));

        scene
    }

    pub fn update(&mut self, delta_time: f64) {
        // Update camera based on input
        let camera = self.renderer.get_camera_mut();
        let moved = self.camera_controller.update_camera(camera, delta_time);

        if moved {
            self.camera_dirty = true;
        }
    }

    pub fn process_keyboard(&mut self, key: winit::keyboard::KeyCode, pressed: bool) {
        self.camera_controller.process_keyboard(key, pressed);
    }

    pub fn process_mouse_motion(&mut self, delta_x: f64, delta_y: f64) {
        self.camera_controller.process_mouse_motion(delta_x, delta_y);
        self.camera_dirty = true;
    }

    pub fn render_frame(&mut self) {
        // Reset accumulator if camera moved
        if self.camera_dirty {
            self.accumulator.reset();
            self.camera_dirty = false;
        }

        // Add a sample if not yet converged
        if !self.accumulator.is_converged() {
            let sample = self.renderer.render_sample(&self.scene);
            self.accumulator.add_sample(sample);
        }

        // Get averaged result and convert to RGBA
        let averaged = self.accumulator.get_averaged();
        self.pixel_buffer = Self::vec3_to_rgba(&averaged);
    }

    fn vec3_to_rgba(pixels: &[Vec3]) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(pixels.len() * 4);
        for pixel in pixels {
            // Clamp to [0, 1] and convert to [0, 255]
            let r = (pixel.x.clamp(0.0, 1.0) * 255.0) as u8;
            let g = (pixel.y.clamp(0.0, 1.0) * 255.0) as u8;
            let b = (pixel.z.clamp(0.0, 1.0) * 255.0) as u8;
            buffer.push(r);
            buffer.push(g);
            buffer.push(b);
            buffer.push(255); // Alpha
        }
        buffer
    }

    pub fn add_random_sphere(&mut self) {
        use crate::material::{Lambertian, Metal};
        use crate::objects::Sphere;
        use rand::Rng;
        use std::sync::Arc;

        let mut rng = rand::rng();
        let center = Vec3::new(
            rng.random_range(-3.0..3.0),
            rng.random_range(0.2..1.0),
            rng.random_range(-3.0..0.0),
        );
        let radius = rng.random_range(0.1..0.5);

        let material: Arc<dyn crate::material::Material> = if rng.random::<f64>() < 0.5 {
            Arc::new(Lambertian::new(Vec3::new(
                rng.random(),
                rng.random(),
                rng.random(),
            )))
        } else {
            Arc::new(Metal::new(
                Vec3::new(rng.random(), rng.random(), rng.random()),
                rng.random_range(0.0..1.0),
            ))
        };

        self.scene.add(Sphere::new(center, radius, material));
        self.scene.build_bvh();
        self.accumulator.reset();
        self.camera_dirty = true;
    }

    pub fn clear_scene(&mut self) {
        self.scene.clear();
        self.accumulator.reset();
        self.camera_dirty = true;
    }

    pub fn reset_scene(&mut self) {
        self.scene = Self::create_default_scene();
        self.scene.build_bvh();
        self.accumulator.reset();
        self.camera_dirty = true;
    }

    pub fn set_quality_fast(&mut self) {
        self.samples_per_pixel = 1;
        self.target_samples = 16;
        self.max_depth = 8;
        self.accumulator.set_target_samples(16);
        self.accumulator.reset();
        self.camera_dirty = true;
    }

    pub fn set_quality_medium(&mut self) {
        self.samples_per_pixel = 4;
        self.target_samples = 64;
        self.max_depth = 16;
        self.accumulator.set_target_samples(64);
        self.accumulator.reset();
        self.camera_dirty = true;
    }

    pub fn set_quality_high(&mut self) {
        self.samples_per_pixel = 8;
        self.target_samples = 256;
        self.max_depth = 50;
        self.accumulator.set_target_samples(256);
        self.accumulator.reset();
        self.camera_dirty = true;
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.render_width = width;
        self.render_height = height;
        self.pixel_buffer.resize(width * height * 4, 0);
        self.accumulator.resize(width, height);

        let aspect_ratio = width as f64 / height as f64;
        let camera = Camera::new_look_at(
            self.camera_controller.position,
            self.camera_controller.position
                + Vec3::new(
                    self.camera_controller.yaw.sin() * self.camera_controller.pitch.cos(),
                    self.camera_controller.pitch.sin(),
                    self.camera_controller.yaw.cos() * self.camera_controller.pitch.cos(),
                ),
            Vec3::new(0.0, 1.0, 0.0),
            60.0,
            aspect_ratio,
        );

        self.renderer = Renderer::new(width, height)
            .with_samples(self.samples_per_pixel)
            .with_max_depth(self.max_depth)
            .with_camera(camera);

        self.camera_dirty = true;
    }
}
