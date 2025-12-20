pub mod bxdf;
pub mod material;

// Re-export for convenience
pub use bxdf::{BxDF, BxDFType};
pub use material::{DiffuseMaterial, EmissiveMaterial, Material, PbrMaterial};
