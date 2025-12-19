use crate::hit::HitRecord;
use crate::ray::Ray;
use crate::vec3::Vec3;

/// Material trait defines how rays interact with surfaces
pub trait Material: Send + Sync {
    /// Attempt to scatter a ray off the surface
    /// Returns Some((attenuation, scattered_ray)) if the ray scatters
    /// Returns None if the ray is absorbed
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<(Vec3, Ray)>;
}

/// Lambertian (diffuse) material - scatters light randomly
#[derive(Clone, Copy)]
pub struct Lambertian {
    pub albedo: Vec3, // Surface color
}

impl Lambertian {
    pub fn new(albedo: Vec3) -> Self {
        Lambertian { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, hit: &HitRecord) -> Option<(Vec3, Ray)> {
        // Scatter in random direction in hemisphere
        let mut scatter_direction = hit.normal + Vec3::random_unit_vector();

        // Catch degenerate scatter direction
        if scatter_direction.length_squared() < 1e-8 {
            scatter_direction = hit.normal;
        }

        let scattered = Ray::new(hit.point, scatter_direction);
        Some((self.albedo, scattered))
    }
}

/// Metal (reflective) material
#[derive(Clone, Copy)]
pub struct Metal {
    pub albedo: Vec3, // Surface color
    pub fuzz: f64,    // Fuzziness of reflection (0 = perfect mirror, 1 = very fuzzy)
}

impl Metal {
    pub fn new(albedo: Vec3, fuzz: f64) -> Self {
        Metal {
            albedo,
            fuzz: fuzz.clamp(0.0, 1.0),
        }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<(Vec3, Ray)> {
        let reflected = ray.direction.reflect(hit.normal);
        let scattered = Ray::new(
            hit.point,
            reflected + self.fuzz * Vec3::random_unit_vector(),
        );

        // Only scatter if ray reflects above surface
        if scattered.direction.dot(hit.normal) > 0.0 {
            Some((self.albedo, scattered))
        } else {
            None // Absorbed
        }
    }
}
