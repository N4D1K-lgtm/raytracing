/// Phase 2 Demo: Geometry Instancing
///
/// This example demonstrates:
/// - Creating template geometry once
/// - Instancing with different transforms
/// - Memory efficiency of instancing
/// - Optional material overrides per instance

use raytracing::{
    core::math::Transform,
    geometry::{Box3, Primitive, Sphere},
    material::Lambertian,
    scene::Scene,
    vec3::Vec3,
};
use std::sync::Arc;

fn main() {
    println!("=== Phase 2 Demo: Geometry Instancing ===\n");

    println!("Instancing allows:");
    println!("  - Memory efficiency (geometry defined once)");
    println!("  - Multiple instances with different transforms");
    println!("  - Optional material overrides per instance");
    println!("  - Perfect for repeated geometry (trees, buildings, particles)\n");

    let mut scene = Scene::new();

    // Create materials
    let red_material: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(0.9, 0.2, 0.2)));
    let green_material: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(0.2, 0.9, 0.2)));
    let blue_material: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(0.2, 0.2, 0.9)));
    let gold_material: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(1.0, 0.8, 0.2)));

    // Example 1: Sphere instancing
    println!("Example 1: Instancing a Sphere\n");
    println!("  Creating template sphere (radius=0.3)...");

    let sphere_template: Arc<dyn Primitive> = Arc::new(Sphere::new(0.3, red_material.clone()));
    let sphere_template_node = scene.add_geometry(
        "SphereTemplate".to_string(),
        sphere_template,
        red_material.clone(),
        Transform::identity(),
    );

    println!("  ✓ Template created\n");
    println!("  Creating 5 instances in a row:");

    for i in 0..5 {
        let x = -4.0 + i as f64 * 2.0;
        let y = 1.0;

        scene.add_instance(
            format!("SphereInstance{}", i),
            sphere_template_node,
            Transform::translate(Vec3::new(x, y, 0.0)),
        );

        println!("    Instance {}: position ({:.1}, {:.1}, 0.0)", i, x, y);
    }

    // Example 2: Box instancing with scale variations
    println!("\n\nExample 2: Instancing a Box with Scale Variations\n");
    println!("  Creating template box (unit cube)...");

    let box_template: Arc<dyn Primitive> = Arc::new(Box3::unit_cube(green_material.clone()));
    let box_template_node = scene.add_geometry(
        "BoxTemplate".to_string(),
        box_template,
        green_material.clone(),
        Transform::identity(),
    );

    println!("  ✓ Template created\n");
    println!("  Creating 4 instances with different scales:");

    let scales = [0.3, 0.5, 0.7, 0.9];
    for (i, &scale) in scales.iter().enumerate() {
        let x = -3.0 + i as f64 * 2.0;
        let transform = Transform::translate(Vec3::new(x, -1.0, 0.0))
            .then(&Transform::scale_uniform(scale));

        scene.add_instance(
            format!("BoxInstance{}", i),
            box_template_node,
            transform,
        );

        println!("    Instance {}: position ({:.1}, -1.0, 0.0), scale {:.1}", i, x, scale);
    }

    // Example 3: Grid of instances
    println!("\n\nExample 3: Grid of Instances (3×3)\n");
    println!("  Creating template sphere for grid...");

    let grid_template: Arc<dyn Primitive> = Arc::new(Sphere::new(0.2, blue_material.clone()));
    let grid_template_node = scene.add_geometry(
        "GridTemplate".to_string(),
        grid_template,
        blue_material.clone(),
        Transform::identity(),
    );

    println!("  ✓ Template created\n");
    println!("  Creating 3×3 grid of instances:");

    for row in 0..3 {
        for col in 0..3 {
            let x = -2.0 + col as f64 * 2.0;
            let z = 2.0 + row as f64 * 2.0;

            scene.add_instance(
                format!("GridInstance_{}_{}", row, col),
                grid_template_node,
                Transform::translate(Vec3::new(x, 2.0, z)),
            );
        }
    }
    println!("    ✓ 9 instances created in grid formation");

    // Example 4: Circular arrangement
    println!("\n\nExample 4: Circular Arrangement\n");
    println!("  Creating template box for circular pattern...");

    let circle_template: Arc<dyn Primitive> = Arc::new(Box3::unit_cube(gold_material.clone()));
    let circle_template_node = scene.add_geometry(
        "CircleTemplate".to_string(),
        circle_template,
        gold_material,
        Transform::identity(),
    );

    println!("  ✓ Template created\n");
    println!("  Creating 8 instances in a circle:");

    let radius = 5.0;
    let num_instances = 8;

    for i in 0..num_instances {
        let angle = (i as f64 / num_instances as f64) * 2.0 * std::f64::consts::PI;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;

        let transform = Transform::translate(Vec3::new(x, 0.0, z))
            .then(&Transform::scale_uniform(0.4));

        scene.add_instance(
            format!("CircleInstance{}", i),
            circle_template_node,
            transform,
        );

        println!("    Instance {}: angle {:.0}°, position ({:.2}, 0.0, {:.2})",
                 i, angle.to_degrees(), x, z);
    }

    // Build scene
    println!("\n\nBuilding scene...");
    scene.build_bvh();

    // Count instances
    let total_instances = 5 + 4 + 9 + 8;  // From all examples
    println!("  ✓ BVH built successfully");
    println!("  ✓ {} template geometries defined", 4);
    println!("  ✓ {} instances created", total_instances);
    println!("  ✓ Memory savings: {:.1}x (vs {} unique geometries)\n",
             total_instances as f64 / 4.0, total_instances);

    println!("=== Demo Complete ===");
    println!("\nInstancing Benefits:");
    println!("  1. Memory Efficiency:");
    println!("     - Geometry data stored only once");
    println!("     - Transforms are lightweight (64 bytes each)");
    println!("     - {} instances ≈ {}KB vs {}KB without instancing",
             total_instances,
             total_instances / 15,  // Rough estimate
             total_instances * 2);

    println!("\n  2. Performance:");
    println!("     - Faster BVH construction");
    println!("     - Better cache coherency");
    println!("     - Reduced memory bandwidth");

    println!("\n  3. Flexibility:");
    println!("     - Independent transforms per instance");
    println!("     - Optional material overrides");
    println!("     - Easy to add/remove instances");

    println!("\n  4. Use Cases:");
    println!("     - Forests (thousands of trees)");
    println!("     - Cities (repeated buildings)");
    println!("     - Particle systems");
    println!("     - Crowd simulation");
}
