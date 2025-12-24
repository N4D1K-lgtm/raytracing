use super::{BxDF, BxDFSample, BxDFType};
use crate::core::math::Vec2;
use crate::vec3::Vec3;

/// Dielectric BSDF for glass, water, and other transparent materials
///
/// Implements:
/// - Fresnel equations (reflection probability)
/// - Snell's law (refraction direction)
/// - Total internal reflection
/// - Proper handling of entering/exiting medium
pub struct DielectricBxDF {
    pub ior: f64, // Index of refraction
}

impl DielectricBxDF {
    pub fn new(ior: f64) -> Self {
        DielectricBxDF {
            ior: ior.max(1.0), // IOR must be >= 1.0
        }
    }

    /// Compute Fresnel reflectance using Schlick's approximation
    fn fresnel_schlick(cos_theta: f64, eta: f64) -> f64 {
        let r0 = ((1.0 - eta) / (1.0 + eta)).powi(2);
        let t = (1.0 - cos_theta).max(0.0).min(1.0);
        let t2 = t * t;
        let t5 = t2 * t2 * t;
        r0 + (1.0 - r0) * t5
    }

    /// Compute exact Fresnel reflectance (unpolarized)
    fn fresnel_dielectric(cos_theta_i: f64, cos_theta_t: f64, eta: f64) -> f64 {
        let r_parallel = ((eta * cos_theta_i) - cos_theta_t) / ((eta * cos_theta_i) + cos_theta_t);
        let r_perpendicular = (cos_theta_i - (eta * cos_theta_t)) / (cos_theta_i + (eta * cos_theta_t));
        (r_parallel * r_parallel + r_perpendicular * r_perpendicular) / 2.0
    }

    /// Refract a vector according to Snell's law
    /// Returns None if total internal reflection occurs
    fn refract(wi: Vec3, n: Vec3, eta: f64) -> Option<Vec3> {
        let cos_theta_i = wi.dot(n);
        let sin2_theta_i = (1.0 - cos_theta_i * cos_theta_i).max(0.0);
        let sin2_theta_t = sin2_theta_i / (eta * eta);

        // Check for total internal reflection
        if sin2_theta_t >= 1.0 {
            return None;
        }

        let cos_theta_t = (1.0 - sin2_theta_t).sqrt();

        // Refracted direction
        Some((wi * (-1.0 / eta)) + n * (cos_theta_i / eta - cos_theta_t))
    }
}

impl BxDF for DielectricBxDF {
    fn bxdf_type(&self) -> BxDFType {
        BxDFType::Specular
    }

    fn f(&self, _wo: Vec3, _wi: Vec3) -> Vec3 {
        // Dielectric is a delta distribution (mirror reflection + refraction)
        // Cannot be evaluated for arbitrary directions
        Vec3::ZERO
    }

    fn sample_f(&self, wo: Vec3, u: Vec2) -> Option<BxDFSample> {
        // Determine if we're entering or exiting the medium
        let entering = wo.z > 0.0;

        // Set up surface normal and relative IOR
        let (n, eta) = if entering {
            // Entering: air (1.0) → medium (ior)
            (Vec3::new(0.0, 0.0, 1.0), 1.0 / self.ior)
        } else {
            // Exiting: medium (ior) → air (1.0)
            (Vec3::new(0.0, 0.0, -1.0), self.ior)
        };

        let cos_theta_i = wo.dot(n).abs();

        // Try to refract
        let refracted = Self::refract(wo, n, eta);

        let fresnel = if let Some(wt) = refracted {
            // Both reflection and refraction possible
            let cos_theta_t = wt.dot(-n).abs();
            Self::fresnel_dielectric(cos_theta_i, cos_theta_t, eta)
        } else {
            // Total internal reflection
            1.0
        };

        // Use random number to choose reflection vs refraction
        let reflect = u.x < fresnel;

        if reflect || refracted.is_none() {
            // Reflection
            let wi = Vec3::new(-wo.x, -wo.y, wo.z);

            Some(BxDFSample {
                wi,
                f: Vec3::new(1.0, 1.0, 1.0), // Perfect reflection
                pdf: 1.0,                     // Delta distribution
                bxdf_type: BxDFType::Specular,
            })
        } else {
            // Refraction
            let wi = refracted.unwrap();

            // Account for radiance scaling due to IOR change
            // When crossing interface, solid angle changes by eta^2
            let scale = if entering {
                // Entering: less solid angle compression
                1.0
            } else {
                // Exiting: solid angle expands, radiance decreases
                (1.0 / self.ior).powi(2)
            };

            Some(BxDFSample {
                wi,
                f: Vec3::new(scale, scale, scale),
                pdf: 1.0, // Delta distribution
                bxdf_type: BxDFType::Specular,
            })
        }
    }

    fn pdf(&self, _wo: Vec3, _wi: Vec3) -> f64 {
        // Delta distribution has zero PDF for any discrete direction
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dielectric_creation() {
        let glass = DielectricBxDF::new(1.5);
        assert_eq!(glass.ior, 1.5);

        // IOR should be clamped to >= 1.0
        let invalid = DielectricBxDF::new(0.5);
        assert!(invalid.ior >= 1.0);
    }

    #[test]
    fn test_fresnel_normal_incidence() {
        // At normal incidence (90°), some light reflects
        let fresnel = DielectricBxDF::fresnel_schlick(1.0, 1.0 / 1.5);
        assert!(fresnel > 0.0 && fresnel < 0.1); // ~4% for glass
    }

    #[test]
    fn test_fresnel_grazing_angle() {
        // At grazing angle (0°), almost all light reflects
        let fresnel = DielectricBxDF::fresnel_schlick(0.0, 1.0 / 1.5);
        assert!(fresnel > 0.9); // Close to 100%
    }

    #[test]
    fn test_refraction_snells_law() {
        let wi = Vec3::new(0.0, 0.0, 1.0); // Normal incidence
        let n = Vec3::new(0.0, 0.0, 1.0);
        let eta = 1.0 / 1.5;

        let wt = DielectricBxDF::refract(wi, n, eta);
        assert!(wt.is_some());

        // At normal incidence, direction shouldn't change much
        let wt = wt.unwrap();
        assert!((wt.z - (-1.0)).abs() < 0.01);
    }

    #[test]
    fn test_total_internal_reflection() {
        // Critical angle for glass (IOR=1.5) is ~41.8°
        // Beyond this, we get total internal reflection

        let glass = DielectricBxDF::new(1.5);

        // Steep angle from inside glass (should refract)
        let wo_steep = Vec3::new(0.2, 0.0, -0.98).normalized();
        let sample_steep = glass.sample_f(wo_steep, Vec2::new(0.0, 0.0));
        assert!(sample_steep.is_some());

        // Grazing angle from inside glass (should have TIR)
        let wo_grazing = Vec3::new(0.9, 0.0, -0.436).normalized();
        let sample_grazing = glass.sample_f(wo_grazing, Vec2::new(0.0, 0.0));

        if let Some(sample) = sample_grazing {
            // Should be reflected, not refracted
            // Reflected ray should be in negative hemisphere
            assert!(sample.wi.z < 0.0);
        }
    }

    #[test]
    fn test_dielectric_entering_exiting() {
        let glass = DielectricBxDF::new(1.5);

        // Ray entering glass (from above, wo.z > 0)
        let wo_enter = Vec3::new(0.3, 0.0, 0.95).normalized();
        let sample_enter = glass.sample_f(wo_enter, Vec2::new(0.9, 0.0)); // Force refraction

        assert!(sample_enter.is_some());
        if let Some(sample) = sample_enter {
            // Refracted ray should bend toward normal (z component larger)
            assert!(sample.wi.z < 0.0); // Should go into material
        }

        // Ray exiting glass (from below, wo.z < 0)
        let wo_exit = Vec3::new(0.3, 0.0, -0.95).normalized();
        let sample_exit = glass.sample_f(wo_exit, Vec2::new(0.9, 0.0)); // Force refraction

        assert!(sample_exit.is_some());
        if let Some(sample) = sample_exit {
            // Refracted ray should bend away from normal
            assert!(sample.wi.z > 0.0); // Should exit material
        }
    }

    #[test]
    fn test_dielectric_reflection_direction() {
        let glass = DielectricBxDF::new(1.5);
        let wo = Vec3::new(0.5, 0.3, 0.8).normalized();

        // Force reflection (u.x < fresnel, so use very small value)
        let sample = glass.sample_f(wo, Vec2::new(0.0, 0.0));

        assert!(sample.is_some());
        if let Some(s) = sample {
            // Check reflection: wi = reflect(wo)
            let expected = Vec3::new(-wo.x, -wo.y, wo.z).normalized();
            assert!((s.wi.x - expected.x).abs() < 1e-6);
            assert!((s.wi.y - expected.y).abs() < 1e-6);
            assert!((s.wi.z - expected.z).abs() < 1e-6);
        }
    }

    #[test]
    fn test_dielectric_is_specular() {
        let glass = DielectricBxDF::new(1.5);
        assert_eq!(glass.bxdf_type(), BxDFType::Specular);

        // Delta distributions have zero PDF
        let wo = Vec3::new(0.0, 0.0, 1.0);
        let wi = Vec3::new(0.0, 0.0, -1.0);
        assert_eq!(glass.pdf(wo, wi), 0.0);

        // f() should return zero (delta distribution)
        assert_eq!(glass.f(wo, wi), Vec3::ZERO);
    }
}
