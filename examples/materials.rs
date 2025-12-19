use raytracing::material::{Lambertian, Metal};
use raytracing::objects::Sphere;
use raytracing::renderer::Renderer;
use raytracing::scene::Scene;
use raytracing::vec3::Vec3;
use std::sync::Arc;

fn main() {
    let mut scene = Scene::new();

    // materials
    let ground_material = Arc::new(Lambertian::new(Vec3::new(0.8, 0.8, 0.0)));
    let center_material = Arc::new(Lambertian::new(Vec3::new(0.1, 0.2, 0.5)));
    let left_material = Arc::new(Metal::new(Vec3::new(0.8, 0.8, 0.8), 0.3));
    let right_material = Arc::new(Metal::new(Vec3::new(0.8, 0.6, 0.2), 0.0));

    // ground
    scene.add(Sphere::new(
        Vec3::new(0.0, -100.5, -1.0),
        100.0,
        ground_material,
    ));
    // center
    scene.add(Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5, center_material));
    // left
    scene.add(Sphere::new(Vec3::new(-1.0, 0.0, -1.0), 0.5, left_material));
    // right
    scene.add(Sphere::new(Vec3::new(1.0, 0.0, -1.0), 0.5, right_material));

    let renderer = Renderer::new(400, 225).with_samples(100).with_max_depth(50);

    renderer.render(&scene, "materials.ppm");
}
