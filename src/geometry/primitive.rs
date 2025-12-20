use crate::core::intersection::Intersection;
use crate::core::math::{AABB, Vec2};
use crate::ray::Ray;
use crate::vec3::Vec3;

/// Primitive trait for all geometric objects
/// Replaces the old Hittable trait with richer intersection information
pub trait Primitive: Send + Sync {
    /// Test for ray intersection and return detailed intersection information
    ///
    /// # Arguments
    /// * `ray` - The ray to test
    /// * `t_min` - Minimum valid distance along ray
    /// * `t_max` - Maximum valid distance along ray
    ///
    /// # Returns
    /// * `Some(Intersection)` if the ray intersects the primitive
    /// * `None` if there is no intersection
    fn intersect(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<Intersection>;

    /// Fast intersection test without computing full intersection details
    /// Default implementation uses intersect(), but primitives can optimize this
    fn intersect_p(&self, ray: &Ray, t_min: f64, t_max: f64) -> bool {
        self.intersect(ray, t_min, t_max).is_some()
    }

    /// Get the world-space bounding box for this primitive
    fn world_bounds(&self) -> AABB;

    /// Get the surface area of this primitive (used for light sampling)
    fn surface_area(&self) -> f64 {
        0.0 // Default implementation for non-sampable primitives
    }

    /// Sample a point on the surface
    ///
    /// # Arguments
    /// * `u` - Random 2D sample in [0,1]²
    ///
    /// # Returns
    /// * `(point, normal, pdf)` - Sampled point, normal, and probability density
    fn sample(&self, _u: Vec2) -> (Vec3, Vec3, f64) {
        // Default implementation (not sampable)
        (Vec3::ZERO, Vec3::ZERO, 0.0)
    }

    /// Probability density function for sampling a direction from origin towards the surface
    ///
    /// # Arguments
    /// * `origin` - Point from which we're sampling
    /// * `direction` - Direction being queried
    ///
    /// # Returns
    /// * PDF value for the given direction
    fn pdf(&self, _origin: Vec3, _direction: Vec3) -> f64 {
        0.0 // Default implementation
    }
}

/// Wrapper that applies a transform to a primitive
pub struct TransformedPrimitive {
    primitive: Box<dyn Primitive>,
    object_to_world: crate::core::math::Transform,
    world_to_object: crate::core::math::Transform,
}

impl TransformedPrimitive {
    pub fn new(primitive: Box<dyn Primitive>, transform: crate::core::math::Transform) -> Self {
        let world_to_object = transform.inverse();
        TransformedPrimitive {
            primitive,
            object_to_world: transform,
            world_to_object,
        }
    }
}

impl Primitive for TransformedPrimitive {
    fn intersect(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<Intersection> {
        // Transform ray to object space
        let object_ray = self.world_to_object.transform_ray(ray);

        // Intersect in object space
        let mut isect = self.primitive.intersect(&object_ray, t_min, t_max)?;

        // Transform intersection back to world space
        isect.point = self.object_to_world.transform_point(isect.point);
        isect.normal = self
            .object_to_world
            .clone()
            .transform_normal(isect.normal)
            .normalized();

        // Transform tangent vectors
        isect.dpdu = self.object_to_world.transform_vector(isect.dpdu);
        isect.dpdv = self.object_to_world.transform_vector(isect.dpdv);

        Some(isect)
    }

    fn intersect_p(&self, ray: &Ray, t_min: f64, t_max: f64) -> bool {
        let object_ray = self.world_to_object.transform_ray(ray);
        self.primitive.intersect_p(&object_ray, t_min, t_max)
    }

    fn world_bounds(&self) -> AABB {
        // Transform object-space bounds to world space
        let object_bounds = self.primitive.world_bounds();
        self.object_to_world.transform_bounds(&object_bounds)
    }

    fn surface_area(&self) -> f64 {
        // Note: Surface area changes under non-uniform scaling
        // For now, just return the untransformed surface area
        // TODO: Properly account for transform
        self.primitive.surface_area()
    }

    fn sample(&self, u: Vec2) -> (Vec3, Vec3, f64) {
        let (point, normal, pdf) = self.primitive.sample(u);
        (
            self.object_to_world.transform_point(point),
            self.object_to_world
                .clone()
                .transform_normal(normal)
                .normalized(),
            pdf, // TODO: Adjust PDF for transformed area
        )
    }

    fn pdf(&self, origin: Vec3, direction: Vec3) -> f64 {
        // Transform to object space
        let object_origin = self.world_to_object.transform_point(origin);
        let object_direction = self.world_to_object.transform_vector(direction).normalized();
        self.primitive.pdf(object_origin, object_direction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test helper: Simple sphere primitive for testing
    struct TestSphere {
        center: Vec3,
        radius: f64,
    }

    impl Primitive for TestSphere {
        fn intersect(&self, _ray: &Ray, _t_min: f64, _t_max: f64) -> Option<Intersection> {
            None // Stub implementation
        }

        fn world_bounds(&self) -> AABB {
            let r = Vec3::new(self.radius, self.radius, self.radius);
            AABB::new(self.center - r, self.center + r)
        }
    }

    #[test]
    fn test_transformed_bounds() {
        use crate::core::math::Transform;

        let sphere = Box::new(TestSphere {
            center: Vec3::ZERO,
            radius: 1.0,
        });

        let transform = Transform::translate(Vec3::new(5.0, 0.0, 0.0));
        let transformed = TransformedPrimitive::new(sphere, transform);

        let bounds = transformed.world_bounds();
        let center = bounds.center();

        // Sphere should be translated to (5, 0, 0)
        assert!((center.x - 5.0).abs() < 1e-6);
        assert!(center.y.abs() < 1e-6);
        assert!(center.z.abs() < 1e-6);
    }
}
