use raytracing::{
    camera::Camera,
    core::math::{Transform, Vec2},
    geometry::{Plane, Primitive, Rect, RectAxis, Sphere},
    lights::{Light, PointLight},
    materials::material::{DiffuseMaterial, Material, PbrMaterial},
    renderer::Renderer,
    scene::Scene,
    vec3::Vec3,
};
use std::sync::Arc;

fn main() {
    println!("=== Materials Showcase Example ===\n");
    println!("This example demonstrates the advanced materials:");
    println!("  - GGX microfacet metal (gold, rough silver)");
    println!("  - Dielectric glass with refraction");
    println!("  - Lambertian diffuse for comparison\n");

    // Create scene
    let mut scene = Scene::new();

    // === Ground Plane ===
    let ground_material: Arc<dyn Material> =
        Arc::new(DiffuseMaterial::new(Vec3::new(0.5, 0.5, 0.5)));

    let ground_plane: Arc<dyn Primitive> = Arc::new(Plane::new(
        Vec3::new(0.0, 1.0, 0.0),  // Up normal
        Vec3::new(0.0, -1.0, 0.0), // Position at y=-1
        ground_material.clone(),
    ));

    scene.add_geometry(
        "Ground".to_string(),
        ground_plane,
        ground_material,
        Transform::identity(),
    );

    // === Row 1: Metallic Materials (GGX) ===

    // Polished gold (low roughness)
    let gold_smooth: Arc<dyn Material> = Arc::new(
        PbrMaterial::new(
            Vec3::new(1.0, 0.85, 0.4), // Gold color
            1.0,                        // Fully metallic
            0.1,                        // Low roughness = polished
        ),
    );

    let gold_sphere: Arc<dyn Primitive> = Arc::new(Sphere::new(0.5, gold_smooth.clone()));
    scene.add_geometry(
        "PolishedGold".to_string(),
        gold_sphere,
        gold_smooth,
        Transform::translate(Vec3::new(-2.5, 0.0, -5.0)),
    );

    // Rough gold (high roughness)
    let gold_rough: Arc<dyn Material> = Arc::new(
        PbrMaterial::new(
            Vec3::new(1.0, 0.85, 0.4), // Same gold color
            1.0,                        // Fully metallic
            0.5,                        // High roughness = brushed/rough
        ),
    );

    let gold_rough_sphere: Arc<dyn Primitive> = Arc::new(Sphere::new(0.5, gold_rough.clone()));
    scene.add_geometry(
        "RoughGold".to_string(),
        gold_rough_sphere,
        gold_rough,
        Transform::translate(Vec3::new(-1.0, 0.0, -5.0)),
    );

    // === Center: Glass Sphere (Dielectric) ===
    let glass_material: Arc<dyn Material> = Arc::new(
        PbrMaterial::new(
            Vec3::new(1.0, 1.0, 1.0), // White (tinted glass)
            0.0,                       // Not metallic
            0.01,                      // Very smooth (< 0.1 triggers Dielectric)
        )
        .with_ior(1.5), // Glass IOR
    );

    let glass_sphere: Arc<dyn Primitive> = Arc::new(Sphere::new(0.5, glass_material.clone()));
    scene.add_geometry(
        "Glass".to_string(),
        glass_sphere,
        glass_material,
        Transform::translate(Vec3::new(0.5, 0.0, -5.0)),
    );

    // === Row 2: More Metals ===

    // Silver (polished)
    let silver: Arc<dyn Material> = Arc::new(
        PbrMaterial::new(
            Vec3::new(0.95, 0.95, 0.95), // Silver color
            1.0,                          // Fully metallic
            0.15,                         // Polished
        ),
    );

    let silver_sphere: Arc<dyn Primitive> = Arc::new(Sphere::new(0.5, silver.clone()));
    scene.add_geometry(
        "Silver".to_string(),
        silver_sphere,
        silver,
        Transform::translate(Vec3::new(2.0, 0.0, -5.0)),
    );

    // === Diffuse Comparison Sphere ===
    let diffuse_red: Arc<dyn Material> = Arc::new(DiffuseMaterial::new(Vec3::new(0.8, 0.2, 0.2)));

    let diffuse_sphere: Arc<dyn Primitive> = Arc::new(Sphere::new(0.5, diffuse_red.clone()));
    scene.add_geometry(
        "DiffuseRed".to_string(),
        diffuse_sphere,
        diffuse_red,
        Transform::translate(Vec3::new(-2.5, -1.5, -4.0)),
    );

    // === Back Wall for Reflections ===
    let wall_material: Arc<dyn Material> = Arc::new(DiffuseMaterial::new(Vec3::new(0.7, 0.7, 0.8)));

    let back_wall: Arc<dyn Primitive> = Arc::new(Rect::new(
        RectAxis::XY,
        Vec2::new(-5.0, -2.0),
        Vec2::new(5.0, 3.0),
        -7.0, // z position
        wall_material.clone(),
    ));

    scene.add_geometry(
        "BackWall".to_string(),
        back_wall,
        wall_material,
        Transform::identity(),
    );

    // === Lighting ===

    // Key light (main light from top-right)
    let key_light = Arc::new(PointLight::new(
        Vec3::new(3.0, 4.0, -2.0),
        Vec3::new(20.0, 20.0, 20.0), // Bright white
    )) as Arc<dyn Light>;

    scene.add_light("KeyLight".to_string(), key_light, Transform::identity());

    // Fill light (softer from left)
    let fill_light = Arc::new(PointLight::new(
        Vec3::new(-3.0, 2.0, -2.0),
        Vec3::new(8.0, 8.0, 10.0), // Cooler fill
    )) as Arc<dyn Light>;

    scene.add_light("FillLight".to_string(), fill_light, Transform::identity());

    // Rim light (from behind)
    let rim_light = Arc::new(PointLight::new(
        Vec3::new(0.0, 1.0, -8.0),
        Vec3::new(5.0, 5.0, 5.0),
    )) as Arc<dyn Light>;

    scene.add_light("RimLight".to_string(), rim_light, Transform::identity());

    // Build BVH
    println!("Building scene BVH...");
    scene.build_bvh();

    // Create camera
    let aspect_ratio = 16.0 / 9.0;
    let camera = Camera::new(aspect_ratio);

    // Create renderer with higher sample count for glass/metal convergence
    let width = 1280;
    let height = (width as f64 / aspect_ratio) as usize;

    println!("Creating renderer ({}x{})...", width, height);
    let renderer = Renderer::new(width, height)
        .with_samples(200) // Higher samples for glass caustics and metal highlights
        .with_max_depth(12) // Deeper for glass refractions
        .with_camera(camera);

    // Render!
    println!("\n>>> Rendering materials showcase to materials_showcase.ppm...\n");
    println!("Materials shown (left to right):");
    println!("  - Polished Gold (GGX, roughness=0.1)");
    println!("  - Rough Gold (GGX, roughness=0.5)");
    println!("  - Glass (Dielectric, IOR=1.5)");
    println!("  - Polished Silver (GGX, roughness=0.15)");
    println!("  - Red Diffuse (Lambertian) for comparison");
    println!("\nThis will take a few minutes due to high sample count...\n");

    renderer.render(&scene, "materials_showcase.ppm");

    println!("\n=== Render Complete! ===");
    println!("Output: materials_showcase.ppm");
    println!("\nLook for:");
    println!("  - Sharp highlights on polished metals");
    println!("  - Diffuse reflections on rough gold");
    println!("  - Refractions and caustics through glass");
    println!("  - Fresnel reflections on glass edges");
    println!("\nView with:");
    println!("  - Linux: eog materials_showcase.ppm");
    println!("  - macOS: open materials_showcase.ppm");
    println!("  - Windows: materials_showcase.ppm");
}
