use crate::core::intersection::Intersection;
use crate::core::math::{AABB, Vec2};
use crate::materials::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;
use std::sync::Arc;

use super::Primitive;

/// Axis-aligned rectangle orientation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RectAxis {
    XY, // Rectangle in XY plane (perpendicular to Z)
    XZ, // Rectangle in XZ plane (perpendicular to Y)
    YZ, // Rectangle in YZ plane (perpendicular to X)
}

/// Axis-aligned rectangle primitive
pub struct Rect {
    /// Rectangle orientation
    pub axis: RectAxis,
    /// Minimum coordinates in the two primary axes
    pub min: Vec2,
    /// Maximum coordinates in the two primary axes
    pub max: Vec2,
    /// Position along the perpendicular axis
    pub k: f64,
    /// Material
    pub material: Arc<dyn Material>,
}

impl Rect {
    /// Create a new axis-aligned rectangle
    ///
    /// # Arguments
    /// * `axis` - Which plane the rectangle lies in
    /// * `min` - Minimum coordinates (u, v)
    /// * `max` - Maximum coordinates (u, v)
    /// * `k` - Position along the perpendicular axis
    /// * `material` - Material
    ///
    /// # Example
    /// ```ignore
    /// // Rectangle in XY plane at z=5, from (0,0) to (10,10)
    /// let rect = Rect::new(
    ///     RectAxis::XY,
    ///     Vec2::new(0.0, 0.0),
    ///     Vec2::new(10.0, 10.0),
    ///     5.0,
    ///     material
    /// );
    /// ```
    pub fn new(axis: RectAxis, min: Vec2, max: Vec2, k: f64, material: Arc<dyn Material>) -> Self {
        Rect {
            axis,
            min,
            max,
            material,
            k,
        }
    }

    /// Get the 3D point from UV coordinates on this rectangle
    fn uv_to_point(&self, u: f64, v: f64) -> Vec3 {
        match self.axis {
            RectAxis::XY => Vec3::new(u, v, self.k),
            RectAxis::XZ => Vec3::new(u, self.k, v),
            RectAxis::YZ => Vec3::new(self.k, u, v),
        }
    }

    /// Get the normal vector for this rectangle
    fn get_normal(&self) -> Vec3 {
        match self.axis {
            RectAxis::XY => Vec3::new(0.0, 0.0, 1.0),
            RectAxis::XZ => Vec3::new(0.0, 1.0, 0.0),
            RectAxis::YZ => Vec3::new(1.0, 0.0, 0.0),
        }
    }

    /// Get tangent vectors for this rectangle
    fn get_tangents(&self) -> (Vec3, Vec3) {
        match self.axis {
            RectAxis::XY => (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
            RectAxis::XZ => (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
            RectAxis::YZ => (Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
        }
    }
}

impl Primitive for Rect {
    fn intersect(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<Intersection> {
        // Determine which axis is perpendicular and which are parallel
        let (axis_idx, u_idx, v_idx) = match self.axis {
            RectAxis::XY => (2, 0, 1), // Z perpendicular, X and Y parallel
            RectAxis::XZ => (1, 0, 2), // Y perpendicular, X and Z parallel
            RectAxis::YZ => (0, 1, 2), // X perpendicular, Y and Z parallel
        };

        // Check if ray is parallel to rectangle
        if ray.direction[axis_idx].abs() < 1e-8 {
            return None;
        }

        // Compute t value for intersection with plane
        let t = (self.k - ray.origin[axis_idx]) / ray.direction[axis_idx];

        // Check if t is in valid range
        if t < t_min || t > t_max {
            return None;
        }

        // Compute intersection point
        let point = ray.at(t);

        // Extract u and v coordinates from the point
        let u = point[u_idx];
        let v = point[v_idx];

        // Check if point is within rectangle bounds
        if u < self.min.x || u > self.max.x || v < self.min.y || v > self.max.y {
            return None;
        }

        // Compute UV coordinates (normalized to [0,1])
        let uv = Vec2::new(
            (u - self.min.x) / (self.max.x - self.min.x),
            (v - self.min.y) / (self.max.y - self.min.y),
        );

        // Get normal and tangents
        let normal = self.get_normal();
        let (dpdu, dpdv) = self.get_tangents();

        Some(Intersection::new(
            ray,
            t,
            point,
            normal,
            uv,
            dpdu * (self.max.x - self.min.x),
            dpdv * (self.max.y - self.min.y),
            Arc::clone(&self.material),
        ))
    }

    fn world_bounds(&self) -> AABB {
        let p0 = self.uv_to_point(self.min.x, self.min.y);
        let p1 = self.uv_to_point(self.max.x, self.max.y);

        // Expand slightly to avoid zero-thickness
        let epsilon = 0.0001;
        AABB::new(
            Vec3::new(
                p0.x.min(p1.x) - epsilon,
                p0.y.min(p1.y) - epsilon,
                p0.z.min(p1.z) - epsilon,
            ),
            Vec3::new(
                p0.x.max(p1.x) + epsilon,
                p0.y.max(p1.y) + epsilon,
                p0.z.max(p1.z) + epsilon,
            ),
        )
    }

    fn surface_area(&self) -> f64 {
        let width = self.max.x - self.min.x;
        let height = self.max.y - self.min.y;
        width * height
    }

    fn sample(&self, u: Vec2) -> (Vec3, Vec3, f64) {
        // Uniform sampling on rectangle
        let point_u = self.min.x + u.x * (self.max.x - self.min.x);
        let point_v = self.min.y + u.y * (self.max.y - self.min.y);
        let point = self.uv_to_point(point_u, point_v);
        let normal = self.get_normal();
        let pdf = 1.0 / self.surface_area();

        (point, normal, pdf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::material::DiffuseMaterial;

    #[test]
    fn test_xy_rect_intersection() {
        let material = Arc::new(DiffuseMaterial::new(Vec3::new(0.5, 0.5, 0.5)));

        // XY rectangle at z=0, from (0,0) to (5,5)
        let rect = Rect::new(
            RectAxis::XY,
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 5.0),
            0.0,
            material,
        );

        // Ray pointing at center of rectangle
        let ray = Ray::new(Vec3::new(2.5, 2.5, 10.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = rect.intersect(&ray, 0.0, f64::INFINITY);

        assert!(hit.is_some());
        let isect = hit.unwrap();
        assert!((isect.t - 10.0).abs() < 1e-6);
        assert!((isect.point.x - 2.5).abs() < 1e-6);
        assert!((isect.point.y - 2.5).abs() < 1e-6);
        assert!(isect.point.z.abs() < 1e-6);
    }

    #[test]
    fn test_rect_out_of_bounds() {
        let material = Arc::new(DiffuseMaterial::new(Vec3::new(0.5, 0.5, 0.5)));

        // XY rectangle at z=0, from (0,0) to (5,5)
        let rect = Rect::new(
            RectAxis::XY,
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 5.0),
            0.0,
            material,
        );

        // Ray pointing outside rectangle bounds
        let ray = Ray::new(Vec3::new(10.0, 10.0, 10.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = rect.intersect(&ray, 0.0, f64::INFINITY);

        assert!(hit.is_none());
    }

    #[test]
    fn test_xz_rect_intersection() {
        let material = Arc::new(DiffuseMaterial::new(Vec3::new(0.5, 0.5, 0.5)));

        // XZ rectangle at y=0, from (0,0) to (5,5)
        let rect = Rect::new(
            RectAxis::XZ,
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 5.0),
            0.0,
            material,
        );

        // Ray pointing at center of rectangle from above
        let ray = Ray::new(Vec3::new(2.5, 10.0, 2.5), Vec3::new(0.0, -1.0, 0.0));
        let hit = rect.intersect(&ray, 0.0, f64::INFINITY);

        assert!(hit.is_some());
        let isect = hit.unwrap();
        assert!((isect.t - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_rect_surface_area() {
        let material = Arc::new(DiffuseMaterial::new(Vec3::new(0.5, 0.5, 0.5)));

        let rect = Rect::new(
            RectAxis::XY,
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 5.0),
            0.0,
            material,
        );

        assert!((rect.surface_area() - 50.0).abs() < 1e-6);
    }
}
