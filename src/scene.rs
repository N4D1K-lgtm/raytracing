use crate::{
    aabb::AABB,
    bvh::BvhNode,
    hit::{HitRecord, Hittable},
    ray::Ray,
};

pub struct Scene {
    // Staging area for objects before BVH is built
    staging: Vec<Box<dyn Hittable>>,
    // The BVH owns all objects after building
    bvh: Option<BvhNode>,
    object_count: usize,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            staging: Vec::new(),
            bvh: None,
            object_count: 0,
        }
    }

    pub fn add(&mut self, object: impl Hittable + 'static) {
        self.staging.push(Box::new(object));
        self.object_count += 1;
        // Mark that we need to rebuild BVH
        self.bvh = None;
    }

    pub fn clear(&mut self) {
        self.staging.clear();
        self.bvh = None;
        self.object_count = 0;
    }

    pub fn len(&self) -> usize {
        self.object_count
    }

    pub fn is_empty(&self) -> bool {
        self.object_count == 0
    }

    pub fn build_bvh(&mut self) {
        if self.staging.is_empty() {
            return;
        }

        if self.staging.len() == 1 {
            // For single object, no need for BVH
            return;
        }

        // Take ownership of staging objects and build BVH
        let objects = std::mem::take(&mut self.staging);
        self.bvh = Some(BvhNode::new(objects));
    }

    pub fn needs_rebuild(&self) -> bool {
        !self.staging.is_empty() && self.bvh.is_none()
    }
}

impl Hittable for Scene {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        // Try BVH first
        if let Some(bvh) = &self.bvh {
            return bvh.hit(ray, t_min, t_max);
        }

        // Fall back to linear search through staging
        let mut closest_so_far = t_max;
        let mut hit_record = None;

        for object in &self.staging {
            if let Some(hit) = object.hit(ray, t_min, closest_so_far) {
                closest_so_far = hit.t;
                hit_record = Some(hit);
            }
        }

        hit_record
    }

    fn bounding_box(&self) -> Option<AABB> {
        if let Some(bvh) = &self.bvh {
            return bvh.bounding_box();
        }

        if self.staging.is_empty() {
            return None;
        }

        let mut output_box: Option<AABB> = None;

        for object in &self.staging {
            if let Some(bbox) = object.bounding_box() {
                output_box = Some(if let Some(ob) = output_box {
                    AABB::surrounding_box(ob, bbox)
                } else {
                    bbox
                });
            } else {
                return None;
            }
        }

        output_box
    }
}
