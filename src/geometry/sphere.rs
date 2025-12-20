use crate::core::intersection::Intersection;
use crate::core::math::{Vec2, AABB};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;
use std::f64::consts::PI;
use std::sync::Arc;

use super::Primitive;

/// Sphere primitive centered at origin with given radius
/// Use Transform to position/scale the sphere in world space
pub struct Sphere {
    pub radius: f64,
    pub material: Arc<dyn Material>,
}

impl Sphere {
    /// Create a new sphere at origin with given radius
    pub fn new(radius: f64, material: Arc<dyn Material>) -> Self {
        Sphere { radius, material }
    }

    /// Compute UV coordinates from a point on the unit sphere
    /// U goes from 0 to 1 around the equator (longitude)
    /// V goes from 0 to 1 from bottom pole to top pole (latitude)
    fn get_sphere_uv(p: Vec3) -> Vec2 {
        let theta = (-p.y).acos(); // Angle from top pole (0 to PI)
        let phi = (-p.z).atan2(p.x) + PI; // Angle around Y axis (0 to 2*PI)

        Vec2::new(phi / (2.0 * PI), theta / PI)
    }

    /// Compute tangent vectors (dpdu, dpdv) for a point on the sphere
    fn get_sphere_tangents(p: Vec3) -> (Vec3, Vec3) {
        // Normalize point to unit sphere
        let n = p.normalized();

        // dpdu is tangent in phi direction (around equator)
        // At point (x, y, z) on unit sphere, dpdu = (-z, 0, x) (perpendicular to radius in XZ plane)
        let dpdu = Vec3::new(-n.z, 0.0, n.x).normalized();

        // dpdv is tangent in theta direction (from pole to pole)
        // This is perpendicular to both normal and dpdu
        let dpdv = n.cross(dpdu);

        (dpdu, dpdv)
    }
}

impl Primitive for Sphere {
    fn intersect(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<Intersection> {
        // Ray-sphere intersection (sphere at origin)
        let oc = ray.origin;

        // Quadratic coefficients
        let a = ray.direction.dot(ray.direction);
        let half_b = oc.dot(ray.direction);
        let c = oc.dot(oc) - self.radius * self.radius;

        // Discriminant
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        // Find nearest root in valid range
        let mut root = (-half_b - sqrtd) / a;

        if root < t_min || root > t_max {
            root = (-half_b + sqrtd) / a;
            if root < t_min || root > t_max {
                return None;
            }
        }

        // Compute intersection details
        let point = ray.at(root);
        let outward_normal = point / self.radius; // Normalize to get normal

        // Compute UV coordinates
        let uv = Self::get_sphere_uv(outward_normal);

        // Compute tangent vectors
        let (dpdu, dpdv) = Self::get_sphere_tangents(point);

        Some(Intersection::new(
            ray,
            root,
            point,
            outward_normal,
            uv,
            dpdu * self.radius, // Scale tangents by radius
            dpdv * self.radius,
            Arc::clone(&self.material),
        ))
    }

    fn world_bounds(&self) -> AABB {
        let r = Vec3::new(self.radius, self.radius, self.radius);
        AABB::new(-r, r)
    }

    fn surface_area(&self) -> f64 {
        4.0 * PI * self.radius * self.radius
    }

    fn sample(&self, u: Vec2) -> (Vec3, Vec3, f64) {
        // Uniform sampling on sphere surface
        let z = 1.0 - 2.0 * u.x;
        let r = (1.0 - z * z).max(0.0).sqrt();
        let phi = 2.0 * PI * u.y;
        let x = r * phi.cos();
        let y = r * phi.sin();

        let point = Vec3::new(x, y, z) * self.radius;
        let normal = point.normalized();
        let pdf = 1.0 / self.surface_area();

        (point, normal, pdf)
    }

    fn pdf(&self, origin: Vec3, direction: Vec3) -> f64 {
        // Create ray from origin in direction
        let ray = Ray::new(origin, direction.normalized());

        // Check if ray hits sphere
        if self.intersect(&ray, 0.001, f64::INFINITY).is_some() {
            // PDF for uniform sphere sampling
            let cos_theta_max = (1.0 - self.radius * self.radius / origin.length_squared())
                .max(0.0)
                .sqrt();
            let solid_angle = 2.0 * PI * (1.0 - cos_theta_max);
            1.0 / solid_angle
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;

    #[test]
    fn test_sphere_intersection() {
        let material = Arc::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5)));
        let sphere = Sphere::new(1.0, material);

        // Ray pointing at sphere from distance
        let ray = Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        let hit = sphere.intersect(&ray, 0.0, f64::INFINITY);

        assert!(hit.is_some());
        let isect = hit.unwrap();
        assert!((isect.t - 4.0).abs() < 1e-6); // Should hit at t=4 (5-1)
    }

    #[test]
    fn test_sphere_uv() {
        // Test UV mapping at equator
        let p = Vec3::new(1.0, 0.0, 0.0);
        let uv = Sphere::get_sphere_uv(p);
        // At x=1, y=0, z=0: phi should be around PI/2, theta around PI/2
        assert!(uv.x >= 0.0 && uv.x <= 1.0);
        assert!(uv.y >= 0.0 && uv.y <= 1.0);
    }

    #[test]
    fn test_sphere_bounds() {
        let material = Arc::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5)));
        let sphere = Sphere::new(2.0, material);
        let bounds = sphere.world_bounds();

        assert_eq!(bounds.min(), Vec3::new(-2.0, -2.0, -2.0));
        assert_eq!(bounds.max(), Vec3::new(2.0, 2.0, 2.0));
    }
}
