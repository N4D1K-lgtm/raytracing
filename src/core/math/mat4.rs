use crate::vec3::Vec3;
use std::ops::Mul;

/// 4x4 transformation matrix stored in column-major order
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 {
    // Column-major storage: m[column][row]
    m: [[f64; 4]; 4],
}

impl Mat4 {
    /// Create identity matrix
    pub const fn identity() -> Self {
        Mat4 {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Create zero matrix
    pub const fn zero() -> Self {
        Mat4 {
            m: [
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
            ],
        }
    }

    /// Create matrix from columns
    pub const fn from_cols(c0: [f64; 4], c1: [f64; 4], c2: [f64; 4], c3: [f64; 4]) -> Self {
        Mat4 { m: [c0, c1, c2, c3] }
    }

    /// Create translation matrix
    pub fn translate(v: Vec3) -> Self {
        Mat4::from_cols(
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [v.x, v.y, v.z, 1.0],
        )
    }

    /// Create uniform scale matrix
    pub fn scale_uniform(s: f64) -> Self {
        Mat4::from_cols(
            [s, 0.0, 0.0, 0.0],
            [0.0, s, 0.0, 0.0],
            [0.0, 0.0, s, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        )
    }

    /// Create non-uniform scale matrix
    pub fn scale(s: Vec3) -> Self {
        Mat4::from_cols(
            [s.x, 0.0, 0.0, 0.0],
            [0.0, s.y, 0.0, 0.0],
            [0.0, 0.0, s.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        )
    }

    /// Create rotation matrix around X axis (angle in radians)
    pub fn rotate_x(angle: f64) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Mat4::from_cols(
            [1.0, 0.0, 0.0, 0.0],
            [0.0, c, s, 0.0],
            [0.0, -s, c, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        )
    }

    /// Create rotation matrix around Y axis (angle in radians)
    pub fn rotate_y(angle: f64) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Mat4::from_cols(
            [c, 0.0, -s, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [s, 0.0, c, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        )
    }

    /// Create rotation matrix around Z axis (angle in radians)
    pub fn rotate_z(angle: f64) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Mat4::from_cols(
            [c, s, 0.0, 0.0],
            [-s, c, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        )
    }

    /// Create rotation matrix around arbitrary axis (axis must be normalized, angle in radians)
    pub fn rotate_axis(axis: Vec3, angle: f64) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        let t = 1.0 - c;
        let x = axis.x;
        let y = axis.y;
        let z = axis.z;

        Mat4::from_cols(
            [t * x * x + c, t * x * y + s * z, t * x * z - s * y, 0.0],
            [t * x * y - s * z, t * y * y + c, t * y * z + s * x, 0.0],
            [t * x * z + s * y, t * y * z - s * x, t * z * z + c, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        )
    }

    /// Create look-at view matrix
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let f = (target - eye).normalized();
        let r = f.cross(up).normalized();
        let u = r.cross(f);

        Mat4::from_cols(
            [r.x, u.x, -f.x, 0.0],
            [r.y, u.y, -f.y, 0.0],
            [r.z, u.z, -f.z, 0.0],
            [-r.dot(eye), -u.dot(eye), f.dot(eye), 1.0],
        )
    }

    /// Get element at row i, column j
    #[inline]
    pub fn get(&self, row: usize, col: usize) -> f64 {
        self.m[col][row]
    }

    /// Set element at row i, column j
    #[inline]
    pub fn set(&mut self, row: usize, col: usize, value: f64) {
        self.m[col][row] = value;
    }

    /// Transpose matrix
    pub fn transpose(&self) -> Mat4 {
        let mut result = Mat4::zero();
        for i in 0..4 {
            for j in 0..4 {
                result.m[i][j] = self.m[j][i];
            }
        }
        result
    }

    /// Compute determinant
    pub fn determinant(&self) -> f64 {
        let m = &self.m;

        // Compute 2x2 determinants
        let s0 = m[0][0] * m[1][1] - m[1][0] * m[0][1];
        let s1 = m[0][0] * m[1][2] - m[1][0] * m[0][2];
        let s2 = m[0][0] * m[1][3] - m[1][0] * m[0][3];
        let s3 = m[0][1] * m[1][2] - m[1][1] * m[0][2];
        let s4 = m[0][1] * m[1][3] - m[1][1] * m[0][3];
        let s5 = m[0][2] * m[1][3] - m[1][2] * m[0][3];

        let c5 = m[2][2] * m[3][3] - m[3][2] * m[2][3];
        let c4 = m[2][1] * m[3][3] - m[3][1] * m[2][3];
        let c3 = m[2][1] * m[3][2] - m[3][1] * m[2][2];
        let c2 = m[2][0] * m[3][3] - m[3][0] * m[2][3];
        let c1 = m[2][0] * m[3][2] - m[3][0] * m[2][2];
        let c0 = m[2][0] * m[3][1] - m[3][0] * m[2][1];

        s0 * c5 - s1 * c4 + s2 * c3 + s3 * c2 - s4 * c1 + s5 * c0
    }

    /// Compute inverse matrix, returns None if matrix is singular
    pub fn inverse(&self) -> Option<Mat4> {
        let m = &self.m;

        // Compute 2x2 determinants for first two columns
        let s0 = m[0][0] * m[1][1] - m[1][0] * m[0][1];
        let s1 = m[0][0] * m[1][2] - m[1][0] * m[0][2];
        let s2 = m[0][0] * m[1][3] - m[1][0] * m[0][3];
        let s3 = m[0][1] * m[1][2] - m[1][1] * m[0][2];
        let s4 = m[0][1] * m[1][3] - m[1][1] * m[0][3];
        let s5 = m[0][2] * m[1][3] - m[1][2] * m[0][3];

        // Compute 2x2 determinants for last two columns
        let c5 = m[2][2] * m[3][3] - m[3][2] * m[2][3];
        let c4 = m[2][1] * m[3][3] - m[3][1] * m[2][3];
        let c3 = m[2][1] * m[3][2] - m[3][1] * m[2][2];
        let c2 = m[2][0] * m[3][3] - m[3][0] * m[2][3];
        let c1 = m[2][0] * m[3][2] - m[3][0] * m[2][2];
        let c0 = m[2][0] * m[3][1] - m[3][0] * m[2][1];

        // Compute determinant
        let det = s0 * c5 - s1 * c4 + s2 * c3 + s3 * c2 - s4 * c1 + s5 * c0;

        if det.abs() < 1e-10 {
            return None;
        }

        let inv_det = 1.0 / det;

        // Compute adjugate matrix and divide by determinant
        let mut inv = Mat4::zero();

        inv.m[0][0] = (m[1][1] * c5 - m[1][2] * c4 + m[1][3] * c3) * inv_det;
        inv.m[0][1] = (-m[0][1] * c5 + m[0][2] * c4 - m[0][3] * c3) * inv_det;
        inv.m[0][2] = (m[3][1] * s5 - m[3][2] * s4 + m[3][3] * s3) * inv_det;
        inv.m[0][3] = (-m[2][1] * s5 + m[2][2] * s4 - m[2][3] * s3) * inv_det;

        inv.m[1][0] = (-m[1][0] * c5 + m[1][2] * c2 - m[1][3] * c1) * inv_det;
        inv.m[1][1] = (m[0][0] * c5 - m[0][2] * c2 + m[0][3] * c1) * inv_det;
        inv.m[1][2] = (-m[3][0] * s5 + m[3][2] * s2 - m[3][3] * s1) * inv_det;
        inv.m[1][3] = (m[2][0] * s5 - m[2][2] * s2 + m[2][3] * s1) * inv_det;

        inv.m[2][0] = (m[1][0] * c4 - m[1][1] * c2 + m[1][3] * c0) * inv_det;
        inv.m[2][1] = (-m[0][0] * c4 + m[0][1] * c2 - m[0][3] * c0) * inv_det;
        inv.m[2][2] = (m[3][0] * s4 - m[3][1] * s2 + m[3][3] * s0) * inv_det;
        inv.m[2][3] = (-m[2][0] * s4 + m[2][1] * s2 - m[2][3] * s0) * inv_det;

        inv.m[3][0] = (-m[1][0] * c3 + m[1][1] * c1 - m[1][2] * c0) * inv_det;
        inv.m[3][1] = (m[0][0] * c3 - m[0][1] * c1 + m[0][2] * c0) * inv_det;
        inv.m[3][2] = (-m[3][0] * s3 + m[3][1] * s1 - m[3][2] * s0) * inv_det;
        inv.m[3][3] = (m[2][0] * s3 - m[2][1] * s1 + m[2][2] * s0) * inv_det;

        Some(inv)
    }

    /// Transform a point (applies full transformation including translation)
    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        let x = self.m[0][0] * p.x + self.m[1][0] * p.y + self.m[2][0] * p.z + self.m[3][0];
        let y = self.m[0][1] * p.x + self.m[1][1] * p.y + self.m[2][1] * p.z + self.m[3][1];
        let z = self.m[0][2] * p.x + self.m[1][2] * p.y + self.m[2][2] * p.z + self.m[3][2];
        let w = self.m[0][3] * p.x + self.m[1][3] * p.y + self.m[2][3] * p.z + self.m[3][3];

        if w != 0.0 && w != 1.0 {
            Vec3::new(x / w, y / w, z / w)
        } else {
            Vec3::new(x, y, z)
        }
    }

    /// Transform a vector (ignores translation component)
    pub fn transform_vector(&self, v: Vec3) -> Vec3 {
        let x = self.m[0][0] * v.x + self.m[1][0] * v.y + self.m[2][0] * v.z;
        let y = self.m[0][1] * v.x + self.m[1][1] * v.y + self.m[2][1] * v.z;
        let z = self.m[0][2] * v.x + self.m[1][2] * v.y + self.m[2][2] * v.z;
        Vec3::new(x, y, z)
    }

    /// Transform a normal (uses inverse transpose for correct normal transformation)
    /// Note: Caller should provide the inverse transpose matrix for efficiency
    pub fn transform_normal(&self, n: Vec3) -> Vec3 {
        // For normals, we need (M^-1)^T * n
        // Since we're passing in the already inverted-transposed matrix, just multiply
        self.transform_vector(n)
    }
}

// Matrix multiplication
impl Mul for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Mat4 {
        let mut result = Mat4::zero();
        for i in 0..4 {
            for j in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += self.m[k][i] * rhs.m[j][k];
                }
                result.m[j][i] = sum;
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let m = Mat4::identity();
        let v = Vec3::new(1.0, 2.0, 3.0);
        let result = m.transform_point(v);
        assert_eq!(result, v);
    }

    #[test]
    fn test_translation() {
        let m = Mat4::translate(Vec3::new(1.0, 2.0, 3.0));
        let v = Vec3::new(0.0, 0.0, 0.0);
        let result = m.transform_point(v);
        assert_eq!(result, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_scale() {
        let m = Mat4::scale(Vec3::new(2.0, 3.0, 4.0));
        let v = Vec3::new(1.0, 1.0, 1.0);
        let result = m.transform_point(v);
        assert_eq!(result, Vec3::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn test_inverse() {
        let m = Mat4::translate(Vec3::new(1.0, 2.0, 3.0));
        let inv = m.inverse().unwrap();
        let result = m * inv;

        // Should be close to identity
        for i in 0..4 {
            for j in 0..4 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((result.get(i, j) - expected).abs() < 1e-10);
            }
        }
    }
}
