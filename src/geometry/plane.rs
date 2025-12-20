use crate::core::intersection::Intersection;
use crate::core::math::{Vec2, AABB};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;
use std::sync::Arc;

use super::Primitive;

/// Infinite plane primitive
/// Defined by a point on the plane and a normal vector
pub struct Plane {
    /// A point on the plane (used for distance calculation)
    pub point: Vec3,
    /// Normal vector (should be normalized)
    pub normal: Vec3,
    /// Material
    pub material: Arc<dyn Material>,
    /// Scale for UV coordinates (larger = more repetitions)
    pub uv_scale: f64,
}

impl Plane {
    /// Create a new plane
    /// normal will be automatically normalized
    pub fn new(point: Vec3, normal: Vec3, material: Arc<dyn Material>) -> Self {
        Plane {
            point,
            normal: normal.normalized(),
            material,
            uv_scale: 1.0,
        }
    }

    /// Create a plane with custom UV scale
    pub fn with_uv_scale(mut self, scale: f64) -> Self {
        self.uv_scale = scale;
        self
    }

    /// Compute UV coordinates for a point on the plane
    fn get_plane_uv(&self, p: Vec3) -> Vec2 {
        // Create a coordinate system on the plane
        let (u_axis, v_axis) = self.get_plane_basis();

        // Project point onto plane axes
        let offset = p - self.point;
        let u = offset.dot(u_axis) * self.uv_scale;
        let v = offset.dot(v_axis) * self.uv_scale;

        Vec2::new(u, v)
    }

    /// Get orthonormal basis vectors on the plane
    fn get_plane_basis(&self) -> (Vec3, Vec3) {
        // Create arbitrary orthonormal basis from normal
        let u_axis = if self.normal.x.abs() > 0.9 {
            Vec3::new(0.0, 1.0, 0.0).cross(self.normal).normalized()
        } else {
            Vec3::new(1.0, 0.0, 0.0).cross(self.normal).normalized()
        };

        let v_axis = self.normal.cross(u_axis);

        (u_axis, v_axis)
    }
}

impl Primitive for Plane {
    fn intersect(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<Intersection> {
        let denom = self.normal.dot(ray.direction);

        // Check if ray is parallel to plane (or very close to parallel)
        if denom.abs() < 1e-8 {
            return None;
        }

        // Compute t value for intersection
        let t = (self.point - ray.origin).dot(self.normal) / denom;

        // Check if intersection is in valid range
        if t < t_min || t > t_max {
            return None;
        }

        // Compute intersection point
        let point = ray.at(t);

        // Compute UV coordinates
        let uv = self.get_plane_uv(point);

        // Get tangent vectors
        let (dpdu, dpdv) = self.get_plane_basis();

        Some(Intersection::new(
            ray,
            t,
            point,
            self.normal,
            uv,
            dpdu,
            dpdv,
            Arc::clone(&self.material),
        ))
    }

    fn world_bounds(&self) -> AABB {
        // Infinite plane has infinite bounds
        // Return a very large bounding box
        let big = 1e10;
        AABB::new(
            Vec3::new(-big, -big, -big),
            Vec3::new(big, big, big),
        )
    }

    fn surface_area(&self) -> f64 {
        // Infinite surface area
        f64::INFINITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;

    #[test]
    fn test_plane_intersection() {
        let material = Arc::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5)));

        // XY plane at z=0
        let plane = Plane::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            material,
        );

        // Ray pointing at plane from above
        let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = plane.intersect(&ray, 0.0, f64::INFINITY);

        assert!(hit.is_some());
        let isect = hit.unwrap();
        assert!((isect.t - 5.0).abs() < 1e-6);
        assert!((isect.point.z).abs() < 1e-6);
    }

    #[test]
    fn test_plane_parallel_ray() {
        let material = Arc::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5)));

        // XY plane at z=0
        let plane = Plane::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            material,
        );

        // Ray parallel to plane
        let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(1.0, 0.0, 0.0));
        let hit = plane.intersect(&ray, 0.0, f64::INFINITY);

        assert!(hit.is_none());
    }

    #[test]
    fn test_plane_normal_facing() {
        let material = Arc::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5)));

        // XY plane at z=0
        let plane = Plane::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            material,
        );

        // Ray from above (hitting front face)
        let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = plane.intersect(&ray, 0.0, f64::INFINITY).unwrap();
        assert!(hit.front_face);

        // Ray from below (hitting back face)
        let ray = Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        let hit = plane.intersect(&ray, 0.0, f64::INFINITY).unwrap();
        assert!(!hit.front_face);
    }
}
