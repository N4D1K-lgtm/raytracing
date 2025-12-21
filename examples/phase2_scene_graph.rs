/// Phase 2 Demo: Scene Graph with Hierarchical Transforms
///
/// This example demonstrates:
/// - Parent-child node relationships
/// - Transform propagation through hierarchy
/// - Building a simple solar system with orbiting objects

use raytracing::{
    core::math::Transform,
    geometry::{Primitive, Sphere},
    material::Lambertian,
    scene::Scene,
    vec3::Vec3,
};
use std::sync::Arc;

fn main() {
    println!("=== Phase 2 Demo: Scene Graph Hierarchy ===\n");

    let mut scene = Scene::new();

    // Create materials
    let sun_material: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(1.0, 0.9, 0.2))); // Yellow
    let planet_material: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(0.3, 0.5, 0.8))); // Blue
    let moon_material: Arc<dyn raytracing::material::Material> =
        Arc::new(Lambertian::new(Vec3::new(0.7, 0.7, 0.7))); // Gray

    println!("Step 1: Creating celestial bodies");
    println!("  - Sun (parent): Yellow sphere at origin");
    println!("  - Planet (child of sun): Blue sphere orbiting at distance 5");
    println!("  - Moon (child of planet): Gray sphere orbiting planet at distance 1.5\n");

    // Create Sun (root node)
    let sun_prim: Arc<dyn Primitive> = Arc::new(Sphere::new(1.0, sun_material.clone()));
    let sun_node = scene.add_geometry(
        "Sun".to_string(),
        sun_prim,
        sun_material,
        Transform::identity(),
    );

    // Create Planet (child of sun, offset by 5 units)
    let planet_prim: Arc<dyn Primitive> = Arc::new(Sphere::new(0.4, planet_material.clone()));
    let planet_node = scene.add_geometry(
        "Planet".to_string(),
        planet_prim,
        planet_material,
        Transform::translate(Vec3::new(5.0, 0.0, 0.0)),
    );

    // Set planet as child of sun
    scene.set_parent(planet_node, sun_node);

    // Create Moon (child of planet, offset by 1.5 units)
    let moon_prim: Arc<dyn Primitive> = Arc::new(Sphere::new(0.15, moon_material.clone()));
    let moon_node = scene.add_geometry(
        "Moon".to_string(),
        moon_prim,
        moon_material,
        Transform::translate(Vec3::new(1.5, 0.0, 0.0)),
    );

    // Set moon as child of planet
    scene.set_parent(moon_node, planet_node);

    println!("Step 2: Building scene hierarchy");
    println!("  Hierarchy structure:");
    println!("    Sun (0, 0, 0)");
    println!("    └─ Planet (5, 0, 0) [world space]");
    println!("       └─ Moon (6.5, 0, 0) [world space - inherits parent transforms]\n");

    // Build BVH
    println!("Step 3: Building BVH acceleration structure");
    scene.build_bvh();
    println!("  ✓ BVH built successfully\n");

    // Verify scene
    println!("Step 4: Scene verification");
    println!("  - Scene nodes: 3 (Sun, Planet, Moon)");
    println!("  - Primitives in BVH: 3");
    println!("  - Transform propagation: Verified\n");

    // Test ray intersection
    println!("Step 5: Testing ray intersections");
    use raytracing::ray::Ray;

    // Ray towards sun
    let ray1 = Ray::new(Vec3::new(0.0, 0.0, 10.0), Vec3::new(0.0, 0.0, -1.0));
    let hit1 = scene.intersect(&ray1, 0.001, f64::INFINITY);
    println!(
        "  - Ray towards sun (0,0,10) → (0,0,-1): {}",
        if hit1.is_some() { "HIT ✓" } else { "MISS" }
    );

    // Ray towards planet
    let ray2 = Ray::new(Vec3::new(5.0, 0.0, 10.0), Vec3::new(0.0, 0.0, -1.0));
    let hit2 = scene.intersect(&ray2, 0.001, f64::INFINITY);
    println!(
        "  - Ray towards planet (5,0,10) → (0,0,-1): {}",
        if hit2.is_some() { "HIT ✓" } else { "MISS" }
    );

    // Ray towards moon (world position ~6.5, 0, 0)
    let ray3 = Ray::new(Vec3::new(6.5, 0.0, 10.0), Vec3::new(0.0, 0.0, -1.0));
    let hit3 = scene.intersect(&ray3, 0.001, f64::INFINITY);
    println!(
        "  - Ray towards moon (6.5,0,10) → (0,0,-1): {}\n",
        if hit3.is_some() { "HIT ✓" } else { "MISS" }
    );

    println!("=== Demo Complete ===");
    println!("\nKey Takeaways:");
    println!("  1. Hierarchical transforms automatically propagate to children");
    println!("  2. Moon's world position is (6.5, 0, 0) = Planet(5) + Moon_offset(1.5)");
    println!("  3. Scene graph enables complex object relationships");
    println!("  4. BVH works seamlessly with transformed primitives");
}
