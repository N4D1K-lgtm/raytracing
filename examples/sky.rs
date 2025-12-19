use raytracing::camera::Camera;
use raytracing::image::Image;
use raytracing::ray::Ray;
use raytracing::vec3::Vec3;

/// Simple sky gradient: white at bottom, blue at top
fn ray_color(ray: &Ray) -> Vec3 {
    // Normalize the ray direction (already normalized in Ray::new, but being explicit)
    let unit_direction = ray.direction;

    // Map Y component from [-1, 1] to t in [0, 1]
    // Y = -1 (down) -> t = 0.0 (white)
    // Y = +1 (up)   -> t = 1.0 (blue)
    let t = 0.5 * (unit_direction.y + 1.0);

    // Linear interpolation (lerp): (1-t)*white + t*blue
    let white = Vec3::new(1.0, 1.0, 1.0);
    let blue = Vec3::new(0.5, 0.7, 1.0);

    (1.0 - t) * white + t * blue
}

fn main() {
    // Image settings
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let image_height = (image_width as f64 / aspect_ratio) as usize;

    // Camera
    let camera = Camera::new(aspect_ratio);

    // Render
    let mut img = Image::new(image_width, image_height);

    println!("Rendering {}x{} image...", image_width, image_height);

    for j in 0..image_height {
        if j % 20 == 0 {
            println!("Scanline {}/{}", j, image_height);
        }

        for i in 0..image_width {
            // Convert pixel coordinates to normalized viewport coordinates [0, 1]
            let u = i as f64 / (image_width - 1) as f64;
            let v = j as f64 / (image_height - 1) as f64;

            // Generate ray from camera through this pixel
            let ray = camera.get_ray(u, v);

            // Determine color for this ray
            let color = ray_color(&ray);

            // Note: PPM has (0,0) at top-left, but our viewport has (0,0) at bottom-left
            // We're rendering with j=0 at top, which maps to v=0 at bottom of viewport
            // This creates the correct orientation
            img.set_pixel(i, j, color);
        }
    }

    img.write_ppm("sky.ppm").expect("Failed to write image");
    println!("Sky gradient written to sky.ppm");
}
