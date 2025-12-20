use crate::camera::Camera;
use crate::hit::Hittable;
use crate::image::Image;
use crate::ray::Ray;
use crate::scene::Scene;
use crate::vec3::Vec3;
use rand::Rng;
use rayon::prelude::*;

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

    pub fn with_camera(mut self, camera: Camera) -> Self {
        self.camera = camera;
        self
    }

    pub fn get_camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn render(&self, world: &Scene, filename: &str) {
        let mut img = Image::new(self.width, self.height);

        println!(
            "Rendering {}x{} with {} samples per pixel...",
            self.width, self.height, self.samples_per_pixel
        );

        // Parallel rendering with rayon
        let pixels: Vec<Vec3> = (0..self.height)
            .into_par_iter()
            .flat_map(|j| {
                if j % 20 == 0 {
                    println!("Scanline {}/{}", j, self.height);
                }

                let mut rng = rand::rng();

                (0..self.width)
                    .map(|i| {
                        let mut pixel_color = Vec3::ZERO;

                        // Anti-aliasing: multiple samples per pixel
                        for _ in 0..self.samples_per_pixel {
                            let u = (i as f64 + rng.random::<f64>()) / (self.width - 1) as f64;
                            // Flip v: PPM has (0,0) at top-left, viewport has (0,0) at bottom-left
                            let v = ((self.height - 1 - j) as f64 + rng.random::<f64>())
                                / (self.height - 1) as f64;

                            let ray = self.camera.get_ray(u, v);
                            pixel_color += self.ray_color(&ray, world, self.max_depth);
                        }

                        // Average the samples and apply gamma correction
                        let scale = 1.0 / self.samples_per_pixel as f64;
                        let r = (pixel_color.x * scale).sqrt(); // Gamma 2 correction
                        let g = (pixel_color.y * scale).sqrt();
                        let b = (pixel_color.z * scale).sqrt();

                        Vec3::new(r, g, b)
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        // Write pixels to image
        for j in 0..self.height {
            for i in 0..self.width {
                img.set_pixel(i, j, pixels[j * self.width + i]);
            }
        }

        img.write_ppm(filename).expect("Failed to write image");
        println!("Rendered to {}", filename);
    }

    pub fn render_to_buffer(&self, world: &Scene) -> Vec<u8> {
        // Parallel rendering with rayon
        let pixels: Vec<Vec3> = (0..self.height)
            .into_par_iter()
            .flat_map(|j| {
                let mut rng = rand::rng();

                (0..self.width)
                    .map(|i| {
                        let mut pixel_color = Vec3::ZERO;

                        // Anti-aliasing: multiple samples per pixel
                        for _ in 0..self.samples_per_pixel {
                            let u = (i as f64 + rng.random::<f64>()) / (self.width - 1) as f64;
                            // Flip v: PPM has (0,0) at top-left, viewport has (0,0) at bottom-left
                            let v = ((self.height - 1 - j) as f64 + rng.random::<f64>())
                                / (self.height - 1) as f64;

                            let ray = self.camera.get_ray(u, v);
                            pixel_color += self.ray_color(&ray, world, self.max_depth);
                        }

                        // Average the samples and apply gamma correction
                        let scale = 1.0 / self.samples_per_pixel as f64;
                        let r = (pixel_color.x * scale).sqrt(); // Gamma 2 correction
                        let g = (pixel_color.y * scale).sqrt();
                        let b = (pixel_color.z * scale).sqrt();

                        Vec3::new(r, g, b)
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        Self::vec3_to_rgba(&pixels)
    }

    pub fn render_sample(&self, world: &Scene) -> Vec<Vec3> {
        // Render single sample per pixel (for progressive rendering)
        (0..self.height)
            .into_par_iter()
            .flat_map(|j| {
                let mut rng = rand::rng();

                (0..self.width)
                    .map(|i| {
                        let u = (i as f64 + rng.random::<f64>()) / (self.width - 1) as f64;
                        let v = ((self.height - 1 - j) as f64 + rng.random::<f64>())
                            / (self.height - 1) as f64;

                        let ray = self.camera.get_ray(u, v);
                        self.ray_color(&ray, world, self.max_depth)
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
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
