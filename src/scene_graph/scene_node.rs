use crate::core::math::Transform;
use crate::geometry::Primitive;
use crate::lights::Light;
use crate::material::Material;
use std::sync::Arc;

/// Unique identifier for scene nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// Content that a scene node can hold
#[derive(Clone)]
pub enum NodeContent {
    /// Empty node (for grouping/transforms only)
    Empty,

    /// Geometry with material
    Geometry {
        primitive: Arc<dyn Primitive>,
        material: Arc<dyn Material>,
    },

    /// Instance of another node (shares geometry, unique transform)
    Instance {
        template: NodeId,
        /// Optional material override
        override_material: Option<Arc<dyn Material>>,
    },

    /// Light source
    Light(Arc<dyn Light>),
}

/// Scene graph node with hierarchical transforms
pub struct SceneNode {
    /// Unique identifier
    pub id: NodeId,

    /// Node name (for debugging/selection)
    pub name: String,

    /// Local transform relative to parent
    pub local_transform: Transform,

    /// Cached world transform (local_transform * parent_world_transform)
    world_transform: Option<Transform>,

    /// Dirty flag - true if world transform needs recomputation
    dirty: bool,

    /// Parent node ID (None for root)
    pub parent: Option<NodeId>,

    /// Child node IDs
    pub children: Vec<NodeId>,

    /// Node content
    pub content: NodeContent,
}

impl SceneNode {
    /// Create a new empty node
    pub fn new(id: NodeId, name: String) -> Self {
        SceneNode {
            id,
            name,
            local_transform: Transform::identity(),
            world_transform: None,
            dirty: true,
            parent: None,
            children: Vec::new(),
            content: NodeContent::Empty,
        }
    }

    /// Create a geometry node
    pub fn new_geometry(
        id: NodeId,
        name: String,
        primitive: Arc<dyn Primitive>,
        material: Arc<dyn Material>,
    ) -> Self {
        let mut node = Self::new(id, name);
        node.content = NodeContent::Geometry { primitive, material };
        node
    }

    /// Create a light node
    pub fn new_light(id: NodeId, name: String, light: Arc<dyn Light>) -> Self {
        let mut node = Self::new(id, name);
        node.content = NodeContent::Light(light);
        node
    }

    /// Create an instance node
    pub fn new_instance(id: NodeId, name: String, template: NodeId) -> Self {
        let mut node = Self::new(id, name);
        node.content = NodeContent::Instance {
            template,
            override_material: None,
        };
        node
    }

    /// Set local transform
    pub fn set_local_transform(&mut self, transform: Transform) {
        self.local_transform = transform;
        self.mark_dirty();
    }

    /// Get world transform (computes if dirty)
    pub fn world_transform(&self) -> Option<&Transform> {
        self.world_transform.as_ref()
    }

    /// Mark this node and all descendants as dirty
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.world_transform = None;
    }

    /// Check if node needs transform update
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Update world transform given parent's world transform
    pub fn update_world_transform(&mut self, parent_world: Option<&Transform>) {
        if let Some(parent) = parent_world {
            self.world_transform = Some(self.local_transform.then(parent));
        } else {
            // Root node - world transform is same as local transform
            self.world_transform = Some(self.local_transform.clone());
        }
        self.dirty = false;
    }

    /// Add a child node ID
    pub fn add_child(&mut self, child_id: NodeId) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }

    /// Remove a child node ID
    pub fn remove_child(&mut self, child_id: NodeId) {
        self.children.retain(|&id| id != child_id);
    }

    /// Check if this is a geometry node
    pub fn is_geometry(&self) -> bool {
        matches!(self.content, NodeContent::Geometry { .. })
    }

    /// Check if this is a light node
    pub fn is_light(&self) -> bool {
        matches!(self.content, NodeContent::Light(_))
    }

    /// Check if this is an instance node
    pub fn is_instance(&self) -> bool {
        matches!(self.content, NodeContent::Instance { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = SceneNode::new(NodeId(0), "test".to_string());
        assert_eq!(node.id, NodeId(0));
        assert_eq!(node.name, "test");
        assert!(node.is_dirty());
    }

    #[test]
    fn test_transform_propagation() {
        let mut node = SceneNode::new(NodeId(0), "test".to_string());
        let parent_transform = Transform::translate(crate::vec3::Vec3::new(1.0, 0.0, 0.0));

        node.update_world_transform(Some(&parent_transform));

        assert!(!node.is_dirty());
        assert!(node.world_transform().is_some());
    }

    #[test]
    fn test_dirty_flag() {
        let mut node = SceneNode::new(NodeId(0), "test".to_string());
        node.update_world_transform(None);
        assert!(!node.is_dirty());

        node.mark_dirty();
        assert!(node.is_dirty());
        assert!(node.world_transform().is_none());
    }

    #[test]
    fn test_children() {
        let mut parent = SceneNode::new(NodeId(0), "parent".to_string());
        parent.add_child(NodeId(1));
        parent.add_child(NodeId(2));

        assert_eq!(parent.children.len(), 2);
        assert!(parent.children.contains(&NodeId(1)));

        parent.remove_child(NodeId(1));
        assert_eq!(parent.children.len(), 1);
        assert!(!parent.children.contains(&NodeId(1)));
    }
}
