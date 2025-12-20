use crate::core::intersection::Intersection;
use crate::core::math::AABB;
use crate::geometry::Primitive;
use crate::ray::Ray;
use rand::Rng;
use std::sync::Arc;

/// BVH node for accelerating ray-primitive intersections
pub enum BvhNode {
    /// Leaf node containing a single primitive
    Leaf {
        primitive: Arc<dyn Primitive>,
        bbox: AABB,
    },
    /// Internal node with left and right children
    Internal {
        left: Box<BvhNode>,
        right: Box<BvhNode>,
        bbox: AABB,
    },
}

impl BvhNode {
    /// Build a BVH from a vector of primitives
    pub fn build(primitives: Vec<Arc<dyn Primitive>>) -> Option<Self> {
        if primitives.is_empty() {
            return None;
        }

        build_bvh_recursive(primitives)
    }

    /// Test for ray intersection, returning closest hit
    pub fn intersect(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<Intersection> {
        // First test against bounding box
        if !self.bbox().hit(ray, t_min, t_max) {
            return None;
        }

        match self {
            BvhNode::Leaf { primitive, .. } => primitive.intersect(ray, t_min, t_max),
            BvhNode::Internal { left, right, .. } => {
                // Test both children, keeping closest hit
                let hit_left = left.intersect(ray, t_min, t_max);
                let t_max = if let Some(ref hit) = hit_left {
                    hit.t
                } else {
                    t_max
                };
                let hit_right = right.intersect(ray, t_min, t_max);

                // Return closest hit
                hit_right.or(hit_left)
            }
        }
    }

    /// Fast intersection test without computing full intersection
    pub fn intersect_p(&self, ray: &Ray, t_min: f64, t_max: f64) -> bool {
        if !self.bbox().hit(ray, t_min, t_max) {
            return false;
        }

        match self {
            BvhNode::Leaf { primitive, .. } => primitive.intersect_p(ray, t_min, t_max),
            BvhNode::Internal { left, right, .. } => {
                left.intersect_p(ray, t_min, t_max) || right.intersect_p(ray, t_min, t_max)
            }
        }
    }

    /// Get the bounding box of this node
    pub fn bbox(&self) -> &AABB {
        match self {
            BvhNode::Leaf { bbox, .. } => bbox,
            BvhNode::Internal { bbox, .. } => bbox,
        }
    }
}

/// Build BVH recursively
fn build_bvh_recursive(mut primitives: Vec<Arc<dyn Primitive>>) -> Option<BvhNode> {
    match primitives.len() {
        0 => None,
        1 => {
            let primitive = primitives.pop().unwrap();
            let bbox = primitive.world_bounds();
            Some(BvhNode::Leaf { primitive, bbox })
        }
        _ => {
            // Choose split axis (random for now, could use SAH)
            let axis = rand::rng().random_range(0..3);

            // Sort primitives along chosen axis
            primitives.sort_by(|a, b| {
                let bbox_a = a.world_bounds();
                let bbox_b = b.world_bounds();
                let center_a = bbox_a.center()[axis];
                let center_b = bbox_b.center()[axis];
                center_a.partial_cmp(&center_b).unwrap_or(std::cmp::Ordering::Equal)
            });

            // Split primitives
            let mid = primitives.len() / 2;
            let right_prims = primitives.split_off(mid);

            // Recursively build children
            let left = build_bvh_recursive(primitives)?;
            let right = build_bvh_recursive(right_prims)?;

            // Compute combined bounding box
            let bbox = AABB::surrounding_box(*left.bbox(), *right.bbox());

            Some(BvhNode::Internal {
                left: Box::new(left),
                right: Box::new(right),
                bbox,
            })
        }
    }
}

/// Convenience function to build a BVH from a slice of primitives
pub fn build_bvh(primitives: &[Arc<dyn Primitive>]) -> Option<BvhNode> {
    BvhNode::build(primitives.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Sphere;
    use crate::material::Lambertian;
    use crate::vec3::Vec3;

    #[test]
    fn test_bvh_single_primitive() {
        let material: Arc<dyn crate::material::Material> = Arc::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5)));
        let sphere = Arc::new(Sphere::new(1.0, material)) as Arc<dyn Primitive>;

        let bvh = BvhNode::build(vec![sphere]).unwrap();

        // Ray hitting the sphere
        let ray = Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        let hit = bvh.intersect(&ray, 0.0, f64::INFINITY);
        assert!(hit.is_some());
    }

    #[test]
    fn test_bvh_multiple_primitives() {
        let material: Arc<dyn crate::material::Material> = Arc::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5)));

        let spheres: Vec<Arc<dyn Primitive>> = vec![
            Arc::new(Sphere::new(1.0, Arc::clone(&material))),
            Arc::new(Sphere::new(1.0, Arc::clone(&material))),
            Arc::new(Sphere::new(1.0, Arc::clone(&material))),
        ];

        let bvh = BvhNode::build(spheres).unwrap();

        // Test that BVH was built
        match bvh {
            BvhNode::Internal { .. } => { /* Expected */ }
            BvhNode::Leaf { .. } => panic!("Should be internal node with 3 primitives"),
        }
    }

    #[test]
    fn test_bvh_intersection() {
        let material: Arc<dyn crate::material::Material> = Arc::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5)));

        // Create multiple spheres at different positions using TransformedPrimitive
        use crate::core::math::Transform;
        use crate::geometry::TransformedPrimitive;

        let spheres: Vec<Arc<dyn Primitive>> = vec![
            Arc::new(TransformedPrimitive::new(
                Box::new(Sphere::new(1.0, Arc::clone(&material))),
                Transform::translate(Vec3::new(-3.0, 0.0, 0.0)),
            )),
            Arc::new(TransformedPrimitive::new(
                Box::new(Sphere::new(1.0, Arc::clone(&material))),
                Transform::translate(Vec3::new(0.0, 0.0, 0.0)),
            )),
            Arc::new(TransformedPrimitive::new(
                Box::new(Sphere::new(1.0, Arc::clone(&material))),
                Transform::translate(Vec3::new(3.0, 0.0, 0.0)),
            )),
        ];

        let bvh = BvhNode::build(spheres).unwrap();

        // Ray hitting middle sphere
        let ray = Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        let hit = bvh.intersect(&ray, 0.0, f64::INFINITY);
        assert!(hit.is_some());

        // Ray missing all spheres
        let ray = Ray::new(Vec3::new(10.0, 10.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        let hit = bvh.intersect(&ray, 0.0, f64::INFINITY);
        assert!(hit.is_none());
    }

    #[test]
    fn test_bvh_empty() {
        let bvh = BvhNode::build(vec![]);
        assert!(bvh.is_none());
    }
}
