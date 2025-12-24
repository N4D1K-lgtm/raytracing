use crate::camera::Camera;
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

    /// Power heuristic for Multiple Importance Sampling (MIS)
    /// Uses β=2 (balance heuristic) which is optimal for most cases
    fn power_heuristic(pdf_a: f64, pdf_b: f64) -> f64 {
        if pdf_a == 0.0 {
            return 0.0;
        }
        if pdf_b == 0.0 {
            return 1.0;
        }
        let a2 = pdf_a * pdf_a;
        let b2 = pdf_b * pdf_b;
        a2 / (a2 + b2)
    }

    fn ray_color(&self, ray: &Ray, world: &Scene, depth: i32) -> Vec3 {
        // Russian Roulette path termination for unbiased rendering
        const MIN_DEPTH: i32 = 3;  // Always trace first 3 bounces
        const RR_SURVIVAL_PROB: f64 = 0.85;  // 85% survival probability after min depth

        // If we've exceeded the ray bounce limit, no more light is gathered
        if depth <= 0 {
            return Vec3::ZERO;
        }

        // Russian Roulette: probabilistically terminate paths after min depth
        // depth counts down: max_depth -> 0, so (max_depth - depth) is number of bounces taken
        let bounces_taken = self.max_depth - depth;

        if bounces_taken >= MIN_DEPTH {
            let mut rng = rand::rng();
            if rng.random::<f64>() > RR_SURVIVAL_PROB {
                return Vec3::ZERO; // Terminate path
            }
            // Note: We don't divide by survival prob here - we'll do it in the contribution below
        }

        // Check if ray hits anything using new scene graph API
        if let Some(isect) = world.intersect(ray, 0.001, f64::INFINITY) {
            // Add emitted light from surface
            let mut color = isect.material.emitted(&isect);

            // Russian Roulette compensation: divide by survival probability to remain unbiased
            let rr_compensation = if bounces_taken >= MIN_DEPTH {
                1.0 / RR_SURVIVAL_PROB
            } else {
                1.0
            };

            // Compute scattering at intersection point
            let bxdf = isect.material.compute_scattering_functions(&isect);

            // Transform ray direction to local shading space
            // Local space has normal pointing along +Z
            let wo_world = -ray.direction; // Direction towards viewer
            let wo_local = Self::world_to_local(wo_world, &isect);

            let mut rng = rand::rng();

            // === DIRECT LIGHTING: Sample all lights ===
            for light in world.lights() {
                // Sample a point on the light
                let light_u = crate::core::math::Vec2::new(rng.random::<f64>(), rng.random::<f64>());

                if let Some(light_sample) = light.sample_li(isect.point, light_u) {
                    // Transform light direction to local space
                    let wi_local = Self::world_to_local(light_sample.wi, &isect);

                    // Check if light is in correct hemisphere
                    if wi_local.z <= 0.0 {
                        continue;
                    }

                    // Cast shadow ray to check visibility
                    let shadow_ray = Ray::new(isect.point, light_sample.wi);
                    let blocked = world.intersect_p(&shadow_ray, 0.001, light_sample.distance - 0.001);

                    if !blocked {
                        // Evaluate BSDF for this direction
                        let f = bxdf.f(wo_local, wi_local);
                        let cos_theta = wi_local.z.abs();

                        // Multiple Importance Sampling: weight light sample by BSDF PDF
                        let mis_weight = if light.is_delta() {
                            // Delta lights can't be importance sampled by BSDF, so weight = 1
                            1.0
                        } else {
                            // Evaluate BSDF PDF for this direction and compute balance heuristic
                            let bsdf_pdf = bxdf.pdf(wo_local, wi_local);
                            Self::power_heuristic(light_sample.pdf, bsdf_pdf)
                        };

                        // Add direct lighting contribution with MIS weight
                        if light_sample.pdf > 0.0 {
                            color += mis_weight * f * light_sample.radiance * (cos_theta / light_sample.pdf);
                        }
                    }
                }
            }

            // === INDIRECT LIGHTING: Sample BSDF for next bounce ===
            let u = crate::core::math::Vec2::new(rng.random::<f64>(), rng.random::<f64>());

            if let Some(sample) = bxdf.sample_f(wo_local, u) {
                // Transform sampled direction back to world space
                let wi_world = Self::local_to_world(sample.wi, &isect);

                // Create scattered ray
                let scattered = Ray::new(isect.point, wi_world);

                // Compute rendering equation contribution for indirect lighting
                let cos_theta = sample.wi.z.abs();

                // Recursively trace scattered ray
                let incoming_radiance = self.ray_color(&scattered, world, depth - 1);

                // Multiple Importance Sampling: weight BSDF sample by light PDF if we hit a light
                let mut mis_weight = 1.0;

                // Check if we hit an emissive surface (light)
                if incoming_radiance.length_squared() > 0.0 {
                    if let Some(hit_isect) = world.intersect(&scattered, 0.001, f64::INFINITY) {
                        let emitted = hit_isect.material.emitted(&hit_isect);

                        // If we hit an emissive surface, evaluate light sampling PDF
                        if emitted.length_squared() > 0.0 {
                            // Compute average PDF of sampling this direction via lights
                            let mut light_pdf = 0.0;
                            let mut num_lights = 0;

                            for light in world.lights() {
                                // Skip delta lights - they can't be hit by random rays
                                if !light.is_delta() {
                                    light_pdf += light.pdf_li(isect.point, wi_world);
                                    num_lights += 1;
                                }
                            }

                            if num_lights > 0 {
                                light_pdf /= num_lights as f64;
                                // Apply MIS weight (BSDF sampling strategy)
                                mis_weight = Self::power_heuristic(sample.pdf, light_pdf);
                            }
                        }
                    }
                }

                // Add indirect light with MIS weight (f * L_i * cos_theta / pdf)
                if sample.pdf > 0.0 {
                    color += mis_weight * sample.f * incoming_radiance * (cos_theta / sample.pdf);
                }
            }

            // Apply Russian Roulette compensation to final color
            return color * rr_compensation;
        }

        // Sky gradient background (light source)
        let unit_direction = ray.direction;
        let t = 0.5 * (unit_direction.y + 1.0);
        (1.0 - t) * Vec3::new(1.0, 1.0, 1.0) + t * Vec3::new(0.5, 0.7, 1.0)
    }

    /// Transform a world-space direction to local shading space
    /// Local space has: normal = +Z, tangent = +X, bitangent = +Y
    fn world_to_local(world: Vec3, isect: &crate::core::intersection::Intersection) -> Vec3 {
        let tangent = isect.tangent();
        let bitangent = isect.bitangent();
        let normal = isect.shading_normal();

        Vec3::new(
            world.dot(tangent),
            world.dot(bitangent),
            world.dot(normal),
        )
    }

    /// Transform a local shading space direction to world space
    fn local_to_world(local: Vec3, isect: &crate::core::intersection::Intersection) -> Vec3 {
        let tangent = isect.tangent();
        let bitangent = isect.bitangent();
        let normal = isect.shading_normal();

        tangent * local.x + bitangent * local.y + normal * local.z
    }
}
