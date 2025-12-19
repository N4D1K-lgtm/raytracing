use crate::vec3::Vec3;

use std::fs::File;
use std::io::{self, Write};

pub struct Image {
    width: usize,
    height: usize,
    pixels: Vec<Vec3>,
}

fn f64_to_u8(value: f64) -> u8 {
    let clamped = value.clamp(0.0, 1.0);
    (clamped * 255.0 + 0.5) as u8
}

impl Image {
    pub fn new(width: usize, height: usize) -> Image {
        Image {
            width,
            height,
            pixels: vec![Vec3::ZERO; width * height],
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Vec3) {
        let index = y * self.width + x;
        self.pixels[index] = color;
    }

    pub fn write_ppm(&self, filename: &str) -> io::Result<()> {
        let mut file = File::create(filename)?;

        writeln!(file, "P3")?;
        writeln!(file, "{} {}", self.width, self.height)?;
        writeln!(file, "255")?;

        for pixel in &self.pixels {
            let r = f64_to_u8(pixel.x);
            let g = f64_to_u8(pixel.y);
            let b = f64_to_u8(pixel.z);

            writeln!(file, "{} {} {}", r, g, b)?;
        }

        Ok(())
    }
}

