use crate::{
    acceleration::BvhNode,
    core::{
        intersection::Intersection,
        math::{AABB, Transform},
    },
    geometry::{Primitive, TransformedPrimitive},
    lights::Light,
    materials::material::Material,
    ray::Ray,
    scene_graph::{NodeContent, NodeId, SceneNode},
};
use std::collections::HashMap;
use std::sync::Arc;

/// Scene representation with scene graph and spatial acceleration
pub struct Scene {
    // Scene graph storage
    nodes: HashMap<NodeId, SceneNode>,
    root_nodes: Vec<NodeId>,
    next_id: usize,

    // Spatial acceleration structure (built from scene graph)
    bvh: Option<BvhNode>,

    // Lights collection (extracted from scene graph)
    lights: Vec<Arc<dyn Light>>,

    // Dirty flag - true if scene graph changed and BVH needs rebuild
    dirty: bool,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            nodes: HashMap::new(),
            root_nodes: Vec::new(),
            next_id: 0,
            bvh: None,
            lights: Vec::new(),
            dirty: false,
        }
    }

    // ========== Scene Graph API ==========

    /// Add a node to the scene graph
    pub fn add_node(&mut self, node: SceneNode) -> NodeId {
        let id = node.id;
        let is_root = node.parent.is_none();

        self.nodes.insert(id, node);

        if is_root {
            self.root_nodes.push(id);
        }

        self.dirty = true;
        id
    }

    /// Create and add a geometry node
    pub fn add_geometry(
        &mut self,
        name: String,
        primitive: Arc<dyn Primitive>,
        material: Arc<dyn Material>,
        transform: Transform,
    ) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;

        let mut node = SceneNode::new_geometry(id, name, primitive, material);
        node.set_local_transform(transform);
        self.add_node(node)
    }

    /// Create and add a light node
    pub fn add_light(
        &mut self,
        name: String,
        light: Arc<dyn Light>,
        transform: Transform,
    ) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;

        let mut node = SceneNode::new_light(id, name, light);
        node.set_local_transform(transform);
        self.add_node(node)
    }

    /// Create and add an instance node
    pub fn add_instance(&mut self, name: String, template: NodeId, transform: Transform) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;

        let mut node = SceneNode::new_instance(id, name, template);
        node.set_local_transform(transform);
        self.add_node(node)
    }

    /// Set parent-child relationship between nodes
    pub fn set_parent(&mut self, child_id: NodeId, parent_id: NodeId) {
        // Remove child from root nodes if present
        self.root_nodes.retain(|&id| id != child_id);

        // Update child's parent
        if let Some(child) = self.nodes.get_mut(&child_id) {
            child.parent = Some(parent_id);
            child.mark_dirty();
        }

        // Add child to parent's children list
        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            parent.add_child(child_id);
        }

        self.dirty = true;
    }

    /// Update transforms in scene graph
    pub fn update_transforms(&mut self) {
        // Update transforms for all root nodes and propagate to children
        for &root_id in self.root_nodes.clone().iter() {
            self.update_node_transform(root_id, None);
        }
    }

    fn update_node_transform(&mut self, node_id: NodeId, parent_world: Option<&Transform>) {
        // Get node info before recursive calls to avoid borrow conflicts
        let (should_update, children, world_transform) = {
            if let Some(node) = self.nodes.get_mut(&node_id) {
                let should_update = node.is_dirty();
                if should_update {
                    node.update_world_transform(parent_world);
                }
                let children = node.children.clone();
                let world_transform = node.world_transform().cloned();
                (should_update, children, world_transform)
            } else {
                return;
            }
        };

        // Recursively update children
        for &child_id in &children {
            self.update_node_transform(child_id, world_transform.as_ref());
        }
    }

    /// Build BVH from scene graph
    pub fn build_bvh(&mut self) {
        if !self.dirty && self.bvh.is_some() {
            return;
        }

        // Update all transforms first
        self.update_transforms();

        // Collect all geometry with transforms
        let mut primitives: Vec<Arc<dyn Primitive>> = Vec::new();
        self.lights.clear();

        // Process all nodes
        for node in self.nodes.values() {
            match &node.content {
                NodeContent::Geometry { primitive, .. } => {
                    // Apply world transform to primitive
                    if let Some(world_transform) = node.world_transform() {
                        let transformed =
                            TransformedPrimitive::new(primitive.clone(), world_transform.clone());
                        primitives.push(Arc::new(transformed));
                    } else {
                        // No transform - use primitive directly
                        primitives.push(primitive.clone());
                    }
                }
                NodeContent::Light(light) => {
                    // TODO: Apply transform to light
                    self.lights.push(light.clone());
                }
                NodeContent::Instance {
                    template,
                    override_material,
                } => {
                    // Resolve instance: get template geometry and apply instance transform
                    if let Some(template_node) = self.nodes.get(template) {
                        if let NodeContent::Geometry {
                            primitive,
                            material,
                        } = &template_node.content
                        {
                            let instance_material = override_material.as_ref().unwrap_or(material);

                            // Combine template's world transform with instance's world transform
                            if let Some(instance_world) = node.world_transform() {
                                // TODO: Create new primitive with instance material if overridden
                                let transformed = TransformedPrimitive::new(
                                    primitive.clone(),
                                    instance_world.clone(),
                                );
                                primitives.push(Arc::new(transformed));
                            }
                        }
                    }
                }
                NodeContent::Empty => {}
            }
        }

        // Build BVH if we have primitives
        if !primitives.is_empty() {
            self.bvh = BvhNode::build(primitives);
        } else {
            self.bvh = None;
        }

        self.dirty = false;
    }

    /// Intersect ray with scene using new Primitive API
    pub fn intersect(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<Intersection> {
        if let Some(bvh) = &self.bvh {
            bvh.intersect(ray, t_min, t_max)
        } else {
            None
        }
    }

    /// Fast intersection test (for shadow rays)
    pub fn intersect_p(&self, ray: &Ray, t_min: f64, t_max: f64) -> bool {
        if let Some(bvh) = &self.bvh {
            bvh.intersect_p(ray, t_min, t_max)
        } else {
            false
        }
    }

    /// Get all lights in scene
    pub fn lights(&self) -> &[Arc<dyn Light>] {
        &self.lights
    }

    /// Get a node by ID
    pub fn get_node(&self, id: NodeId) -> Option<&SceneNode> {
        self.nodes.get(&id)
    }

    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut SceneNode> {
        self.dirty = true;
        self.nodes.get_mut(&id)
    }

    /// Check if scene graph needs BVH rebuild
    pub fn needs_rebuild(&self) -> bool {
        self.dirty
    }

    /// Clear all objects
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.root_nodes.clear();
        self.next_id = 0;
        self.bvh = None;
        self.lights.clear();
        self.dirty = false;
    }

    /// Check if scene is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Sphere;
    use crate::materials::material::DiffuseMaterial;
    use crate::vec3::Vec3;

    #[test]
    fn test_scene_creation() {
        let scene = Scene::new();
        assert!(scene.is_empty());
        assert_eq!(scene.lights().len(), 0);
    }

    #[test]
    fn test_add_geometry_node() {
        let mut scene = Scene::new();
        let material: Arc<dyn Material> = Arc::new(DiffuseMaterial::new(Vec3::new(0.5, 0.5, 0.5)));
        let primitive: Arc<dyn Primitive> = Arc::new(Sphere::new(1.0, material.clone()));

        let node_id = scene.add_geometry(
            "test_sphere".to_string(),
            primitive,
            material,
            Transform::identity(),
        );

        assert_eq!(node_id, NodeId(0));
        assert_eq!(scene.nodes.len(), 1);
        assert!(scene.needs_rebuild());
    }

    #[test]
    fn test_scene_graph_hierarchy() {
        let mut scene = Scene::new();
        let material: Arc<dyn Material> = Arc::new(DiffuseMaterial::new(Vec3::new(0.5, 0.5, 0.5)));
        let primitive: Arc<dyn Primitive> = Arc::new(Sphere::new(1.0, material.clone()));

        // Create parent node
        let parent_id = scene.add_geometry(
            "parent".to_string(),
            primitive.clone(),
            material.clone(),
            Transform::translate(Vec3::new(1.0, 0.0, 0.0)),
        );

        // Create child node
        let child_id = scene.add_geometry(
            "child".to_string(),
            primitive,
            material,
            Transform::translate(Vec3::new(0.0, 1.0, 0.0)),
        );

        // Set parent-child relationship
        scene.set_parent(child_id, parent_id);

        assert_eq!(scene.root_nodes.len(), 1);
        assert_eq!(scene.root_nodes[0], parent_id);

        let parent = scene.get_node(parent_id).unwrap();
        assert!(parent.children.contains(&child_id));
    }

    #[test]
    fn test_build_bvh() {
        let mut scene = Scene::new();
        let material: Arc<dyn Material> = Arc::new(DiffuseMaterial::new(Vec3::new(0.5, 0.5, 0.5)));
        let primitive: Arc<dyn Primitive> = Arc::new(Sphere::new(1.0, material.clone()));

        scene.add_geometry(
            "sphere1".to_string(),
            primitive.clone(),
            material.clone(),
            Transform::identity(),
        );

        scene.add_geometry(
            "sphere2".to_string(),
            primitive,
            material,
            Transform::translate(Vec3::new(3.0, 0.0, 0.0)),
        );

        scene.build_bvh();

        assert!(scene.bvh.is_some());
        assert!(!scene.needs_rebuild());
    }

    #[test]
    fn test_intersect() {
        let mut scene = Scene::new();
        let material: Arc<dyn Material> = Arc::new(DiffuseMaterial::new(Vec3::new(0.5, 0.5, 0.5)));
        let primitive: Arc<dyn Primitive> = Arc::new(Sphere::new(1.0, material.clone()));

        scene.add_geometry(
            "sphere".to_string(),
            primitive,
            material,
            Transform::identity(),
        );

        scene.build_bvh();

        // Ray hitting the sphere
        let ray = Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        let hit = scene.intersect(&ray, 0.001, f64::INFINITY);
        assert!(hit.is_some());

        // Ray missing the sphere
        let ray = Ray::new(Vec3::new(10.0, 10.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        let hit = scene.intersect(&ray, 0.001, f64::INFINITY);
        assert!(hit.is_none());
    }
}
