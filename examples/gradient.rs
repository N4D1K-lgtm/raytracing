use raytracing::image::Image;
use raytracing::vec3::Vec3;

fn main() {
    let width = 256;
    let height = 256;
    let mut img = Image::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let r = x as f64 / (width - 1) as f64;
            let g = y as f64 / (height - 1) as f64;
            let b = 0.0;

            let color = Vec3::new(r, g, b);
            img.set_pixel(x, y, color);
        }
    }

    img.write_ppm("gradient.ppm").expect("Failed to write image");
    println!("Gradient image written to gradient.ppm");
}
