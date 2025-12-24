use crate::core::intersection::Intersection;
use crate::textures::Texture;
use crate::vec3::Vec3;
use std::sync::Arc;

use super::bxdf::{BxDF, DielectricBxDF, GgxBxDF, LambertianBxDF};

/// Material trait for new rendering system
/// Materials evaluate surface properties and provide BxDF for light transport
pub trait Material: Send + Sync {
    /// Compute the BSDF at an intersection point
    /// Returns a BxDF that describes how light scatters at this point
    fn compute_scattering_functions(&self, isect: &Intersection) -> Box<dyn BxDF>;

    /// Check if material emits light
    fn is_emissive(&self) -> bool {
        false
    }

    /// Get emitted radiance (for emissive materials)
    fn emitted(&self, _isect: &Intersection) -> Vec3 {
        Vec3::ZERO
    }
}

/// Physically-Based Rendering material with texture support
pub struct PbrMaterial {
    /// Base color / albedo texture
    pub albedo: Arc<dyn Texture>,

    /// Metallic factor (0 = dielectric, 1 = metal)
    pub metallic: Arc<dyn Texture>,

    /// Roughness factor (0 = smooth, 1 = rough)
    pub roughness: Arc<dyn Texture>,

    /// Index of refraction (for dielectrics)
    pub ior: f64,

    /// Optional emissive texture
    pub emissive: Option<Arc<dyn Texture>>,
}

impl PbrMaterial {
    /// Create a new PBR material with constant values
    pub fn new(albedo: Vec3, metallic: f64, roughness: f64) -> Self {
        use crate::textures::ConstantTexture;

        PbrMaterial {
            albedo: Arc::new(ConstantTexture::new(albedo)),
            metallic: Arc::new(ConstantTexture::new(Vec3::new(
                metallic, metallic, metallic,
            ))),
            roughness: Arc::new(ConstantTexture::new(Vec3::new(
                roughness, roughness, roughness,
            ))),
            ior: 1.5,
            emissive: None,
        }
    }

    /// Create a PBR material with textures
    pub fn with_textures(
        albedo: Arc<dyn Texture>,
        metallic: Arc<dyn Texture>,
        roughness: Arc<dyn Texture>,
    ) -> Self {
        PbrMaterial {
            albedo,
            metallic,
            roughness,
            ior: 1.5,
            emissive: None,
        }
    }

    /// Set index of refraction
    pub fn with_ior(mut self, ior: f64) -> Self {
        self.ior = ior;
        self
    }

    /// Set emissive texture
    pub fn with_emissive(mut self, emissive: Arc<dyn Texture>) -> Self {
        self.emissive = Some(emissive);
        self
    }
}

impl Material for PbrMaterial {
    fn compute_scattering_functions(&self, isect: &Intersection) -> Box<dyn BxDF> {
        // Evaluate textures at intersection point
        let albedo = self.albedo.evaluate(isect.uv);
        let metallic_value = self.metallic.evaluate(isect.uv).x.clamp(0.0, 1.0);
        let roughness_value = self.roughness.evaluate(isect.uv).x.clamp(0.01, 1.0);

        // For now, use simple material selection based on metallic value
        // TODO: Implement proper material layering
        if metallic_value > 0.5 {
            // Metallic material - use GGX with albedo as specular color
            Box::new(GgxBxDF::new(albedo, roughness_value))
        } else if roughness_value < 0.1 {
            // Smooth dielectric - use Dielectric BxDF
            Box::new(DielectricBxDF::new(self.ior))
        } else {
            // Diffuse material - use Lambertian
            Box::new(LambertianBxDF::new(albedo))
        }
    }

    fn is_emissive(&self) -> bool {
        self.emissive.is_some()
    }

    fn emitted(&self, isect: &Intersection) -> Vec3 {
        if let Some(ref emissive) = self.emissive {
            emissive.evaluate(isect.uv)
        } else {
            Vec3::ZERO
        }
    }
}

/// Simple Lambertian diffuse material (for compatibility)
pub struct DiffuseMaterial {
    pub albedo: Arc<dyn Texture>,
}

impl DiffuseMaterial {
    pub fn new(albedo: Vec3) -> Self {
        use crate::textures::ConstantTexture;
        DiffuseMaterial {
            albedo: Arc::new(ConstantTexture::new(albedo)),
        }
    }

    pub fn with_texture(albedo: Arc<dyn Texture>) -> Self {
        DiffuseMaterial { albedo }
    }
}

impl Material for DiffuseMaterial {
    fn compute_scattering_functions(&self, isect: &Intersection) -> Box<dyn BxDF> {
        let albedo = self.albedo.evaluate(isect.uv);
        Box::new(LambertianBxDF::new(albedo))
    }
}

/// Emissive material (area lights)
pub struct EmissiveMaterial {
    pub emission: Arc<dyn Texture>,
    pub intensity: f64,
}

impl EmissiveMaterial {
    pub fn new(color: Vec3, intensity: f64) -> Self {
        use crate::textures::ConstantTexture;
        EmissiveMaterial {
            emission: Arc::new(ConstantTexture::new(color)),
            intensity,
        }
    }
}

impl Material for EmissiveMaterial {
    fn compute_scattering_functions(&self, _isect: &Intersection) -> Box<dyn BxDF> {
        // Emissive materials don't scatter light
        Box::new(LambertianBxDF::new(Vec3::ZERO))
    }

    fn is_emissive(&self) -> bool {
        true
    }

    fn emitted(&self, isect: &Intersection) -> Vec3 {
        self.emission.evaluate(isect.uv) * self.intensity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbr_material_creation() {
        let mat = PbrMaterial::new(Vec3::new(0.8, 0.8, 0.8), 0.0, 0.5);
        assert!(!mat.is_emissive());
    }

    #[test]
    fn test_diffuse_material() {
        let mat = DiffuseMaterial::new(Vec3::new(0.5, 0.5, 0.5));
        assert!(!mat.is_emissive());
    }

    #[test]
    fn test_emissive_material() {
        let mat = EmissiveMaterial::new(Vec3::new(1.0, 1.0, 1.0), 5.0);
        assert!(mat.is_emissive());
    }
}
