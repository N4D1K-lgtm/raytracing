use crate::{material::Material, ray::Ray, vec3::Vec3};
use std::sync::Arc;

pub struct HitRecord {
    pub point: Vec3,
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool,
    pub material: Arc<dyn Material>,
}

impl HitRecord {
    /// Create a new HitRecord with automatic front-face detection
    /// The normal will be set to always point against the ray direction
    pub fn new(
        ray: &Ray,
        point: Vec3,
        outward_normal: Vec3,
        t: f64,
        material: Arc<dyn Material>,
    ) -> HitRecord {
        // Determine if ray hit from outside (front face)
        let front_face = ray.direction.dot(outward_normal) < 0.0;

        // Make normal point against ray
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        HitRecord {
            point,
            normal,
            t,
            front_face,
            material,
        }
    }
}

pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
    fn bounding_box(&self) -> Option<crate::aabb::AABB>;
}
