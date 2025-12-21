/// Phase 2 Demo: Area Lights
///
/// This example demonstrates:
/// - Area lights using primitive shapes
/// - Light sampling and importance sampling
/// - Comparison between point lights and area lights
/// - Soft shadows from area lights

use raytracing::{
    core::math::{Transform, Vec2},
    geometry::{Primitive, Rect, RectAxis, Sphere},
    lights::{AreaLight, Light, PointLight},
    material::Lambertian,
    scene::Scene,
    vec3::Vec3,
};
use std::sync::Arc;

fn main() {
    println!("=== Phase 2 Demo: Area Lights ===\n");

    println!("Area lights provide:");
    println!("  - Soft shadows (unlike point lights)");
    println!("  - Physically accurate light distribution");
    println!("  - Energy conservation");
    println!("  - Importance sampling for path tracing\n");

    // Create scene
    let mut scene = Scene::new();

    // Ground plane
    println!("Step 1: Setting up scene geometry");
    let ground_material: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5)));

    let ground: Arc<dyn Primitive> = Arc::new(Rect::new(
        RectAxis::XZ,
        Vec2::new(-10.0, -10.0),
        Vec2::new(10.0, 10.0),
        0.0,
        ground_material.clone(),
    ));

    scene.add_geometry(
        "Ground".to_string(),
        ground,
        ground_material,
        Transform::identity(),
    );

    // Add some spheres to see lighting effects
    let sphere_mat: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(0.8, 0.3, 0.3)));

    for i in 0..3 {
        let x = -2.0 + i as f64 * 2.0;
        let sphere: Arc<dyn Primitive> = Arc::new(Sphere::new(0.5, sphere_mat.clone()));
        scene.add_geometry(
            format!("Sphere{}", i),
            sphere,
            sphere_mat.clone(),
            Transform::translate(Vec3::new(x, 0.5, -5.0)),
        );
    }
    println!("  ✓ Ground plane");
    println!("  ✓ 3 spheres positioned at y=0.5\n");

    // Create different types of lights
    println!("Step 2: Creating lights\n");

    // Point light for comparison
    println!("  A. Point Light (delta light)");
    let point_light = Arc::new(PointLight::new(
        Vec3::new(-3.0, 4.0, -5.0),
        Vec3::new(50.0, 50.0, 50.0),
    )) as Arc<dyn Light>;

    println!("     Position: (-3, 4, -5)");
    println!("     Intensity: 50.0");
    println!("     Is delta: {}", point_light.is_delta());
    println!("     Power: {:.2}", point_light.power());

    scene.add_light(
        "PointLight".to_string(),
        point_light.clone(),
        Transform::identity(),
    );

    // Rectangular area light
    println!("\n  B. Rectangular Area Light");
    let light_material: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(1.0, 1.0, 1.0)));

    let light_rect: Arc<dyn Primitive> = Arc::new(Rect::new(
        RectAxis::XZ,
        Vec2::new(-0.5, -0.5),
        Vec2::new(0.5, 0.5),
        4.0,
        light_material,
    ));

    let area_light = Arc::new(AreaLight::new(
        light_rect.clone(),
        Vec3::new(15.0, 15.0, 15.0),
    )) as Arc<dyn Light>;

    println!("     Size: 1.0 × 1.0 units");
    println!("     Position: (0, 4, -5) [center]");
    println!("     Emission: 15.0");
    println!("     Is delta: {}", area_light.is_delta());
    println!("     Power: {:.2}", area_light.power());
    println!("     Surface area: {:.2}", light_rect.surface_area());

    scene.add_light(
        "AreaLight".to_string(),
        area_light.clone(),
        Transform::translate(Vec3::new(0.0, 0.0, -5.0)),
    );

    // Large area light
    println!("\n  C. Large Area Light (2-sided)");
    let large_light_material: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(1.0, 1.0, 1.0)));

    let large_light_rect: Arc<dyn Primitive> = Arc::new(Rect::new(
        RectAxis::XY,
        Vec2::new(-1.0, -1.0),
        Vec2::new(1.0, 1.0),
        -3.0,
        large_light_material,
    ));

    let large_area_light = Arc::new(
        AreaLight::new(large_light_rect.clone(), Vec3::new(8.0, 8.0, 8.0)).with_two_sided(true),
    ) as Arc<dyn Light>;

    println!("     Size: 2.0 × 2.0 units");
    println!("     Position: (3, 0, -3) [center]");
    println!("     Emission: 8.0 (both sides)");
    println!("     Two-sided: Yes");
    println!("     Power: {:.2}", large_area_light.power());

    scene.add_light(
        "LargeAreaLight".to_string(),
        large_area_light.clone(),
        Transform::translate(Vec3::new(3.0, 0.0, 0.0)),
    );

    // Build scene
    println!("\n\nStep 3: Building scene");
    scene.build_bvh();
    println!("  ✓ BVH built");
    println!("  ✓ {} lights in scene", scene.lights().len());

    // Test light sampling
    println!("\nStep 4: Testing light sampling");
    let test_point = Vec3::new(0.0, 0.5, -5.0);

    // Sample point light
    println!("\n  Sampling Point Light from (0, 0.5, -5):");
    if let Some(sample) = point_light.sample_li(test_point, Vec2::new(0.5, 0.5)) {
        println!("    ✓ Direction: ({:.2}, {:.2}, {:.2})", sample.wi.x, sample.wi.y, sample.wi.z);
        println!("    ✓ Radiance: ({:.2}, {:.2}, {:.2})",
                 sample.radiance.x, sample.radiance.y, sample.radiance.z);
        println!("    ✓ PDF: {:.4}", sample.pdf);
        println!("    ✓ Distance: {:.2}", sample.distance);
    }

    // Sample area light
    println!("\n  Sampling Area Light from (0, 0.5, -5):");
    if let Some(sample) = area_light.sample_li(test_point, Vec2::new(0.5, 0.5)) {
        println!("    ✓ Direction: ({:.2}, {:.2}, {:.2})", sample.wi.x, sample.wi.y, sample.wi.z);
        println!("    ✓ Radiance: ({:.2}, {:.2}, {:.2})",
                 sample.radiance.x, sample.radiance.y, sample.radiance.z);
        println!("    ✓ PDF: {:.4}", sample.pdf);
        println!("    ✓ Distance: {:.2}", sample.distance);
        println!("    ✓ Surface point: ({:.2}, {:.2}, {:.2})",
                 sample.position.x, sample.position.y, sample.position.z);
    }

    println!("\n=== Demo Complete ===");
    println!("\nKey Differences:");
    println!("  Point Light:");
    println!("    • Delta distribution (single direction)");
    println!("    • Sharp shadows");
    println!("    • PDF = 1.0");
    println!("    • Cheaper to sample");
    println!("\n  Area Light:");
    println!("    • Distributed across surface");
    println!("    • Soft shadows (penumbra)");
    println!("    • PDF based on solid angle");
    println!("    • More realistic");
    println!("\nArea lights are essential for:");
    println!("  - Physically-based rendering");
    println!("  - Realistic soft shadows");
    println!("  - Energy conservation");
    println!("  - Global illumination");
}
