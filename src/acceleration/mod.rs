mod bvh;

pub use bvh::{BvhNode, build_bvh};

// Re-export old AABB for compatibility
pub use crate::core::math::AABB;
