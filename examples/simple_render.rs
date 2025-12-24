use raytracing::{
    camera::Camera,
    core::math::Transform,
    geometry::{Primitive, Sphere},
    lights::{Light, PointLight},
    materials::material::DiffuseMaterial,
    renderer::Renderer,
    scene::Scene,
    vec3::Vec3,
};
use std::sync::Arc;

fn main() {
    println!("=== Simple Render Example ===\n");

    // Create scene
    let mut scene = Scene::new();

    // Create materials
    let red_material: Arc<dyn raytracing::materials::material::Material> =
        Arc::new(DiffuseMaterial::new(Vec3::new(0.8, 0.2, 0.2)));

    let blue_material: Arc<dyn raytracing::materials::material::Material> =
        Arc::new(DiffuseMaterial::new(Vec3::new(0.2, 0.4, 0.8)));

    let green_material: Arc<dyn raytracing::materials::material::Material> =
        Arc::new(DiffuseMaterial::new(Vec3::new(0.3, 0.8, 0.3)));

    // Add spheres
    println!("Adding geometry...");

    let sphere1: Arc<dyn Primitive> = Arc::new(Sphere::new(0.5, red_material.clone()));
    scene.add_geometry(
        "RedSphere".to_string(),
        sphere1,
        red_material,
        Transform::translate(Vec3::new(-1.2, 0.0, -5.0)),
    );

    let sphere2: Arc<dyn Primitive> = Arc::new(Sphere::new(0.5, blue_material.clone()));
    scene.add_geometry(
        "BlueSphere".to_string(),
        sphere2,
        blue_material,
        Transform::translate(Vec3::new(0.0, 0.0, -5.0)),
    );

    let sphere3: Arc<dyn Primitive> = Arc::new(Sphere::new(0.5, green_material.clone()));
    scene.add_geometry(
        "GreenSphere".to_string(),
        sphere3,
        green_material,
        Transform::translate(Vec3::new(1.2, 0.0, -5.0)),
    );

    // Add lights
    println!("Adding lights...");

    let light1 = Arc::new(PointLight::new(
        Vec3::new(-2.0, 2.0, -3.0),
        Vec3::new(10.0, 10.0, 10.0),
    )) as Arc<dyn Light>;

    scene.add_light("KeyLight".to_string(), light1, Transform::identity());

    let light2 = Arc::new(PointLight::new(
        Vec3::new(2.0, 1.0, -3.0),
        Vec3::new(5.0, 5.0, 8.0),
    )) as Arc<dyn Light>;

    scene.add_light("FillLight".to_string(), light2, Transform::identity());

    // Build BVH
    println!("Building scene BVH...");
    scene.build_bvh();

    // Create camera
    let aspect_ratio = 16.0 / 9.0;
    let camera = Camera::new(aspect_ratio);

    // Create renderer
    let width = 1920;
    let height = (width as f64 / aspect_ratio) as usize;

    println!("Creating renderer ({}x{})...", width, height);
    let renderer = Renderer::new(width, height)
        .with_samples(100) // 50 samples per pixel
        .with_max_depth(10) // 5 ray bounces
        .with_camera(camera);

    // Render!
    println!("\n>>> Rendering scene to simple_render.ppm...\n");
    renderer.render(&scene, "simple_render.ppm");

    println!("\n=== Render Complete! ===");
    println!("Output: simple_render.ppm");
    println!("\nView with:");
    println!("  - Linux: eog simple_render.ppm / feh simple_render.ppm");
    println!("  - macOS: open simple_render.ppm");
    println!("  - Windows: simple_render.ppm (opens in default viewer)");
}
