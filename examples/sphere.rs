use raytracing::objects::Sphere;
use raytracing::renderer::Renderer;
use raytracing::scene::Scene;
use raytracing::vec3::Vec3;

fn main() {
    let mut world = Scene::new();
    world.add(Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5));

    let renderer = Renderer::new(400, 225);
    renderer.render(&world, "sphere.ppm");
}
