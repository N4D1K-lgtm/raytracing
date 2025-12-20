use crate::aabb::AABB;
use crate::hit::{HitRecord, Hittable};
use crate::ray::Ray;
use rand::Rng;

pub struct BvhNode {
    left: Box<dyn Hittable>,
    right: Box<dyn Hittable>,
    bbox: AABB,
}

impl BvhNode {
    pub fn new(mut objects: Vec<Box<dyn Hittable>>) -> Self {
        // Choose a random axis to split on
        let axis = rand::rng().random_range(0..3);

        let comparator = match axis {
            0 => box_x_compare,
            1 => box_y_compare,
            _ => box_z_compare,
        };

        let left: Box<dyn Hittable>;
        let right: Box<dyn Hittable>;

        match objects.len() {
            0 => panic!("Cannot create BVH from empty object list"),
            1 => {
                left = objects.pop().unwrap();
                right = Box::new(EmptyHittable);
            }
            2 => {
                right = objects.pop().unwrap();
                left = objects.pop().unwrap();
            }
            _ => {
                objects.sort_by(comparator);
                let mid = objects.len() / 2;
                let right_objects = objects.split_off(mid);
                left = Box::new(BvhNode::new(objects));
                right = Box::new(BvhNode::new(right_objects));
            }
        }

        let box_left = left
            .bounding_box()
            .expect("No bounding box in BvhNode constructor");
        let box_right = right
            .bounding_box()
            .unwrap_or(box_left); // Use left box if right is empty

        BvhNode {
            left,
            right,
            bbox: AABB::surrounding_box(box_left, box_right),
        }
    }
}

impl Hittable for BvhNode {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        if !self.bbox.hit(ray, t_min, t_max) {
            return None;
        }

        let hit_left = self.left.hit(ray, t_min, t_max);
        let t_max = if let Some(ref rec) = hit_left {
            rec.t
        } else {
            t_max
        };
        let hit_right = self.right.hit(ray, t_min, t_max);

        hit_right.or(hit_left)
    }

    fn bounding_box(&self) -> Option<AABB> {
        Some(self.bbox)
    }
}

// Empty hittable for single-object BVH nodes
struct EmptyHittable;

impl Hittable for EmptyHittable {
    fn hit(&self, _ray: &Ray, _t_min: f64, _t_max: f64) -> Option<HitRecord> {
        None
    }

    fn bounding_box(&self) -> Option<AABB> {
        None
    }
}

// Comparison functions for sorting
fn box_compare(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>, axis: usize) -> std::cmp::Ordering {
    let box_a = a.bounding_box().expect("No bounding box in BVH comparison");
    let box_b = b.bounding_box().expect("No bounding box in BVH comparison");

    box_a.min[axis]
        .partial_cmp(&box_b.min[axis])
        .unwrap_or(std::cmp::Ordering::Equal)
}

fn box_x_compare(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>) -> std::cmp::Ordering {
    box_compare(a, b, 0)
}

fn box_y_compare(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>) -> std::cmp::Ordering {
    box_compare(a, b, 1)
}

fn box_z_compare(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>) -> std::cmp::Ordering {
    box_compare(a, b, 2)
}
