use crate::{
    hit::{HitRecord, Hittable},
    material::Material,
    ray::Ray,
    vec3::Vec3,
};
use std::sync::Arc;

pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
    pub material: Arc<dyn Material>,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f64, material: Arc<dyn Material>) -> Self {
        Sphere {
            center,
            radius,
            material,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        // vector from sphere center to ray origin
        let oc = ray.origin - self.center;

        // quadratic coefficients with direction normalized, a = 1
        let a = ray.direction.dot(ray.direction);
        let half_b = oc.dot(ray.direction);
        let c = oc.dot(oc) - self.radius * self.radius;

        // discriminant
        let discriminant = half_b * half_b - a * c;

        // no intersection if discriminant is neg
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        // find nearest root (smallest t) in the valid range
        let mut root = (-half_b - sqrtd) / a;

        if root < t_min || root > t_max {
            // try the other root
            root = (-half_b + sqrtd) / a;

            if root < t_min || root > t_max {
                return None;
            }
        }

        // Calculate hit point
        let point = ray.at(root);

        // Calculate outward normal (points from center through hit point)
        let outward_normal = (point - self.center) / self.radius;

        // Create hit record with proper facing
        Some(HitRecord::new(
            ray,
            point,
            outward_normal,
            root,
            Arc::clone(&self.material),
        ))
    }

    fn bounding_box(&self) -> Option<crate::aabb::AABB> {
        let radius_vec = Vec3::new(self.radius, self.radius, self.radius);
        Some(crate::aabb::AABB::new(
            self.center - radius_vec,
            self.center + radius_vec,
        ))
    }
}
