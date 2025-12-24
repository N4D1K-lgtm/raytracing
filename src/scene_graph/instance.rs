use super::NodeId;
use crate::core::math::Transform;
use crate::materials::material::Material;
use std::sync::Arc;

/// Instance data for scene nodes
/// Instances share geometry but have unique transforms and optional material overrides
pub struct Instance {
    /// The template node being instanced
    pub template: NodeId,

    /// Instance-specific transform (applied after template transform)
    pub transform: Transform,

    /// Optional material override (uses template material if None)
    pub override_material: Option<Arc<dyn Material>>,
}

impl Instance {
    /// Create a new instance
    pub fn new(template: NodeId, transform: Transform) -> Self {
        Instance {
            template,
            transform,
            override_material: None,
        }
    }

    /// Create instance with material override
    pub fn with_material(mut self, material: Arc<dyn Material>) -> Self {
        self.override_material = Some(material);
        self
    }
}
