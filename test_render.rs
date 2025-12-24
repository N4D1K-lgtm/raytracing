/// Simple test to verify the new rendering system works
use raytracing::{
    camera::Camera,
    core::math::Transform,
    geometry::{Primitive, Sphere},
    materials::material::DiffuseMaterial,
    renderer::Renderer,
    scene::Scene,
    vec3::Vec3,
};
use std::sync::Arc;

fn main() {
    println!("Testing new rendering system...\n");

    // Create scene
    let mut scene = Scene::new();

    // Add a simple sphere
    let material: Arc<dyn raytracing::materials::material::Material> =
        Arc::new(DiffuseMaterial::new(Vec3::new(0.8, 0.3, 0.3)));

    let sphere: Arc<dyn Primitive> = Arc::new(Sphere::new(1.0, material.clone()));

    scene.add_geometry(
        "TestSphere".to_string(),
        sphere,
        material,
        Transform::translate(Vec3::new(0.0, 0.0, -5.0)),
    );

    // Build BVH
    scene.build_bvh();

    // Create renderer
    let camera = Camera::new(16.0 / 9.0);
    let renderer = Renderer::new(400, 225)
        .with_samples(10)
        .with_max_depth(10)
        .with_camera(camera);

    // Render
    renderer.render(&scene, "test_output.ppm");

    println!("\nTest complete! Check test_output.ppm");
}
