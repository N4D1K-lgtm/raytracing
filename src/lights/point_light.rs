use super::{Light, LightSample};
use crate::core::math::Vec2;
use crate::vec3::Vec3;
use std::f64::consts::PI;

/// Point light source (infinitely small, emits equally in all directions)
pub struct PointLight {
    /// Position of the light in world space
    pub position: Vec3,

    /// Intensity (color and brightness)
    pub intensity: Vec3,
}

impl PointLight {
    /// Create a new point light
    pub fn new(position: Vec3, intensity: Vec3) -> Self {
        PointLight {
            position,
            intensity,
        }
    }
}

impl Light for PointLight {
    fn sample_li(&self, point: Vec3, _u: Vec2) -> Option<LightSample> {
        let to_light = self.position - point;
        let distance_squared = to_light.length_squared();

        if distance_squared < 1e-8 {
            return None; // Point is at light position
        }

        let distance = distance_squared.sqrt();
        let wi = to_light / distance;

        // Inverse square falloff
        let radiance = self.intensity / distance_squared;

        Some(LightSample {
            position: self.position,
            normal: Vec3::ZERO, // Point lights don't have a normal
            wi,
            radiance,
            pdf: 1.0, // Delta light has PDF = 1
            distance,
        })
    }

    fn pdf_li(&self, _point: Vec3, _wi: Vec3) -> f64 {
        0.0 // Delta light cannot be importance sampled
    }

    fn power(&self) -> f64 {
        // Power = 4π * intensity (assuming intensity is irradiance)
        4.0 * PI * self.intensity.length()
    }

    fn is_delta(&self) -> bool {
        true // Point lights are delta lights
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_light_sample() {
        let light = PointLight::new(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(10.0, 10.0, 10.0),
        );

        let shading_point = Vec3::ZERO;
        let sample = light.sample_li(shading_point, Vec2::ZERO);

        assert!(sample.is_some());
        let s = sample.unwrap();
        assert!((s.distance - 5.0).abs() < 1e-6);
        assert!((s.wi.y - 1.0).abs() < 1e-6); // Should point up
    }

    #[test]
    fn test_point_light_delta() {
        let light = PointLight::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
        assert!(light.is_delta());
    }
}
