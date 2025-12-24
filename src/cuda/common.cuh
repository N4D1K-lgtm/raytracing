#ifndef COMMON_CUH
#define COMMON_CUH

#include <cuda_runtime.h>
#include <optix.h>

// ============================================================================
// Constants
// ============================================================================

#ifndef M_PI
#define M_PI 3.14159265358979323846f
#endif

#ifndef M_1_PI
#define M_1_PI 0.31830988618379067154f // 1/π
#endif

// ============================================================================
// GPU Data Structures (must match Rust #[repr(C)] structs!)
// ============================================================================

/// Sphere primitive
/// Memory layout: 32 bytes (matches src/gpu/types.rs)
struct GpuSphere {
  float3 center;             // 12 bytes
  float radius;              // 4 bytes
  unsigned int material_idx; // 4 bytes
  unsigned int _padding[3];  // 12 bytes
};

/// Material properties
/// Memory layout: 32 bytes
struct GpuMaterial {
  float3 albedo;     // 12 bytes
  float3 emission;   // 12 bytes
  float _padding[2]; // 8 bytes
};

/// Point light
/// Memory layout: 32 bytes
struct GpuPointLight {
  float3 position;   // 12 bytes
  float3 intensity;  // 12 bytes
  float _padding[2]; // 8 bytes
};

/// Launch parameters (passed to OptiX kernel)
/// Must match src/gpu/params.rs LaunchParams
struct LaunchParams {
  // Output
  unsigned long long output_buffer; // u64 pointer

  // Camera
  float3 camera_eye;
  float _pad0;
  float3 camera_u;
  float _pad1;
  float3 camera_v;
  float _pad2;
  float3 camera_w;
  float _pad3;

  // Image dimensions
  unsigned int width;
  unsigned int height;

  // Render settings
  unsigned int samples_per_pixel;
  unsigned int max_depth;

  // Scene data
  unsigned long long traversable; // OptixTraversableHandle
  unsigned long long materials;   // Device pointer
  unsigned int num_materials;
  unsigned int _pad4;
  unsigned long long lights; // Device pointer
  unsigned int num_lights;
  unsigned int frame_number;
  unsigned long long spheres; // Device pointer
  unsigned int num_spheres;
  unsigned int _pad5;
};

__device__ inline float3 operator+(float3 a, float3 b) {
  return make_float3(a.x + b.x, a.y + b.y, a.z + b.z);
}

__device__ inline float3 operator-(float3 a, float3 b) {
  return make_float3(a.x - b.x, a.y - b.y, a.z - b.z);
}

__device__ inline float3 operator*(float3 a, float s) {
  return make_float3(a.x * s, a.y * s, a.z * s);
}

__device__ inline float dot(float3 a, float3 b) {
  return a.x * b.x + a.y * b.y + a.z * b.z;
}

// Vector division by scalar
__device__ inline float3 operator/(float3 a, float s) {
  float inv = 1.0f / s;
  return make_float3(a.x * inv, a.y * inv, a.z * inv);
}

// Component-wise multiplication
__device__ inline float3 operator*(float3 a, float3 b) {
  return make_float3(a.x * b.x, a.y * b.y, a.z * b.z);
}

// Scalar multiply (reversed order)
__device__ inline float3 operator*(float s, float3 a) { return a * s; }

// Vector length squared
__device__ inline float length_squared(float3 v) { return dot(v, v); }

// Vector length
__device__ inline float length(float3 v) { return sqrtf(length_squared(v)); }

// Normalize vector
__device__ inline float3 normalize(float3 v) { return v / length(v); }

// Linear interpolation
__device__ inline float3 lerp(float3 a, float3 b, float t) {
  return a * (1.0f - t) + b * t;
}

// Component-wise min
__device__ inline float3 fminf3(float3 a, float3 b) {
  return make_float3(fminf(a.x, b.x), fminf(a.y, b.y), fminf(a.z, b.z));
}

// Component-wise max
__device__ inline float3 fmaxf3(float3 a, float3 b) {
  return make_float3(fmaxf(a.x, b.x), fmaxf(a.y, b.y), fmaxf(a.z, b.z));
}

// ============================================================================
// Ray Structure
// ============================================================================

struct Ray {
  float3 origin;
  float3 direction;
};

__device__ inline Ray make_ray(float3 origin, float3 direction) {
  Ray r;
  r.origin = origin;
  r.direction = direction;
  return r;
}

__device__ inline float3 ray_at(const Ray &r, float t) {
  return r.origin + r.direction * t;
}

// ============================================================================
// Orthonormal Basis Construction
// ============================================================================

/// Build orthonormal basis from normal vector
/// Input: normal (normalized)
/// Output: tangent and bitangent (perpendicular to normal and each other)
__device__ inline void build_orthonormal_basis(const float3 &normal,
                                               float3 *tangent,
                                               float3 *bitangent) {
  // Choose an axis that's not parallel to normal
  float3 up = fabsf(normal.z) < 0.999f ? make_float3(0.0f, 0.0f, 1.0f)
                                       : make_float3(1.0f, 0.0f, 0.0f);

  // Gram-Schmidt orthogonalization
  *tangent = normalize(up - normal * dot(up, normal));

  // Cross product for third axis
  *bitangent = make_float3(normal.y * tangent->z - normal.z * tangent->y,
                           normal.z * tangent->x - normal.x * tangent->z,
                           normal.x * tangent->y - normal.y * tangent->x);
}

// ============================================================================
// Coordinate Space Transforms
// ============================================================================

/// Transform direction from world space to local shading space
/// Local space: normal = +Z, tangent = +X, bitangent = +Y
__device__ inline float3 world_to_local(const float3 &v, const float3 &normal,
                                        const float3 &tangent,
                                        const float3 &bitangent) {
  return make_float3(dot(v, tangent), dot(v, bitangent), dot(v, normal));
}

/// Transform direction from local shading space to world space
__device__ inline float3 local_to_world(const float3 &v, const float3 &normal,
                                        const float3 &tangent,
                                        const float3 &bitangent) {
  return tangent * v.x + bitangent * v.y + normal * v.z;
}

// ============================================================================
// Ray-Sphere Intersection
// ============================================================================

/// Intersect ray with sphere
/// Returns true if hit, and sets t to distance along ray
/// This is the CORE algorithm of sphere ray tracing!
///
/// Math explanation:
/// Sphere equation: |P - C|² = r²  (P = point, C = center, r = radius)
/// Ray equation: P(t) = O + t*D    (O = origin, D = direction, t = parameter)
///
/// Substitute ray into sphere:
/// |O + t*D - C|² = r²
///
/// Expand:
/// (O + t*D - C)·(O + t*D - C) = r²
///
/// Let oc = O - C:
/// (oc + t*D)·(oc + t*D) = r²
/// oc·oc + 2*t*(D·oc) + t²*(D·D) = r²
///
/// Rearrange to quadratic: at² + bt + c = 0
/// where:
///   a = D·D  (usually 1 if D is normalized)
///   b = 2*(D·oc)
///   c = oc·oc - r²
///
/// Solve using quadratic formula:
/// t = (-b ± sqrt(b² - 4ac)) / 2a
///
/// Discriminant = b² - 4ac:
///   < 0: no intersection
///   = 0: tangent (1 solution)
///   > 0: two solutions (entry and exit points)
__device__ inline bool intersect_sphere(const Ray &ray, const GpuSphere &sphere,
                                        float t_min, float t_max,
                                        float *t_out) {
  // Vector from ray origin to sphere center
  float3 oc = ray.origin - sphere.center;

  // Quadratic equation coefficients
  // a = dot(D, D), but if ray.direction is normalized, a = 1
  // We'll compute it anyway for robustness
  float a = dot(ray.direction, ray.direction);
  float half_b = dot(oc, ray.direction); // Optimization: use b/2
  float c = dot(oc, oc) - sphere.radius * sphere.radius;

  // Discriminant
  float discriminant = half_b * half_b - a * c;

  // No intersection if discriminant < 0
  if (discriminant < 0.0f) {
    return false;
  }

  // Find nearest root that lies in acceptable range [t_min, t_max]
  float sqrt_d = sqrtf(discriminant);

  // Try first root (nearest intersection)
  float root = (-half_b - sqrt_d) / a;

  // Check if in valid range
  if (root < t_min || root > t_max) {
    // Try second root (exit point)
    root = (-half_b + sqrt_d) / a;

    if (root < t_min || root > t_max) {
      return false; // Both roots outside valid range
    }
  }

  // Valid intersection found!
  *t_out = root;
  return true;
}

/// Compute sphere normal at hit point
__device__ inline float3 sphere_normal(const float3 &hit_point,
                                       const GpuSphere &sphere) {
  return normalize(hit_point - sphere.center);
}

// ============================================================================
// Sampling Utilities
// ============================================================================

/// Sample random point on unit sphere (cosine-weighted hemisphere)
/// Used for Lambertian BRDF sampling
/// Input: u1, u2 = random numbers in [0, 1]
/// Output: direction in local space (normal = +Z)
__device__ inline float3 sample_cosine_hemisphere(float u1, float u2) {
  // Malley's method (project disk to hemisphere)
  float r = sqrtf(u1);
  float theta = 2.0f * M_PI * u2;

  float x = r * cosf(theta);
  float y = r * sinf(theta);
  float z = sqrtf(fmaxf(0.0f, 1.0f - u1)); // Ensure non-negative

  return make_float3(x, y, z);
}

/// Probability density function for cosine-weighted hemisphere sampling
__device__ inline float cosine_hemisphere_pdf(float cos_theta) {
  return cos_theta * M_1_PI; // cos(θ) / π
}

#endif // COMMON_CUH
