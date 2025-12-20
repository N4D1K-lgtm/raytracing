use crate::core::math::Vec2;
use crate::geometry::Primitive;
use crate::vec3::Vec3;
use std::sync::Arc;

use super::light::{Light, LightSample};

/// Area light - emits light from a geometric surface
pub struct AreaLight {
    /// The primitive shape that emits light
    primitive: Arc<dyn Primitive>,

    /// Emitted radiance (color * intensity)
    emission: Vec3,

    /// Surface area (cached for PDF calculations)
    area: f64,

    /// Two-sided emission (emit from both sides of surface)
    two_sided: bool,
}

impl AreaLight {
    pub fn new(primitive: Arc<dyn Primitive>, emission: Vec3) -> Self {
        let area = primitive.surface_area();
        AreaLight {
            primitive,
            emission,
            area,
            two_sided: false,
        }
    }

    pub fn with_two_sided(mut self, two_sided: bool) -> Self {
        self.two_sided = two_sided;
        self
    }

    /// Get the emitted radiance for a given outgoing direction
    fn le(&self, normal: Vec3, wo: Vec3) -> Vec3 {
        if self.two_sided || wo.dot(normal) > 0.0 {
            self.emission
        } else {
            Vec3::ZERO
        }
    }
}

impl Light for AreaLight {
    fn sample_li(&self, point: Vec3, u: Vec2) -> Option<LightSample> {
        // Sample a point on the light's surface
        let (light_point, light_normal, shape_pdf) = self.primitive.sample(u);

        // Vector from shading point to light sample
        let to_light = light_point - point;
        let distance = to_light.length();

        if distance < 1e-6 || shape_pdf == 0.0 {
            return None;
        }

        let wi = to_light / distance;

        // Check if light is visible from the shading point
        // (light normal points away from surface)
        let cos_theta = (-wi).dot(light_normal);
        if !self.two_sided && cos_theta <= 0.0 {
            return None; // Back-facing
        }

        // Compute emitted radiance
        let radiance = self.le(light_normal, -wi);

        if radiance.length_squared() < 1e-6 {
            return None;
        }

        // Convert area PDF to solid angle PDF
        // pdf_solid_angle = pdf_area * distance^2 / cos_theta
        let pdf = shape_pdf * distance * distance / cos_theta.abs();

        Some(LightSample {
            position: light_point,
            normal: light_normal,
            wi,
            radiance,
            pdf,
            distance,
        })
    }

    fn pdf_li(&self, point: Vec3, wi: Vec3) -> f64 {
        // For area lights, we need to convert solid angle PDF back to area PDF
        // This is used for MIS in path tracing

        // Cast ray to find intersection
        use crate::ray::Ray;
        let ray = Ray::new(point, wi);

        // Intersect with the area light primitive
        if let Some(isect) = self.primitive.intersect(&ray, 1e-4, f64::INFINITY) {
            // Check if we hit the emissive side
            let cos_theta = (-wi).dot(isect.normal);
            if self.two_sided || cos_theta > 0.0 {
                // Area PDF = 1 / surface_area
                let area_pdf = 1.0 / self.area;

                // Convert to solid angle PDF
                let distance_squared = isect.t * isect.t;
                area_pdf * distance_squared / cos_theta.abs()
            } else {
                0.0 // Hit back face of one-sided light
            }
        } else {
            0.0 // No intersection
        }
    }

    fn power(&self) -> f64 {
        // Power = emitted_radiance * area * pi (for diffuse emission)
        let factor = if self.two_sided { 2.0 } else { 1.0 };
        self.emission.length() * self.area * std::f64::consts::PI * factor
    }

    fn is_delta(&self) -> bool {
        false // Area lights have non-zero solid angle
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Rect, RectAxis};
    use crate::material::Lambertian;

    #[test]
    fn test_area_light_creation() {
        let material: Arc<dyn crate::material::Material> =
            Arc::new(Lambertian::new(Vec3::new(1.0, 1.0, 1.0)));
        let rect = Arc::new(Rect::new(
            RectAxis::XY,
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 1.0),
            5.0,
            material,
        )) as Arc<dyn Primitive>;
        let light = AreaLight::new(rect, Vec3::new(1.0, 1.0, 1.0));

        assert!(!light.is_delta());
        assert!(light.power() > 0.0);
    }

    #[test]
    fn test_area_light_sampling() {
        let material: Arc<dyn crate::material::Material> =
            Arc::new(Lambertian::new(Vec3::new(1.0, 1.0, 1.0)));
        let rect = Arc::new(Rect::new(
            RectAxis::XY,
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 1.0),
            5.0,
            material,
        )) as Arc<dyn Primitive>;
        let light = AreaLight::new(rect, Vec3::new(10.0, 10.0, 10.0));

        // Sample light from a point in front of it (rectangle is at z=5, sample from z=10)
        let point = Vec3::new(0.5, 0.5, 10.0);
        let u = Vec2::new(0.5, 0.5);

        let sample = light.sample_li(point, u);
        assert!(sample.is_some());

        if let Some(ls) = sample {
            assert!(ls.radiance.length() > 0.0);
            assert!(ls.pdf > 0.0);
            assert!(ls.distance > 0.0);
        }
    }

    #[test]
    fn test_area_light_two_sided() {
        let material: Arc<dyn crate::material::Material> =
            Arc::new(Lambertian::new(Vec3::new(1.0, 1.0, 1.0)));
        let rect = Arc::new(Rect::new(
            RectAxis::XY,
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 1.0),
            5.0,
            material,
        )) as Arc<dyn Primitive>;

        let one_sided = AreaLight::new(rect.clone(), Vec3::new(1.0, 1.0, 1.0));
        let two_sided = AreaLight::new(rect, Vec3::new(1.0, 1.0, 1.0)).with_two_sided(true);

        assert!(two_sided.power() > one_sided.power());
    }
}
