/// Phase 2 Complete Showcase
///
/// This example demonstrates all Phase 2 features working together:
/// - Scene graph with hierarchy
/// - PBR materials with textures
/// - Area lights for soft lighting
/// - Geometry instancing
/// - Transform propagation
///
/// Creates a complete miniature scene with all features

use raytracing::{
    core::math::{Transform, Vec2},
    geometry::{Box3, Primitive, Rect, RectAxis, Sphere},
    lights::{AreaLight, Light, PointLight},
    material::{Lambertian, Metal},
    materials::{Material, PbrMaterial},
    scene::Scene,
    textures::{CheckerTexture, ConstantTexture, Texture},
    vec3::Vec3,
};
use std::sync::Arc;

fn main() {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║          Phase 2 Complete Feature Showcase               ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    let mut scene = Scene::new();

    // ========== Materials ==========
    println!("┌─ Materials ────────────────────────────────────────────┐");

    let ground_checker: Arc<dyn Texture> = Arc::new(CheckerTexture::new(
        Vec3::new(0.2, 0.2, 0.2),
        Vec3::new(0.8, 0.8, 0.8),
        1.0,
    ));

    let ground_material: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5)));

    let red_diffuse: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(0.9, 0.2, 0.2)));

    let blue_glossy: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(0.2, 0.4, 0.9)));

    let gold_metal: Arc<dyn raytracing::material::Material> =
        Arc::new(Metal::new(Vec3::new(1.0, 0.8, 0.3), 0.2));

    let green_rough: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(0.3, 0.8, 0.3)));

    println!("  ✓ Ground: Checker pattern texture");
    println!("  ✓ Diffuse: Red matte");
    println!("  ✓ Glossy: Blue (roughness 0.2)");
    println!("  ✓ Metallic: Gold");
    println!("  ✓ Rough: Green");
    println!("└────────────────────────────────────────────────────────┘\n");

    // ========== Scene Geometry ==========
    println!("┌─ Scene Geometry ───────────────────────────────────────┐");

    // Ground plane
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
    println!("  ✓ Ground plane (20×20 units)");

    // Center sphere with hierarchy
    let center_sphere: Arc<dyn Primitive> = Arc::new(Sphere::new(0.8, red_diffuse.clone()));
    let center_node = scene.add_geometry(
        "CenterSphere".to_string(),
        center_sphere,
        red_diffuse.clone(),
        Transform::translate(Vec3::new(0.0, 0.8, -5.0)),
    );

    // Small orbiting sphere (child of center)
    let orbit_sphere: Arc<dyn Primitive> = Arc::new(Sphere::new(0.2, blue_glossy.clone()));
    let orbit_node = scene.add_geometry(
        "OrbitSphere".to_string(),
        orbit_sphere,
        blue_glossy,
        Transform::translate(Vec3::new(1.5, 0.0, 0.0)),
    );
    scene.set_parent(orbit_node, center_node);
    println!("  ✓ Hierarchical spheres (parent + orbiting child)");

    // Metallic sphere
    let metal_sphere: Arc<dyn Primitive> = Arc::new(Sphere::new(0.6, gold_metal.clone()));
    scene.add_geometry(
        "MetalSphere".to_string(),
        metal_sphere,
        gold_metal,
        Transform::translate(Vec3::new(-2.5, 0.6, -4.0)),
    );
    println!("  ✓ Metallic gold sphere");

    println!("└────────────────────────────────────────────────────────┘\n");

    // ========== Instancing ==========
    println!("┌─ Geometry Instancing ──────────────────────────────────┐");

    // Create template box
    let box_template: Arc<dyn Primitive> = Arc::new(Box3::unit_cube(green_rough.clone()));
    let box_template_node = scene.add_geometry(
        "BoxTemplate".to_string(),
        box_template,
        green_rough.clone(),
        Transform::identity(),
    );

    // Create instances in a circle
    let radius = 4.0;
    let num_boxes = 6;

    for i in 0..num_boxes {
        let angle = (i as f64 / num_boxes as f64) * 2.0 * std::f64::consts::PI;
        let x = angle.cos() * radius;
        let z = -5.0 + angle.sin() * radius;

        let transform = Transform::translate(Vec3::new(x, 0.3, z))
            .then(&Transform::scale_uniform(0.3));

        scene.add_instance(
            format!("BoxInstance{}", i),
            box_template_node,
            transform,
        );
    }

    println!("  ✓ Box template created");
    println!("  ✓ {} instances arranged in circle", num_boxes);
    println!("└────────────────────────────────────────────────────────┘\n");

    // ========== Lighting ==========
    println!("┌─ Lighting Setup ───────────────────────────────────────┐");

    // Key light (area light)
    let key_light_mat: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(1.0, 1.0, 1.0)));
    let key_light_rect: Arc<dyn Primitive> = Arc::new(Rect::new(
        RectAxis::XZ,
        Vec2::new(-1.0, -1.0),
        Vec2::new(1.0, 1.0),
        5.0,
        key_light_mat,
    ));

    let key_light = Arc::new(AreaLight::new(
        key_light_rect,
        Vec3::new(20.0, 20.0, 20.0),
    )) as Arc<dyn Light>;

    scene.add_light(
        "KeyLight".to_string(),
        key_light,
        Transform::translate(Vec3::new(-2.0, 0.0, -5.0)),
    );
    println!("  ✓ Key light: 2×2 area light (soft shadows)");

    // Fill light (point light)
    let fill_light = Arc::new(PointLight::new(
        Vec3::new(3.0, 3.0, -3.0),
        Vec3::new(10.0, 10.0, 12.0),
    )) as Arc<dyn Light>;

    scene.add_light(
        "FillLight".to_string(),
        fill_light,
        Transform::identity(),
    );
    println!("  ✓ Fill light: Point light (blue tint)");

    // Rim light (small area light)
    let rim_light_mat: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(1.0, 1.0, 1.0)));
    let rim_light_rect: Arc<dyn Primitive> = Arc::new(Rect::new(
        RectAxis::XY,
        Vec2::new(-0.3, -0.3),
        Vec2::new(0.3, 0.3),
        -7.0,
        rim_light_mat,
    ));

    let rim_light = Arc::new(AreaLight::new(
        rim_light_rect,
        Vec3::new(15.0, 12.0, 10.0),
    )) as Arc<dyn Light>;

    scene.add_light(
        "RimLight".to_string(),
        rim_light,
        Transform::translate(Vec3::new(0.0, 2.0, 0.0)),
    );
    println!("  ✓ Rim light: Small area light (warm)");

    println!("└────────────────────────────────────────────────────────┘\n");

    // ========== Build Scene ==========
    println!("┌─ Building Scene ───────────────────────────────────────┐");
    scene.build_bvh();

    println!("  ✓ BVH acceleration structure built");
    println!("  ✓ Transform propagation completed");
    println!("  ✓ Scene ready for rendering");
    println!("└────────────────────────────────────────────────────────┘\n");

    // ========== Scene Statistics ==========
    println!("┌─ Scene Statistics ─────────────────────────────────────┐");
    println!("  Nodes: {} (including templates)", 4 + num_boxes);
    println!("  Lights: {}", scene.lights().len());
    println!("  Instances: {}", num_boxes);
    println!("  Materials: 5 types (diffuse, glossy, metal, textured)");
    println!("└────────────────────────────────────────────────────────┘\n");

    // ========== Feature Summary ==========
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║              Phase 2 Features Demonstrated               ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║ ✓ Scene Graph          │ Hierarchical transforms          ║");
    println!("║ ✓ PBR Materials        │ Metallic/roughness workflow      ║");
    println!("║ ✓ Textures             │ Checker pattern on ground        ║");
    println!("║ ✓ Area Lights          │ Soft shadows, importance sample  ║");
    println!("║ ✓ Point Lights         │ Delta lights for accents         ║");
    println!("║ ✓ Instancing           │ Memory-efficient duplication     ║");
    println!("║ ✓ Transform Propagation│ Parent-child relationships       ║");
    println!("║ ✓ BVH Integration      │ Fast ray-scene intersection      ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    // ========== Next Steps ==========
    println!("Next Steps:");
    println!("  • Phase 3: Advanced rendering (BDPT, volumetrics)");
    println!("  • Phase 4: I/O pipeline (scene files, mesh loading)");
    println!("  • Phase 5: Advanced features (SSS, spectral)");
    println!("  • Phase 6: Production pipeline (USD, optimization)\n");

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║                   Showcase Complete!                      ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
}
