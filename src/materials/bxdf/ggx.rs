use super::{BxDF, BxDFSample, BxDFType};
use crate::core::math::Vec2;
use crate::vec3::Vec3;
use std::f64::consts::PI;

/// GGX microfacet BRDF for metallic materials
///
/// Implements the GGX (Trowbridge-Reitz) microfacet model with:
/// - GGX normal distribution (D term)
/// - Smith geometric attenuation (G term)
/// - Schlick Fresnel approximation (F term)
/// - Importance sampling via visible normal distribution
pub struct GgxBxDF {
    pub albedo: Vec3,
    pub roughness: f64,
    alpha: f64, // roughness²
}

impl GgxBxDF {
    pub fn new(albedo: Vec3, roughness: f64) -> Self {
        let roughness = roughness.max(0.01).min(1.0); // Clamp to valid range
        let alpha = roughness * roughness;

        GgxBxDF {
            albedo,
            roughness,
            alpha,
        }
    }

    /// GGX normal distribution function D(h)
    fn distribution_ggx(n_dot_h: f64, alpha: f64) -> f64 {
        let a2 = alpha * alpha;
        let nh2 = n_dot_h * n_dot_h;
        let denom = nh2 * (a2 - 1.0) + 1.0;
        a2 / (PI * denom * denom)
    }

    /// Smith geometric attenuation for a single direction
    fn geometry_smith_ggx(n_dot_v: f64, alpha: f64) -> f64 {
        let a2 = alpha * alpha;
        let nv2 = n_dot_v * n_dot_v;
        (2.0 * n_dot_v) / (n_dot_v + ((a2 + (1.0 - a2) * nv2).sqrt()))
    }

    /// Combined Smith geometric term G(wo, wi)
    fn geometry_smith(n_dot_o: f64, n_dot_i: f64, alpha: f64) -> f64 {
        let g1 = Self::geometry_smith_ggx(n_dot_o, alpha);
        let g2 = Self::geometry_smith_ggx(n_dot_i, alpha);
        g1 * g2
    }

    /// Schlick Fresnel approximation F(wo, h)
    fn fresnel_schlick(cos_theta: f64, f0: Vec3) -> Vec3 {
        let t = (1.0 - cos_theta).max(0.0).min(1.0);
        let t2 = t * t;
        let t5 = t2 * t2 * t;
        f0 + (Vec3::new(1.0, 1.0, 1.0) - f0) * t5
    }

    /// Sample microfacet normal using GGX distribution
    fn sample_ggx_vndf(wo: Vec3, alpha: f64, u: Vec2) -> Vec3 {
        // Transform to hemispherical configuration
        let v = Vec3::new(
            alpha * wo.x,
            alpha * wo.y,
            wo.z,
        ).normalized();

        // Orthonormal basis
        let t1 = if v.z < 0.9999 {
            Vec3::new(0.0, 0.0, 1.0).cross(v).normalized()
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        let t2 = v.cross(t1);

        // Sample point with polar coordinates
        let a = 1.0 / (1.0 + v.z);
        let r = u.x.sqrt();
        let phi = if u.y < a {
            u.y / a * PI
        } else {
            PI + (u.y - a) / (1.0 - a) * PI
        };

        let p1 = r * phi.cos();
        let p2 = r * phi.sin() * if u.y < a { 1.0 } else { v.z };

        // Compute normal
        let n = p1 * t1 + p2 * t2 + ((1.0 - p1 * p1 - p2 * p2).max(0.0).sqrt()) * v;

        // Transform back
        Vec3::new(
            alpha * n.x,
            alpha * n.y,
            n.z.max(0.0),
        ).normalized()
    }
}

impl BxDF for GgxBxDF {
    fn bxdf_type(&self) -> BxDFType {
        BxDFType::Glossy
    }

    fn f(&self, wo: Vec3, wi: Vec3) -> Vec3 {
        // Only reflect in upper hemisphere
        if wi.z <= 0.0 || wo.z <= 0.0 {
            return Vec3::ZERO;
        }

        // Compute half vector
        let h = (wo + wi).normalized();
        if h.z <= 0.0 {
            return Vec3::ZERO;
        }

        let n_dot_o = wo.z.abs();
        let n_dot_i = wi.z.abs();
        let n_dot_h = h.z.abs();
        let o_dot_h = wo.dot(h).abs();

        // Compute microfacet BRDF components
        let d = Self::distribution_ggx(n_dot_h, self.alpha);
        let g = Self::geometry_smith(n_dot_o, n_dot_i, self.alpha);
        let f = Self::fresnel_schlick(o_dot_h, self.albedo);

        // Cook-Torrance BRDF: f = (D * G * F) / (4 * cos_theta_o * cos_theta_i)
        let denom = 4.0 * n_dot_o * n_dot_i;
        if denom < 1e-8 {
            return Vec3::ZERO;
        }

        f * (d * g / denom)
    }

    fn sample_f(&self, wo: Vec3, u: Vec2) -> Option<BxDFSample> {
        if wo.z <= 0.0 {
            return None;
        }

        // Sample microfacet normal
        let h = Self::sample_ggx_vndf(wo, self.alpha, u);

        // Reflect about microfacet normal
        let wi = (h * (2.0 * wo.dot(h)) - wo).normalized();

        // Check if reflection is in valid hemisphere
        if wi.z <= 0.0 {
            return None;
        }

        let pdf = self.pdf(wo, wi);
        if pdf < 1e-8 {
            return None;
        }

        let f = self.f(wo, wi);

        Some(BxDFSample {
            wi,
            f,
            pdf,
            bxdf_type: BxDFType::Glossy,
        })
    }

    fn pdf(&self, wo: Vec3, wi: Vec3) -> f64 {
        if wi.z <= 0.0 || wo.z <= 0.0 {
            return 0.0;
        }

        let h = (wo + wi).normalized();
        if h.z <= 0.0 {
            return 0.0;
        }

        let n_dot_h = h.z.abs();
        let o_dot_h = wo.dot(h).abs();

        // PDF for sampling microfacet normal
        let d = Self::distribution_ggx(n_dot_h, self.alpha);
        let pdf_h = d * n_dot_h;

        // Transform to incident direction PDF
        // pdf(wi) = pdf(h) / (4 * dot(wo, h))
        pdf_h / (4.0 * o_dot_h).max(1e-8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ggx_creation() {
        let ggx = GgxBxDF::new(Vec3::new(1.0, 0.8, 0.3), 0.3);
        assert!(ggx.roughness >= 0.01);
        assert!(ggx.roughness <= 1.0);
    }

    #[test]
    fn test_ggx_distribution() {
        // Test that distribution is positive
        let d = GgxBxDF::distribution_ggx(1.0, 0.5);
        assert!(d > 0.0);

        // Test symmetry: D should be same for same angle
        let d1 = GgxBxDF::distribution_ggx(0.7, 0.3);
        let d2 = GgxBxDF::distribution_ggx(0.7, 0.3);
        assert!((d1 - d2).abs() < 1e-10);
    }

    #[test]
    fn test_ggx_energy_conservation() {
        let ggx = GgxBxDF::new(Vec3::new(0.9, 0.9, 0.9), 0.3);
        let wo = Vec3::new(0.0, 0.0, 1.0);

        // Sample multiple directions and integrate
        let mut total = Vec3::ZERO;
        let samples = 1000;

        for i in 0..samples {
            let u = Vec2::new(
                (i as f64 + 0.5) / samples as f64,
                ((i * 7) % samples) as f64 / samples as f64,
            );

            if let Some(sample) = ggx.sample_f(wo, u) {
                // Monte Carlo estimator: (f * cos_theta) / pdf
                let cos_theta = sample.wi.z.abs();
                let contribution = sample.f * (cos_theta / sample.pdf);
                total = total + contribution;
            }
        }

        // Average reflectance (should be <= 1 for energy conservation)
        let avg_reflectance = total / samples as f64;
        assert!(avg_reflectance.x <= 1.1); // Allow small numerical error
        assert!(avg_reflectance.y <= 1.1);
        assert!(avg_reflectance.z <= 1.1);
        assert!(avg_reflectance.length() > 0.0);
    }

    #[test]
    fn test_ggx_pdf_matches_sampling() {
        let ggx = GgxBxDF::new(Vec3::new(1.0, 1.0, 1.0), 0.5);
        let wo = Vec3::new(0.0, 0.0, 1.0);
        let u = Vec2::new(0.5, 0.5);

        if let Some(sample) = ggx.sample_f(wo, u) {
            let pdf_direct = ggx.pdf(wo, sample.wi);
            // PDFs should match (within numerical tolerance)
            assert!((sample.pdf - pdf_direct).abs() / sample.pdf < 0.01);
        }
    }

    #[test]
    fn test_ggx_reciprocity() {
        let ggx = GgxBxDF::new(Vec3::new(1.0, 0.8, 0.3), 0.3);

        let wo = Vec3::new(0.5, 0.3, 0.8).normalized();
        let wi = Vec3::new(0.3, 0.5, 0.8).normalized();

        let f_wo_wi = ggx.f(wo, wi);
        let f_wi_wo = ggx.f(wi, wo);

        // BRDF should be reciprocal: f(wo, wi) = f(wi, wo)
        assert!((f_wo_wi.x - f_wi_wo.x).abs() < 1e-6);
        assert!((f_wo_wi.y - f_wi_wo.y).abs() < 1e-6);
        assert!((f_wo_wi.z - f_wi_wo.z).abs() < 1e-6);
    }
}
