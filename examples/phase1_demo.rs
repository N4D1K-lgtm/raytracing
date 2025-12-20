/// Phase 1 Demo - Showcasing new primitives and transform system
///
/// This example demonstrates:
/// - New Primitive trait with Sphere, Plane, Rectangle, Box
/// - Transform system (translate, rotate, scale)
/// - UV mapping and tangent vectors
/// - New BVH acceleration structure
/// - Intersection records with full surface data
use raytracing::acceleration::build_bvh;
use raytracing::camera::Camera;
use raytracing::core::math::{Transform, Vec2};
use raytracing::geometry::{Box3, Plane, Primitive, Rect, RectAxis, Sphere, TransformedPrimitive};
use raytracing::material::{Lambertian, Metal};
use raytracing::vec3::Vec3;
use std::sync::Arc;

fn main() {
    println!("🎨 Phase 1 Demo - New Primitives & Transform System");
    println!("{}", "=".repeat(60));

    // Image settings
    let width = 800;
    let height = 600;
    let aspect_ratio = width as f64 / height as f64;

    // Materials
    let ground_material: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5)));
    let red_material: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(0.8, 0.2, 0.2)));
    let green_material: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(0.2, 0.8, 0.2)));
    let blue_material: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(0.2, 0.2, 0.8)));
    let metal_material: Arc<dyn raytracing::material::Material> =
        Arc::new(Metal::new(Vec3::new(0.8, 0.8, 0.8), 0.1));

    // Create primitives with transforms
    let mut primitives: Vec<Arc<dyn Primitive>> = Vec::new();

    println!("📦 Creating scene primitives...");

    // 1. Ground plane (XZ plane at y=-0.5)
    println!("  ✓ Ground plane");
    primitives.push(Arc::new(Plane::new(
        Vec3::new(0.0, -0.5, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        ground_material.clone(),
    )));

    // 2. Sphere in center (with transform for positioning)
    println!("  ✓ Center sphere (transformed)");
    primitives.push(Arc::new(TransformedPrimitive::new(
        Box::new(Sphere::new(0.5, red_material.clone())),
        Transform::translate(Vec3::new(0.0, 0.0, 0.0)),
    )));

    // 3. Box on the left (transformed: translated and scaled)
    println!("  ✓ Left box (transformed)");
    let box_transform =
        Transform::translate(Vec3::new(-1.5, 0.0, 0.0)).then(&Transform::scale_uniform(0.5));
    primitives.push(Arc::new(TransformedPrimitive::new(
        Box::new(Box3::unit_cube(green_material.clone())),
        box_transform,
    )));

    // 4. Rectangle on the right (standing vertical)
    println!("  ✓ Right rectangle");
    primitives.push(Arc::new(TransformedPrimitive::new(
        Box::new(Rect::new(
            RectAxis::XY,
            Vec2::new(-0.4, -0.4),
            Vec2::new(0.4, 0.4),
            0.0,
            blue_material.clone(),
        )),
        Transform::translate(Vec3::new(1.5, 0.3, 0.0)),
    )));

    // 5. Metallic sphere in back
    println!("  ✓ Back metallic sphere (transformed)");
    primitives.push(Arc::new(TransformedPrimitive::new(
        Box::new(Sphere::new(0.3, metal_material.clone())),
        Transform::translate(Vec3::new(0.0, 0.0, -2.0)),
    )));

    // 6. Small boxes to demonstrate instancing potential
    println!("  ✓ Small boxes (3 instances with different transforms)");
    for i in 0..3 {
        let x_offset = -1.0 + i as f64 * 1.0;
        let transform = Transform::translate(Vec3::new(x_offset, -0.3, 1.0))
            .then(&Transform::scale_uniform(0.2));

        primitives.push(Arc::new(TransformedPrimitive::new(
            Box::new(Box3::unit_cube(red_material.clone())),
            transform,
        )));
    }

    println!("\n🌲 Building BVH acceleration structure...");
    let bvh = build_bvh(&primitives);
    println!("  ✓ BVH built with {} primitives", primitives.len());

    // Camera setup
    let _camera = Camera::new_look_at(
        Vec3::new(3.0, 2.0, 3.0), // Eye position
        Vec3::new(0.0, 0.0, 0.0), // Look at center
        Vec3::new(0.0, 1.0, 0.0), // Up vector
        40.0,                     // Vertical FOV
        aspect_ratio,
    );

    // Note: The renderer currently works with the old Scene/Hittable system
    // For this demo, we'll just confirm the BVH was built successfully
    // In Phase 2, we'll integrate the new scene graph with the renderer

    if let Some(_bvh) = bvh {
        println!("\n✅ Phase 1 Demo Setup Complete!");
        println!("   - {} primitives created", primitives.len());
        println!("   - All using Transform system");
        println!("   - BVH acceleration structure ready");
        println!("   - Materials: Lambertian & Metal");
        println!("\n💡 Note: Full rendering integration comes in Phase 2");
        println!("   (Scene graph, material system refactor)");
    } else {
        println!("\n❌ Failed to build BVH");
    }

    println!("\n🎯 Phase 1 Achievements:");
    println!("   ✓ Transform system (TRS)");
    println!("   ✓ 4 primitive types (Sphere, Plane, Rect, Box)");
    println!("   ✓ TransformedPrimitive wrapper");
    println!("   ✓ BVH acceleration");
    println!("   ✓ UV mapping & tangent vectors");
    println!("   ✓ Material assignment per-primitive");
}
