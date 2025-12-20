use super::bounds::AABB;
use super::{Mat4, Quaternion};
use crate::ray::Ray;
use crate::vec3::Vec3;

/// Transform represents a spatial transformation with cached inverse
#[derive(Clone, Debug)]
pub struct Transform {
    matrix: Mat4,
    inverse: Option<Mat4>,
}

impl Transform {
    /// Create identity transform
    pub fn identity() -> Self {
        Transform {
            matrix: Mat4::identity(),
            inverse: Some(Mat4::identity()),
        }
    }

    /// Create transform from matrix
    pub fn from_matrix(matrix: Mat4) -> Self {
        Transform {
            matrix,
            inverse: None,
        }
    }

    /// Create transform from matrix with pre-computed inverse
    pub fn from_matrix_pair(matrix: Mat4, inverse: Mat4) -> Self {
        Transform {
            matrix,
            inverse: Some(inverse),
        }
    }

    /// Create translation transform
    pub fn translate(v: Vec3) -> Self {
        let matrix = Mat4::translate(v);
        let inverse = Mat4::translate(-v);
        Transform {
            matrix,
            inverse: Some(inverse),
        }
    }

    /// Create rotation transform from quaternion
    pub fn rotate(q: Quaternion) -> Self {
        let matrix = q.to_matrix();
        let inverse = q.inverse().to_matrix();
        Transform {
            matrix,
            inverse: Some(inverse),
        }
    }

    /// Create rotation transform from axis-angle
    pub fn rotate_axis(axis: Vec3, angle: f64) -> Self {
        let q = Quaternion::from_axis_angle(axis, angle);
        Self::rotate(q)
    }

    /// Create rotation transform around X axis
    pub fn rotate_x(angle: f64) -> Self {
        let matrix = Mat4::rotate_x(angle);
        let inverse = Mat4::rotate_x(-angle);
        Transform {
            matrix,
            inverse: Some(inverse),
        }
    }

    /// Create rotation transform around Y axis
    pub fn rotate_y(angle: f64) -> Self {
        let matrix = Mat4::rotate_y(angle);
        let inverse = Mat4::rotate_y(-angle);
        Transform {
            matrix,
            inverse: Some(inverse),
        }
    }

    /// Create rotation transform around Z axis
    pub fn rotate_z(angle: f64) -> Self {
        let matrix = Mat4::rotate_z(angle);
        let inverse = Mat4::rotate_z(-angle);
        Transform {
            matrix,
            inverse: Some(inverse),
        }
    }

    /// Create scale transform
    pub fn scale(s: Vec3) -> Self {
        let matrix = Mat4::scale(s);
        let inverse = Mat4::scale(Vec3::new(1.0 / s.x, 1.0 / s.y, 1.0 / s.z));
        Transform {
            matrix,
            inverse: Some(inverse),
        }
    }

    /// Create uniform scale transform
    pub fn scale_uniform(s: f64) -> Self {
        let matrix = Mat4::scale_uniform(s);
        let inverse = Mat4::scale_uniform(1.0 / s);
        Transform {
            matrix,
            inverse: Some(inverse),
        }
    }

    /// Create look-at transform
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let matrix = Mat4::look_at(eye, target, up);
        Transform {
            matrix,
            inverse: None,
        }
    }

    /// Create TRS (translate-rotate-scale) transform
    pub fn from_trs(translation: Vec3, rotation: Quaternion, scale: Vec3) -> Self {
        let t = Mat4::translate(translation);
        let r = rotation.to_matrix();
        let s = Mat4::scale(scale);

        let matrix = t * r * s;

        Transform {
            matrix,
            inverse: None,
        }
    }

    /// Get the transformation matrix
    pub fn matrix(&self) -> &Mat4 {
        &self.matrix
    }

    /// Get the inverse matrix (computes if not cached)
    pub fn inverse_matrix(&mut self) -> Mat4 {
        if let Some(inv) = self.inverse {
            inv
        } else {
            let inv = self.matrix.inverse().expect("Matrix is not invertible");
            self.inverse = Some(inv);
            inv
        }
    }

    /// Get inverse transform
    pub fn inverse(&self) -> Transform {
        if let Some(inv_mat) = self.inverse {
            Transform {
                matrix: inv_mat,
                inverse: Some(self.matrix),
            }
        } else {
            let inv_mat = self.matrix.inverse().expect("Matrix is not invertible");
            Transform {
                matrix: inv_mat,
                inverse: Some(self.matrix),
            }
        }
    }

    /// Compose two transforms (self applied first, then other)
    pub fn then(&self, other: &Transform) -> Transform {
        Transform {
            matrix: other.matrix * self.matrix,
            inverse: None,
        }
    }

    /// Transform a point
    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        self.matrix.transform_point(p)
    }

    /// Transform a vector (ignores translation)
    pub fn transform_vector(&self, v: Vec3) -> Vec3 {
        self.matrix.transform_vector(v)
    }

    /// Transform a normal (uses inverse transpose)
    pub fn transform_normal(&mut self, n: Vec3) -> Vec3 {
        let inv = self.inverse_matrix();
        let inv_transpose = inv.transpose();
        inv_transpose.transform_vector(n).normalized()
    }

    /// Transform a ray
    pub fn transform_ray(&self, ray: &Ray) -> Ray {
        Ray::new(
            self.transform_point(ray.origin),
            self.transform_vector(ray.direction).normalized(),
        )
    }

    /// Transform an AABB
    pub fn transform_bounds(&self, bounds: &AABB) -> AABB {
        // Transform all 8 corners and compute new AABB
        let min = bounds.min();
        let max = bounds.max();

        let corners = [
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(min.x, max.y, min.z),
            Vec3::new(min.x, max.y, max.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(max.x, max.y, max.z),
        ];

        let mut new_min = self.transform_point(corners[0]);
        let mut new_max = new_min;

        for &corner in &corners[1..] {
            let p = self.transform_point(corner);
            new_min = Vec3::new(new_min.x.min(p.x), new_min.y.min(p.y), new_min.z.min(p.z));
            new_max = Vec3::new(new_max.x.max(p.x), new_max.y.max(p.y), new_max.z.max(p.z));
        }

        AABB::new(new_min, new_max)
    }

    /// Check if this is an identity transform
    pub fn is_identity(&self) -> bool {
        // Simple check: see if transforming a point doesn't change it
        let p = Vec3::new(1.0, 2.0, 3.0);
        let transformed = self.transform_point(p);
        (transformed - p).length_squared() < 1e-10
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform::identity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let t = Transform::identity();
        let p = Vec3::new(1.0, 2.0, 3.0);
        let result = t.transform_point(p);
        assert_eq!(result, p);
    }

    #[test]
    fn test_translate() {
        let t = Transform::translate(Vec3::new(1.0, 2.0, 3.0));
        let p = Vec3::ZERO;
        let result = t.transform_point(p);
        assert_eq!(result, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_scale() {
        let t = Transform::scale(Vec3::new(2.0, 3.0, 4.0));
        let p = Vec3::new(1.0, 1.0, 1.0);
        let result = t.transform_point(p);
        assert_eq!(result, Vec3::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn test_inverse() {
        let t = Transform::translate(Vec3::new(1.0, 2.0, 3.0));
        let inv = t.inverse();
        let p = Vec3::new(5.0, 6.0, 7.0);
        let transformed = t.transform_point(p);
        let back = inv.transform_point(transformed);
        assert!((back - p).length() < 1e-10);
    }

    #[test]
    fn test_compose() {
        let t1 = Transform::translate(Vec3::new(1.0, 0.0, 0.0));
        let t2 = Transform::scale_uniform(2.0);
        let composed = t1.then(&t2);

        let p = Vec3::new(1.0, 1.0, 1.0);
        let result = composed.transform_point(p);
        // First translate (1,1,1) -> (2,1,1), then scale (2,1,1) -> (4,2,2)
        assert_eq!(result, Vec3::new(4.0, 2.0, 2.0));
    }
}
