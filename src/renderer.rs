use crate::camera::Camera;
use crate::hit::Hittable;
use crate::image::Image;
use crate::ray::Ray;
use crate::scene::Scene;
use crate::vec3::Vec3;
use rand::Rng;

pub struct Renderer {
    camera: Camera,
    width: usize,
    height: usize,
    samples_per_pixel: usize,
    max_depth: i32,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        let aspect_ratio = width as f64 / height as f64;
        Renderer {
            camera: Camera::new(aspect_ratio),
            width,
            height,
            samples_per_pixel: 100,
            max_depth: 50,
        }
    }

    pub fn with_samples(mut self, samples: usize) -> Self {
        self.samples_per_pixel = samples;
        self
    }

    pub fn with_max_depth(mut self, depth: i32) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn render(&self, world: &Scene, filename: &str) {
        let mut img = Image::new(self.width, self.height);
        let mut rng = rand::rng();

        println!(
            "Rendering {}x{} with {} samples per pixel...",
            self.width, self.height, self.samples_per_pixel
        );

        for j in 0..self.height {
            if j % 20 == 0 {
                println!("Scanline {}/{}", j, self.height);
            }

            for i in 0..self.width {
                let mut pixel_color = Vec3::ZERO;

                // Anti-aliasing: multiple samples per pixel
                for _ in 0..self.samples_per_pixel {
                    let u = (i as f64 + rng.random::<f64>()) / (self.width - 1) as f64;
                    // Flip v: PPM has (0,0) at top-left, viewport has (0,0) at bottom-left
                    let v = ((self.height - 1 - j) as f64 + rng.random::<f64>()) / (self.height - 1) as f64;

                    let ray = self.camera.get_ray(u, v);
                    pixel_color += self.ray_color(&ray, world, self.max_depth);
                }

                // Average the samples and apply gamma correction
                let scale = 1.0 / self.samples_per_pixel as f64;
                let r = (pixel_color.x * scale).sqrt(); // Gamma 2 correction
                let g = (pixel_color.y * scale).sqrt();
                let b = (pixel_color.z * scale).sqrt();

                img.set_pixel(i, j, Vec3::new(r, g, b));
            }
        }

        img.write_ppm(filename).expect("Failed to write image");
        println!("Rendered to {}", filename);
    }

    fn ray_color(&self, ray: &Ray, world: &Scene, depth: i32) -> Vec3 {
        // If we've exceeded the ray bounce limit, no more light is gathered
        if depth <= 0 {
            return Vec3::ZERO;
        }

        // Check if ray hits anything
        if let Some(hit) = world.hit(ray, 0.001, f64::INFINITY) {
            // Try to scatter the ray off the material
            if let Some((attenuation, scattered)) = hit.material.scatter(ray, &hit) {
                // Recursively trace the scattered ray
                return attenuation * self.ray_color(&scattered, world, depth - 1);
            }
            // Ray was absorbed
            return Vec3::ZERO;
        }

        // Sky gradient background (light source)
        let unit_direction = ray.direction;
        let t = 0.5 * (unit_direction.y + 1.0);
        (1.0 - t) * Vec3::new(1.0, 1.0, 1.0) + t * Vec3::new(0.5, 0.7, 1.0)
    }
}
