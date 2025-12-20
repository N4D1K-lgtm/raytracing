use crate::vec3::Vec3;
use super::Mat4;
use std::ops::Mul;

/// Quaternion for representing rotations
/// Stored as (x, y, z, w) where w is the scalar part
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quaternion {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Quaternion {
    /// Identity quaternion (no rotation)
    pub const IDENTITY: Quaternion = Quaternion {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    /// Create quaternion from components
    #[inline]
    pub const fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Quaternion { x, y, z, w }
    }

    /// Create quaternion from axis-angle representation
    /// axis should be normalized, angle in radians
    pub fn from_axis_angle(axis: Vec3, angle: f64) -> Self {
        let half_angle = angle * 0.5;
        let s = half_angle.sin();
        Quaternion {
            x: axis.x * s,
            y: axis.y * s,
            z: axis.z * s,
            w: half_angle.cos(),
        }
    }

    /// Create quaternion from Euler angles (in radians)
    /// Order: ZYX (yaw, pitch, roll)
    pub fn from_euler(yaw: f64, pitch: f64, roll: f64) -> Self {
        let cy = (yaw * 0.5).cos();
        let sy = (yaw * 0.5).sin();
        let cp = (pitch * 0.5).cos();
        let sp = (pitch * 0.5).sin();
        let cr = (roll * 0.5).cos();
        let sr = (roll * 0.5).sin();

        Quaternion {
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
            w: cr * cp * cy + sr * sp * sy,
        }
    }

    /// Create quaternion from rotation matrix
    pub fn from_matrix(m: &Mat4) -> Self {
        let trace = m.get(0, 0) + m.get(1, 1) + m.get(2, 2);

        if trace > 0.0 {
            let s = (trace + 1.0).sqrt() * 2.0;
            Quaternion {
                w: 0.25 * s,
                x: (m.get(2, 1) - m.get(1, 2)) / s,
                y: (m.get(0, 2) - m.get(2, 0)) / s,
                z: (m.get(1, 0) - m.get(0, 1)) / s,
            }
        } else if m.get(0, 0) > m.get(1, 1) && m.get(0, 0) > m.get(2, 2) {
            let s = (1.0 + m.get(0, 0) - m.get(1, 1) - m.get(2, 2)).sqrt() * 2.0;
            Quaternion {
                w: (m.get(2, 1) - m.get(1, 2)) / s,
                x: 0.25 * s,
                y: (m.get(0, 1) + m.get(1, 0)) / s,
                z: (m.get(0, 2) + m.get(2, 0)) / s,
            }
        } else if m.get(1, 1) > m.get(2, 2) {
            let s = (1.0 + m.get(1, 1) - m.get(0, 0) - m.get(2, 2)).sqrt() * 2.0;
            Quaternion {
                w: (m.get(0, 2) - m.get(2, 0)) / s,
                x: (m.get(0, 1) + m.get(1, 0)) / s,
                y: 0.25 * s,
                z: (m.get(1, 2) + m.get(2, 1)) / s,
            }
        } else {
            let s = (1.0 + m.get(2, 2) - m.get(0, 0) - m.get(1, 1)).sqrt() * 2.0;
            Quaternion {
                w: (m.get(1, 0) - m.get(0, 1)) / s,
                x: (m.get(0, 2) + m.get(2, 0)) / s,
                y: (m.get(1, 2) + m.get(2, 1)) / s,
                z: 0.25 * s,
            }
        }
    }

    /// Convert quaternion to rotation matrix
    pub fn to_matrix(&self) -> Mat4 {
        let x2 = self.x * self.x;
        let y2 = self.y * self.y;
        let z2 = self.z * self.z;
        let xy = self.x * self.y;
        let xz = self.x * self.z;
        let yz = self.y * self.z;
        let wx = self.w * self.x;
        let wy = self.w * self.y;
        let wz = self.w * self.z;

        Mat4::from_cols(
            [1.0 - 2.0 * (y2 + z2), 2.0 * (xy + wz), 2.0 * (xz - wy), 0.0],
            [2.0 * (xy - wz), 1.0 - 2.0 * (x2 + z2), 2.0 * (yz + wx), 0.0],
            [2.0 * (xz + wy), 2.0 * (yz - wx), 1.0 - 2.0 * (x2 + y2), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        )
    }

    /// Length squared
    #[inline]
    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    /// Length
    #[inline]
    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    /// Normalize quaternion
    pub fn normalized(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            let inv_len = 1.0 / len;
            Quaternion {
                x: self.x * inv_len,
                y: self.y * inv_len,
                z: self.z * inv_len,
                w: self.w * inv_len,
            }
        } else {
            Quaternion::IDENTITY
        }
    }

    /// Conjugate quaternion (inverse for unit quaternions)
    #[inline]
    pub fn conjugate(&self) -> Self {
        Quaternion {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    /// Inverse quaternion
    pub fn inverse(&self) -> Self {
        let len_sq = self.length_squared();
        if len_sq > 0.0 {
            let inv_len_sq = 1.0 / len_sq;
            Quaternion {
                x: -self.x * inv_len_sq,
                y: -self.y * inv_len_sq,
                z: -self.z * inv_len_sq,
                w: self.w * inv_len_sq,
            }
        } else {
            Quaternion::IDENTITY
        }
    }

    /// Dot product
    #[inline]
    pub fn dot(&self, other: &Quaternion) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    /// Spherical linear interpolation
    pub fn slerp(&self, other: &Quaternion, t: f64) -> Self {
        let mut cos_theta = self.dot(other);

        // Take shorter path
        let other = if cos_theta < 0.0 {
            cos_theta = -cos_theta;
            Quaternion {
                x: -other.x,
                y: -other.y,
                z: -other.z,
                w: -other.w,
            }
        } else {
            *other
        };

        // Use linear interpolation for very close quaternions
        if cos_theta > 0.9995 {
            return Quaternion {
                x: self.x + t * (other.x - self.x),
                y: self.y + t * (other.y - self.y),
                z: self.z + t * (other.z - self.z),
                w: self.w + t * (other.w - self.w),
            }
            .normalized();
        }

        let theta = cos_theta.acos();
        let sin_theta = theta.sin();
        let a = ((1.0 - t) * theta).sin() / sin_theta;
        let b = (t * theta).sin() / sin_theta;

        Quaternion {
            x: a * self.x + b * other.x,
            y: a * self.y + b * other.y,
            z: a * self.z + b * other.z,
            w: a * self.w + b * other.w,
        }
    }

    /// Rotate a vector by this quaternion
    pub fn rotate_vector(&self, v: Vec3) -> Vec3 {
        // v' = q * v * q^-1
        // For unit quaternions, q^-1 = q*
        let qv = Quaternion::new(v.x, v.y, v.z, 0.0);
        let result = *self * qv * self.conjugate();
        Vec3::new(result.x, result.y, result.z)
    }
}

// Quaternion multiplication
impl Mul for Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Quaternion) -> Quaternion {
        Quaternion {
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let q = Quaternion::IDENTITY;
        let v = Vec3::new(1.0, 2.0, 3.0);
        let result = q.rotate_vector(v);
        assert!((result - v).length() < 1e-10);
    }

    #[test]
    fn test_axis_angle() {
        let q = Quaternion::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), std::f64::consts::PI / 2.0);
        let v = Vec3::new(1.0, 0.0, 0.0);
        let result = q.rotate_vector(v);
        let expected = Vec3::new(0.0, 1.0, 0.0);
        assert!((result - expected).length() < 1e-10);
    }

    #[test]
    fn test_normalize() {
        let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
        let n = q.normalized();
        assert!((n.length() - 1.0).abs() < 1e-10);
    }
}
