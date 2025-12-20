use raytracing::material::Lambertian;
use raytracing::objects::Sphere;
use raytracing::renderer::Renderer;
use raytracing::scene::Scene;
use raytracing::vec3::Vec3;
use std::sync::Arc;

fn main() {
    let mut world = Scene::new();

    // Create a simple gray material
    let material = Arc::new(Lambertian::new(Vec3::new(0.7, 0.7, 0.7)));

    world.add(Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5, material));

    let renderer = Renderer::new(400, 225);
    renderer.render(&world, "sphere.ppm");
}
