use crate::core::math::Vec2;
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;
use std::sync::Arc;

/// Intersection record containing all information about a ray-surface intersection
#[derive(Clone)]
pub struct Intersection {
    /// Distance along ray where intersection occurred
    pub t: f64,

    /// Point of intersection in world space
    pub point: Vec3,

    /// Surface normal at intersection (always points against ray direction)
    pub normal: Vec3,

    /// UV texture coordinates at intersection
    pub uv: Vec2,

    /// Partial derivative of position with respect to U (tangent vector)
    pub dpdu: Vec3,

    /// Partial derivative of position with respect to V (bitangent vector)
    pub dpdv: Vec3,

    /// True if ray hit the front face of the surface
    pub front_face: bool,

    /// Material at the intersection point
    pub material: Arc<dyn Material>,
}

impl Intersection {
    /// Create a new intersection with automatic front-face detection
    /// The normal will be set to always point against the ray direction
    pub fn new(
        ray: &Ray,
        t: f64,
        point: Vec3,
        outward_normal: Vec3,
        uv: Vec2,
        dpdu: Vec3,
        dpdv: Vec3,
        material: Arc<dyn Material>,
    ) -> Self {
        // Determine if ray hit from outside (front face)
        let front_face = ray.direction.dot(outward_normal) < 0.0;

        // Make normal point against ray
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        Intersection {
            t,
            point,
            normal,
            uv,
            dpdu,
            dpdv,
            front_face,
            material,
        }
    }

    /// Create intersection with default tangent vectors (computed from normal)
    pub fn with_default_tangents(
        ray: &Ray,
        t: f64,
        point: Vec3,
        outward_normal: Vec3,
        uv: Vec2,
        material: Arc<dyn Material>,
    ) -> Self {
        // Compute arbitrary tangent space from normal
        let (dpdu, dpdv) = Self::compute_tangent_space(outward_normal);

        Self::new(ray, t, point, outward_normal, uv, dpdu, dpdv, material)
    }

    /// Compute an arbitrary tangent space from a normal
    fn compute_tangent_space(normal: Vec3) -> (Vec3, Vec3) {
        // Create arbitrary orthonormal basis from normal
        let dpdu = if normal.x.abs() > 0.9 {
            Vec3::new(0.0, 1.0, 0.0).cross(normal).normalized()
        } else {
            Vec3::new(1.0, 0.0, 0.0).cross(normal).normalized()
        };

        let dpdv = normal.cross(dpdu);

        (dpdu, dpdv)
    }

    /// Get the shading normal (can be modified by normal mapping)
    pub fn shading_normal(&self) -> Vec3 {
        // For now, just return the geometric normal
        // Later this can be modified by normal maps
        self.normal
    }

    /// Get the tangent vector (normalized dpdu)
    pub fn tangent(&self) -> Vec3 {
        self.dpdu.normalized()
    }

    /// Get the bitangent vector (normalized dpdv)
    pub fn bitangent(&self) -> Vec3 {
        self.dpdv.normalized()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;

    #[test]
    fn test_front_face_detection() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0));
        let material = Arc::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5)));

        let isect = Intersection::with_default_tangents(
            &ray,
            1.0,
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, -1.0), // Outward normal pointing back at ray
            Vec2::ZERO,
            material,
        );

        assert!(isect.front_face);
        assert_eq!(isect.normal, Vec3::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn test_back_face_detection() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0));
        let material = Arc::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5)));

        let isect = Intersection::with_default_tangents(
            &ray,
            1.0,
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0), // Outward normal pointing same direction as ray
            Vec2::ZERO,
            material,
        );

        assert!(!isect.front_face);
        assert_eq!(isect.normal, Vec3::new(0.0, 0.0, -1.0)); // Flipped to point against ray
    }
}
