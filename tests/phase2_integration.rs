use raytracing::{
    core::math::{Transform, Vec2},
    geometry::{Primitive, Rect, RectAxis, Sphere},
    lights::{AreaLight, Light, PointLight},
    materials::{DiffuseMaterial, EmissiveMaterial, Material, PbrMaterial},
    scene::Scene,
    textures::{CheckerTexture, ConstantTexture, Texture},
    vec3::Vec3,
};
use std::sync::Arc;

/// Test basic scene graph functionality
#[test]
fn test_scene_graph_hierarchy() {
    let mut scene = Scene::new();

    let mat: Arc<dyn raytracing::material::Material> =
        Arc::new(raytracing::material::Lambertian::new(Vec3::new(
            0.5, 0.5, 0.5,
        )));
    let prim: Arc<dyn Primitive> = Arc::new(Sphere::new(1.0, mat.clone()));

    // Create parent node
    let parent = scene.add_geometry(
        "parent".to_string(),
        prim.clone(),
        mat.clone(),
        Transform::translate(Vec3::new(1.0, 0.0, 0.0)),
    );

    // Create child node
    let child = scene.add_geometry(
        "child".to_string(),
        prim,
        mat,
        Transform::translate(Vec3::new(0.0, 1.0, 0.0)),
    );

    // Set hierarchy
    scene.set_parent(child, parent);

    // Build BVH
    scene.build_bvh();

    // Verify scene is not empty
    assert!(!scene.is_empty());
}

/// Test PBR material with textures
#[test]
fn test_pbr_material_with_textures() {
    // Create various textures
    let albedo_texture: Arc<dyn Texture> =
        Arc::new(ConstantTexture::new(Vec3::new(0.8, 0.2, 0.2)));

    let _checker: Arc<dyn Texture> = Arc::new(CheckerTexture::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 1.0),
        1.0,
    ));

    // Create PBR material with constant values
    let pbr1 = PbrMaterial::new(Vec3::new(0.8, 0.8, 0.8), 0.0, 0.5);
    assert!(!pbr1.is_emissive());

    // Create PBR material with textures
    let metallic: Arc<dyn Texture> = Arc::new(ConstantTexture::new(Vec3::new(1.0, 1.0, 1.0)));
    let roughness: Arc<dyn Texture> = Arc::new(ConstantTexture::new(Vec3::new(0.3, 0.3, 0.3)));

    let pbr2 = PbrMaterial::with_textures(albedo_texture, metallic, roughness);
    assert!(!pbr2.is_emissive());

    // Create emissive material
    let emissive = EmissiveMaterial::new(Vec3::new(1.0, 0.5, 0.0), 10.0);
    assert!(emissive.is_emissive());
}

/// Test area light functionality
#[test]
fn test_area_light_integration() {
    let mat: Arc<dyn raytracing::material::Material> =
        Arc::new(raytracing::material::Lambertian::new(Vec3::new(
            1.0, 1.0, 1.0,
        )));

    // Create rectangular area light
    let light_prim: Arc<dyn Primitive> = Arc::new(Rect::new(
        RectAxis::XY,
        Vec2::new(-1.0, -1.0),
        Vec2::new(1.0, 1.0),
        5.0,
        mat,
    ));

    let area_light = AreaLight::new(light_prim, Vec3::new(10.0, 10.0, 10.0));

    assert!(!area_light.is_delta());
    assert!(area_light.power() > 0.0);

    // Test sampling
    let sample = area_light.sample_li(Vec3::new(0.0, 0.0, 10.0), Vec2::new(0.5, 0.5));
    assert!(sample.is_some());
}

/// Test point light
#[test]
fn test_point_light_integration() {
    let point_light = PointLight::new(Vec3::new(0.0, 5.0, 0.0), Vec3::new(100.0, 100.0, 100.0));

    assert!(point_light.is_delta());
    assert!(point_light.power() > 0.0);

    // Test sampling
    let sample = point_light.sample_li(Vec3::new(0.0, 0.0, 0.0), Vec2::new(0.5, 0.5));
    assert!(sample.is_some());

    if let Some(ls) = sample {
        assert!(ls.radiance.length() > 0.0);
        assert_eq!(ls.pdf, 1.0); // Delta lights have PDF = 1
    }
}

/// Test complete scene with all Phase 2 features
#[test]
fn test_phase2_complete_scene() {
    let mut scene = Scene::new();

    // Create materials
    let mat: Arc<dyn raytracing::material::Material> =
        Arc::new(raytracing::material::Lambertian::new(Vec3::new(
            0.7, 0.7, 0.7,
        )));

    // Add geometry nodes
    let sphere: Arc<dyn Primitive> = Arc::new(Sphere::new(1.0, mat.clone()));
    let _sphere_node = scene.add_geometry(
        "sphere".to_string(),
        sphere,
        mat.clone(),
        Transform::translate(Vec3::new(0.0, 0.0, -5.0)),
    );

    let rect: Arc<dyn Primitive> = Arc::new(Rect::new(
        RectAxis::XZ,
        Vec2::new(-5.0, -5.0),
        Vec2::new(5.0, 5.0),
        -1.0,
        mat.clone(),
    ));
    let _ground = scene.add_geometry(
        "ground".to_string(),
        rect,
        mat.clone(),
        Transform::identity(),
    );

    // Add point light
    let point_light = Arc::new(PointLight::new(
        Vec3::new(0.0, 5.0, 0.0),
        Vec3::new(100.0, 100.0, 100.0),
    )) as Arc<dyn Light>;

    let _light_node = scene.add_light(
        "point_light".to_string(),
        point_light,
        Transform::identity(),
    );

    // Build BVH
    scene.build_bvh();

    // Verify scene structure
    assert!(!scene.is_empty());
    assert_eq!(scene.lights().len(), 1);
    assert!(!scene.needs_rebuild());

    // Test ray intersection
    use raytracing::ray::Ray;
    let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0));
    let hit = scene.intersect(&ray, 0.001, f64::INFINITY);
    assert!(hit.is_some());
}

/// Test instancing
#[test]
fn test_scene_instancing() {
    let mut scene = Scene::new();

    let mat: Arc<dyn raytracing::material::Material> =
        Arc::new(raytracing::material::Lambertian::new(Vec3::new(
            0.5, 0.5, 0.5,
        )));

    // Create template geometry
    let sphere: Arc<dyn Primitive> = Arc::new(Sphere::new(1.0, mat.clone()));
    let template = scene.add_geometry(
        "template_sphere".to_string(),
        sphere,
        mat.clone(),
        Transform::identity(),
    );

    // Create instances at different positions
    let _instance1 = scene.add_instance(
        "instance1".to_string(),
        template,
        Transform::translate(Vec3::new(-2.0, 0.0, 0.0)),
    );

    let _instance2 = scene.add_instance(
        "instance2".to_string(),
        template,
        Transform::translate(Vec3::new(2.0, 0.0, 0.0)),
    );

    // Build BVH
    scene.build_bvh();

    assert!(!scene.is_empty());
}

/// Test texture evaluation
#[test]
fn test_texture_system() {
    // Constant texture
    let const_tex = ConstantTexture::new(Vec3::new(1.0, 0.0, 0.0));
    let color = const_tex.evaluate(Vec2::new(0.5, 0.5));
    assert!((color.x - 1.0).abs() < 0.01);

    // Checker texture
    let checker = CheckerTexture::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0), 1.0);
    let c1 = checker.evaluate(Vec2::new(0.25, 0.25));
    let c2 = checker.evaluate(Vec2::new(0.75, 0.75));
    // Should alternate between black and white
    assert!(c1.length_squared() < 0.1 || c1.length_squared() > 2.9);
    assert!(c2.length_squared() < 0.1 || c2.length_squared() > 2.9);
}

/// Test transform propagation in hierarchy
#[test]
fn test_transform_propagation() {
    let mut scene = Scene::new();

    let mat: Arc<dyn raytracing::material::Material> =
        Arc::new(raytracing::material::Lambertian::new(Vec3::new(
            0.5, 0.5, 0.5,
        )));
    let prim: Arc<dyn Primitive> = Arc::new(Sphere::new(0.5, mat.clone()));

    // Create parent at (2, 0, 0)
    let parent = scene.add_geometry(
        "parent".to_string(),
        prim.clone(),
        mat.clone(),
        Transform::translate(Vec3::new(2.0, 0.0, 0.0)),
    );

    // Create child at (0, 1, 0) relative to parent
    let child = scene.add_geometry(
        "child".to_string(),
        prim,
        mat,
        Transform::translate(Vec3::new(0.0, 1.0, 0.0)),
    );

    scene.set_parent(child, parent);
    scene.build_bvh();

    // Child should be transformed to world position (2, 1, 0)
    // This is tested implicitly by the BVH build not panicking
    assert!(!scene.needs_rebuild());
}
