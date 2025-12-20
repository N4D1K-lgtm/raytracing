use crate::vec3::Vec3;

pub struct Accumulator {
    buffer: Vec<Vec3>,
    sample_count: usize,
    width: usize,
    height: usize,
    target_samples: usize,
}

impl Accumulator {
    pub fn new(width: usize, height: usize, target_samples: usize) -> Self {
        Accumulator {
            buffer: vec![Vec3::ZERO; width * height],
            sample_count: 0,
            width,
            height,
            target_samples,
        }
    }

    pub fn add_sample(&mut self, new_frame: Vec<Vec3>) {
        assert_eq!(
            new_frame.len(),
            self.width * self.height,
            "Frame size mismatch"
        );

        for (accumulated, sample) in self.buffer.iter_mut().zip(new_frame.iter()) {
            *accumulated += *sample;
        }

        self.sample_count += 1;
    }

    pub fn get_averaged(&self) -> Vec<Vec3> {
        if self.sample_count == 0 {
            return self.buffer.clone();
        }

        let scale = 1.0 / self.sample_count as f64;
        self.buffer
            .iter()
            .map(|color| {
                // Apply gamma correction (gamma 2)
                let r = (color.x * scale).sqrt();
                let g = (color.y * scale).sqrt();
                let b = (color.z * scale).sqrt();
                Vec3::new(r, g, b)
            })
            .collect()
    }

    pub fn reset(&mut self) {
        self.buffer.fill(Vec3::ZERO);
        self.sample_count = 0;
    }

    pub fn is_converged(&self) -> bool {
        self.sample_count >= self.target_samples
    }

    pub fn sample_count(&self) -> usize {
        self.sample_count
    }

    pub fn target_samples(&self) -> usize {
        self.target_samples
    }

    pub fn set_target_samples(&mut self, target: usize) {
        self.target_samples = target;
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.buffer.resize(width * height, Vec3::ZERO);
        self.sample_count = 0;
    }
}
