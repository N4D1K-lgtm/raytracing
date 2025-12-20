use crate::ray::Ray;
use crate::vec3::Vec3;

/// Axis-Aligned Bounding Box
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    min: Vec3,
    max: Vec3,
}

impl AABB {
    /// Create new AABB from min and max points
    pub fn new(min: Vec3, max: Vec3) -> Self {
        AABB { min, max }
    }

    /// Create AABB from a single point
    pub fn from_point(p: Vec3) -> Self {
        AABB { min: p, max: p }
    }

    /// Create empty AABB (inverted bounds for merging)
    pub fn empty() -> Self {
        AABB {
            min: Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            max: Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
        }
    }

    /// Get minimum point
    #[inline]
    pub fn min(&self) -> Vec3 {
        self.min
    }

    /// Get maximum point
    #[inline]
    pub fn max(&self) -> Vec3 {
        self.max
    }

    /// Get center of AABB
    #[inline]
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get diagonal vector (max - min)
    #[inline]
    pub fn diagonal(&self) -> Vec3 {
        self.max - self.min
    }

    /// Get surface area
    pub fn surface_area(&self) -> f64 {
        let d = self.diagonal();
        2.0 * (d.x * d.y + d.y * d.z + d.z * d.x)
    }

    /// Get volume
    pub fn volume(&self) -> f64 {
        let d = self.diagonal();
        d.x * d.y * d.z
    }

    /// Get the longest axis (0=X, 1=Y, 2=Z)
    pub fn longest_axis(&self) -> usize {
        let d = self.diagonal();
        if d.x > d.y && d.x > d.z {
            0
        } else if d.y > d.z {
            1
        } else {
            2
        }
    }

    /// Fast ray-box intersection test using the slab method
    pub fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> bool {
        let mut t_min = t_min;
        let mut t_max = t_max;

        for axis in 0..3 {
            let inv_d = 1.0 / ray.direction[axis];
            let mut t0 = (self.min[axis] - ray.origin[axis]) * inv_d;
            let mut t1 = (self.max[axis] - ray.origin[axis]) * inv_d;

            if inv_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }

            t_min = if t0 > t_min { t0 } else { t_min };
            t_max = if t1 < t_max { t1 } else { t_max };

            if t_max <= t_min {
                return false;
            }
        }
        true
    }

    /// Merge two AABBs into a single bounding box
    pub fn surrounding_box(box0: AABB, box1: AABB) -> AABB {
        let min = Vec3::new(
            box0.min.x.min(box1.min.x),
            box0.min.y.min(box1.min.y),
            box0.min.z.min(box1.min.z),
        );

        let max = Vec3::new(
            box0.max.x.max(box1.max.x),
            box0.max.y.max(box1.max.y),
            box0.max.z.max(box1.max.z),
        );

        AABB::new(min, max)
    }

    /// Merge this AABB with another
    pub fn merge(&self, other: &AABB) -> AABB {
        AABB::surrounding_box(*self, *other)
    }

    /// Merge this AABB with a point
    pub fn merge_point(&self, p: Vec3) -> AABB {
        AABB::new(
            Vec3::new(
                self.min.x.min(p.x),
                self.min.y.min(p.y),
                self.min.z.min(p.z),
            ),
            Vec3::new(
                self.max.x.max(p.x),
                self.max.y.max(p.y),
                self.max.z.max(p.z),
            ),
        )
    }

    /// Expand the bounding box by a small epsilon to avoid zero-volume boxes
    pub fn expand(&self, delta: f64) -> AABB {
        let padding = Vec3::new(delta, delta, delta);
        AABB::new(self.min - padding, self.max + padding)
    }

    /// Check if a point is inside the AABB
    pub fn contains_point(&self, p: Vec3) -> bool {
        p.x >= self.min.x
            && p.x <= self.max.x
            && p.y >= self.min.y
            && p.y <= self.max.y
            && p.z >= self.min.z
            && p.z <= self.max.z
    }

    /// Check if this AABB overlaps with another
    pub fn overlaps(&self, other: &AABB) -> bool {
        self.max.x >= other.min.x
            && self.min.x <= other.max.x
            && self.max.y >= other.min.y
            && self.min.y <= other.max.y
            && self.max.z >= other.min.z
            && self.min.z <= other.max.z
    }
}

// Re-export for compatibility
pub use AABB as Bounds3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_center() {
        let aabb = AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 4.0, 6.0));
        assert_eq!(aabb.center(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_surface_area() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(aabb.surface_area(), 6.0); // Cube with side 1
    }

    #[test]
    fn test_contains_point() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
        assert!(aabb.contains_point(Vec3::new(0.5, 0.5, 0.5)));
        assert!(!aabb.contains_point(Vec3::new(2.0, 0.5, 0.5)));
    }
}
