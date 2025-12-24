# Raytracer Implementation Guide

**Status**: Phase 2 complete, renderer working, core features built but underutilized
**Current State**: Can render scenes with point lights and Lambertian materials. Advanced materials (GGX, Dielectric) and area lights implemented but not integrated into examples.

---

## Architecture Overview

### Rendering Pipeline

```
Camera → Renderer → Scene → BVH → Primitive::intersect() → Intersection {material, ...}
                                                                    ↓
                                              Material::compute_scattering_functions()
                                                                    ↓
                                                            BxDF::sample_f()
                                                                    ↓
                                        Direct Lighting (sample lights) + Indirect (BSDF sample)
```

### Key Systems

**Material Dispatch** (`src/materials/material.rs`):

- `PbrMaterial` evaluates textures at UV, then chooses BxDF:
  - `metallic > 0.5` → `GgxBxDF` (microfacet metal)
  - `roughness < 0.1 && metallic < 0.5` → `DielectricBxDF` (glass)
  - Otherwise → `LambertianBxDF` (diffuse)
- Current limitation: Hard thresholds instead of blending

**Scene Graph** (`src/scene_graph/`):

- Arena storage via `HashMap<NodeId, SceneNode>`
- Transform propagation with dirty flags
- `build_bvh()` flattens hierarchy into BVH with world-space transforms
- Lights extracted during build into `Vec<Arc<dyn Light>>`

**BVH Acceleration** (`src/acceleration/bvh.rs`):

- Recursive enum: `BvhNode::Internal{left, right, bbox}` | `BvhNode::Leaf{primitives}`
- Split strategy: Random axis selection (line 92) - not optimal
- Both `intersect()` and `intersect_p()` (shadow rays) implemented

**Lighting Model** (`src/renderer.rs::ray_color()`):

- Direct: Loop over `scene.lights()`, sample each, cast shadow ray
- Indirect: Sample BSDF, recurse with depth-1
- Local shading space: normal = +Z, tangent = +X, bitangent = +Y
- Current issue: No MIS weighting, treats direct/indirect independently

---

## Quick Wins - Built Features Not Demonstrated

### 1. Enable Area Lights

**Problem**: Renderer skips area lights (line 201 in `renderer.rs`)

```rust
if light.is_delta() {
    continue;  // This skips area lights!
}
```

**Solution**: Remove the skip, area lights already return correct `LightSample`

**Files**:

- `src/renderer.rs:201` - Remove delta check
- `src/lights/area_light.rs` - Already complete with surface sampling + PDF conversion
- Example usage: `examples/simple_render.rs` - add `AreaLight::new(rect_primitive, emission)`

**Implementation notes**:

- AreaLight wraps any `Primitive` (Rect, Sphere, etc.)
- `sample_li()` returns area PDF converted to solid angle
- Shadow ray should stop at `distance - epsilon` to avoid self-intersection

---

### 2. Create Advanced Materials Example

**Problem**: GGX and Dielectric exist but no examples use them

**What to show**:

- Metallic sphere using GGX (highlights, anisotropy)
- Glass sphere using Dielectric (refraction, Fresnel, TIR)
- Side-by-side comparison with Lambertian

**Implementation**:
Create `examples/materials_showcase.rs`:

```rust
// Metal
let metal_mat = Arc::new(PbrMaterial::new(
    Vec3::new(1.0, 0.8, 0.3),  // gold albedo
    1.0,  // metallic = 1.0 triggers GGX
    0.2,  // roughness controls highlight size
));

// Glass
let glass_mat = Arc::new(PbrMaterial::new(
    Vec3::new(1.0, 1.0, 1.0),
    0.0,  // metallic = 0.0
    0.01, // roughness < 0.1 triggers Dielectric
).with_ior(1.5));  // glass IOR

// Diffuse comparison
let diffuse_mat = Arc::new(DiffuseMaterial::new(Vec3::new(0.8, 0.2, 0.2)));
```

**Files**:

- `src/materials/bxdf/ggx.rs` - Complete microfacet implementation (D, G, F terms, VNDF sampling)
- `src/materials/bxdf/dielectric.rs` - Complete Fresnel + Snell's law
- `src/materials/material.rs:87-105` - Dispatch logic in `PbrMaterial::compute_scattering_functions()`

**Gotchas**:

- GGX requires `metallic > 0.5` - design choice, could be adjusted
- Dielectric requires `roughness < 0.1 AND metallic < 0.5` - hard threshold
- Glass spheres need high sample count to converge (caustics are noisy)
- Sky background provides environment lighting for Fresnel reflections

---

### 3. Texture-Mapped Materials

**Problem**: Texture system exists (Constant, Checker, ImageTexture) but only ConstantTexture used

**What works**:

- `ImageTexture::load(path)` - loads PNG/JPG via `image` crate
- `CheckerTexture::new(color1, color2, scale)` - procedural pattern
- UV coordinates provided by all primitives

**Implementation**:

```rust
use raytracing::textures::{ImageTexture, CheckerTexture};

// Checker pattern
let checker: Arc<dyn Texture> = Arc::new(CheckerTexture::new(
    Vec3::new(0.2, 0.2, 0.2),
    Vec3::new(0.8, 0.8, 0.8),
    2.0  // scale
));

let checker_mat = Arc::new(PbrMaterial::with_textures(
    checker.clone(),  // albedo
    ConstantTexture::new(Vec3::new(0.0, 0.0, 0.0)),  // metallic
    ConstantTexture::new(Vec3::new(0.8, 0.8, 0.8)),  // roughness
));

// Image texture
let wood_tex = Arc::new(ImageTexture::load("assets/wood.png")?);
```

**Files**:

- `src/textures/image_texture.rs:11-45` - Load and sample images
- `src/textures/checker.rs:8-28` - Procedural checker
- `src/materials/material.rs:59-71` - `PbrMaterial::with_textures()`

**Gotchas**:

- Image V coordinate is flipped (line 37: `let v = 1.0 - v`)
- UV wrapping uses `.fract()` for repeat (line 38-39)
- Bilinear filtering not implemented (nearest neighbor sampling)

---

## High-Impact Features - Missing Critical Pieces

### 4. Multiple Importance Sampling (MIS)

**Problem**: Current renderer samples lights AND BSDF independently, causing high variance on specular surfaces

**Current behavior** (`src/renderer.rs:198-232`):

- Direct lighting: Sample all lights, evaluate BSDF at those directions
- Indirect lighting: Sample BSDF, recurse
- Issue: Double counts some paths, misses others, high fireflies

**What MIS does**:
Balance two sampling strategies using power heuristic:

```
weight_light = (n_light * pdf_light)^β / [(n_light * pdf_light)^β + (n_bsdf * pdf_bsdf)^β]
weight_bsdf = (n_bsdf * pdf_bsdf)^β / [(n_light * pdf_light)^β + (n_bsdf * pdf_bsdf)^β]

β = 2  // balance heuristic (optimal for most cases)
```

**Implementation strategy**:

1. **Add balance heuristic helper** (new function in `renderer.rs`):

```rust
fn power_heuristic(pdf_a: f64, pdf_b: f64) -> f64 {
    let a2 = pdf_a * pdf_a;
    let b2 = pdf_b * pdf_b;
    a2 / (a2 + b2)
}
```

2. **Modify direct lighting** (line 198-232):

```rust
// Sample light
let light_pdf = light_sample.pdf;
let bsdf_pdf = bxdf.pdf(wo_local, wi_local);  // NEW: evaluate BSDF PDF for this direction
let mis_weight = power_heuristic(light_pdf, bsdf_pdf);

color += mis_weight * f * light_sample.radiance * (cos_theta / light_pdf);
```

3. **Modify indirect lighting** (line 234-254):

```rust
// Sample BSDF
let bsdf_pdf = sample.pdf;

// NEW: Check if sampled direction hits a light
let mut light_pdf = 0.0;
if let Some(hit_isect) = world.intersect(&scattered, 0.001, f64::INFINITY) {
    if hit_isect.material.is_emissive() {
        // Evaluate probability of sampling this via light
        for light in world.lights() {
            light_pdf += light.pdf_li(isect.point, wi_world);
        }
        light_pdf /= world.lights().len() as f64;
    }
}

let mis_weight = if light_pdf > 0.0 {
    power_heuristic(bsdf_pdf, light_pdf)
} else {
    1.0  // No light hit, use full BSDF contribution
};

color += mis_weight * sample.f * incoming_radiance * (cos_theta / bsdf_pdf);
```

**Files**:

- `src/renderer.rs:177-263` - Main `ray_color()` function
- `src/lights/light.rs:39` - `pdf_li()` already implemented for all light types
- `src/materials/bxdf/bsdf.rs:56` - `BxDF::pdf()` already implemented

**References**:

- Veach thesis Chapter 9: Multiple Importance Sampling
- Balance heuristic is optimal for 2 sampling techniques
- Need both `Light::pdf_li()` and `BxDF::pdf()` - already implemented

**Expected impact**:

- Reduces variance by ~50% (need half as many samples)
- Eliminates fireflies on mirror/glass surfaces
- Slight overhead per sample (~10-15%) but worth the quality gain

---

### 5. Russian Roulette Path Termination

**Problem**: Hard depth limit causes bias (paths > max_depth contribute 0)

**Current** (`src/renderer.rs:179-181`):

```rust
if depth <= 0 {
    return Vec3::ZERO;  // Bias: ignores energy below depth
}
```

**Improved**:

```rust
// Start RR after min depth
const MIN_DEPTH: i32 = 3;
const MAX_DEPTH: i32 = 10;

if depth < MIN_DEPTH {
    // Always continue for first few bounces
} else if depth >= MAX_DEPTH {
    // Hard cap to prevent infinite loops
    return Vec3::ZERO;
} else {
    // Russian roulette
    let throughput = color.length();  // Current path throughput
    let survival_prob = throughput.min(0.95);  // Cap at 95% to avoid low probabilities

    if rng.random::<f64>() > survival_prob {
        return Vec3::ZERO;  // Terminate path
    }

    // Continue path with boosted contribution
    color = color / survival_prob;  // Unbiased estimator
}
```

**Implementation notes**:

- Start RR after 3-5 bounces (most energy captured)
- Use path throughput (albedo × albedo × ...) as survival probability
- Cap probability at 0.95 to avoid very low chances
- Divide contribution by survival probability to remain unbiased
- Still need hard max depth to prevent infinite loops (set to 10-20)

**Files**:

- `src/renderer.rs:179` - Depth check location
- Single function change, no new dependencies

**Expected impact**:

- 20-30% faster convergence
- Properly samples long light paths without bias
- No visual quality loss (mathematically unbiased)

---

### 6. Complete Instance Rendering

**Problem**: Instance system exists but material override doesn't work (line 198 in `scene.rs`)

**Current behavior** (`src/scene.rs:180-210`):

```rust
NodeContent::Instance { template, override_material } => {
    // ...
    let instance_material = override_material.as_ref().unwrap_or(material);
    // TODO: Create new primitive with instance material if overridden
    // Currently ignores override_material!
}
```

**Issue**: `TransformedPrimitive` wraps the primitive but doesn't allow material override

**Solution approaches**:

**Option A**: Material at intersection time (cleaner)

- Store material separately in BVH leaf
- `Intersection` already carries `Arc<dyn Material>`
- Modify `BvhNode::Leaf` to store `Vec<(Arc<dyn Primitive>, Arc<dyn Material>)>`

**Option B**: Wrapper primitive (current architecture)

- Create `MaterialOverridePrimitive` wrapper
- Wraps both primitive and material
- Returns modified `Intersection` with new material

**Recommended: Option A** (less object wrapping, cleaner architecture)

**Implementation**:

```rust
// In acceleration/bvh.rs
pub enum BvhNode {
    Leaf {
        primitives: Vec<(Arc<dyn Primitive>, Arc<dyn Material>)>,  // Changed
        bbox: AABB,
    },
    // ...
}

// In scene.rs build_bvh()
let primitive = TransformedPrimitive::new(prim.clone(), world_transform);
let material_to_use = override_material.unwrap_or(material.clone());
flattened.push((Arc::new(primitive), material_to_use));

// In bvh.rs intersect()
for (primitive, material) in &self.primitives {
    if let Some(mut isect) = primitive.intersect(ray, t_min, closest) {
        isect.material = material.clone();  // Override material
        // ...
    }
}
```

**Files**:

- `src/acceleration/bvh.rs:15-29` - BvhNode enum definition
- `src/acceleration/bvh.rs:60-85` - Leaf intersection
- `src/scene.rs:180-210` - Instance processing in build_bvh

**Gotchas**:

- Must clone material Arc at intersection (cheap, just refcount)
- Affects BVH serialization if added later
- Template geometry still shares underlying primitive (memory efficient)

---

## Performance Optimizations

### 7. Surface Area Heuristic (SAH) for BVH

**Problem**: Random axis selection for BVH splits (line 92 in `bvh.rs`) is suboptimal

**Current**:

```rust
let axis = rand::rng().random_range(0..3);  // Random
let mid = primitives.len() / 2;
primitives.sort_by(|a, b| /* sort on axis */);
```

**SAH algorithm**:
For each axis and each possible split position, compute cost:

```
cost = C_traversal +
       (surface_area_left / surface_area_parent) * cost_left +
       (surface_area_right / surface_area_parent) * cost_right

cost_left/right = num_primitives * C_intersect
```

Choose split with minimum cost.

**Implementation**:

```rust
fn find_best_split(primitives: &[(Arc<dyn Primitive>, Arc<dyn Material>)],
                   parent_bbox: &AABB) -> (usize, f64) {
    const NUM_BUCKETS: usize = 12;
    const C_TRAVERSAL: f64 = 1.0;
    const C_INTERSECT: f64 = 1.0;

    let mut best_cost = f64::INFINITY;
    let mut best_axis = 0;
    let mut best_split = 0;

    for axis in 0..3 {
        // Bucket primitives by centroid along axis
        let mut buckets = vec![Bucket::new(); NUM_BUCKETS];

        for (prim, _mat) in primitives {
            let bbox = prim.world_bounds();
            let centroid = bbox.center();
            let bucket_idx = /* map centroid[axis] to bucket */;
            buckets[bucket_idx].add(prim);
        }

        // Try all split positions
        for split in 1..NUM_BUCKETS {
            let (left_bbox, left_count) = /* combine buckets 0..split */;
            let (right_bbox, right_count) = /* combine buckets split..NUM_BUCKETS */;

            let cost = C_TRAVERSAL +
                (left_bbox.surface_area() / parent_bbox.surface_area()) * left_count * C_INTERSECT +
                (right_bbox.surface_area() / parent_bbox.surface_area()) * right_count * C_INTERSECT;

            if cost < best_cost {
                best_cost = cost;
                best_axis = axis;
                best_split = split;
            }
        }
    }

    (best_axis, best_split)
}
```

**Files**:

- `src/acceleration/bvh.rs:85-110` - `new()` constructor
- `src/core/math/aabb.rs` - Add `surface_area()` method if not present

**Implementation notes**:

- Bucket approach is O(n log n), faster than testing all positions
- 12 buckets is good balance (PBRT uses 12)
- Can fall back to median split if SAH suggests no split
- Consider leaf threshold (4-8 primitives per leaf)

**Expected impact**: 30-50% faster ray traversal, worth the build time cost

---

### 8. Material Blending

**Problem**: Hard thresholds in `PbrMaterial::compute_scattering_functions()` (line 95-104)

**Current logic**:

```rust
if metallic_value > 0.5 {
    Box::new(GgxBxDF::new(albedo, roughness_value))
} else if roughness_value < 0.1 {
    Box::new(DielectricBxDF::new(self.ior))
} else {
    Box::new(LambertianBxDF::new(albedo))
}
```

**Issue**: Real materials are combinations (rough metal, frosted glass, painted plastic)

**Proper approach**:
Create `MixedBxDF` that evaluates/samples multiple BxDFs with weights:

```rust
pub struct MixedBxDF {
    bxdfs: Vec<(Box<dyn BxDF>, f64)>,  // (bxdf, weight)
}

impl BxDF for MixedBxDF {
    fn f(&self, wo: Vec3, wi: Vec3) -> Vec3 {
        self.bxdfs.iter()
            .map(|(bxdf, weight)| bxdf.f(wo, wi) * *weight)
            .sum()
    }

    fn sample_f(&self, wo: Vec3, u: Vec2) -> Option<BxDFSample> {
        // Select BxDF based on weights
        let r = u.x;
        let mut cumulative = 0.0;

        for (bxdf, weight) in &self.bxdfs {
            cumulative += weight;
            if r < cumulative {
                // Sample selected BxDF, reweight u.x
                let remapped_u = Vec2::new((r - (cumulative - weight)) / weight, u.y);
                let mut sample = bxdf.sample_f(wo, remapped_u)?;

                // MIS: weight by all BxDF PDFs
                let pdf_sum: f64 = self.bxdfs.iter()
                    .map(|(b, w)| b.pdf(wo, sample.wi) * w)
                    .sum();
                sample.pdf = pdf_sum;

                return Some(sample);
            }
        }
        None
    }
}
```

**Usage**:

```rust
// Smooth transition metallic 0.0 -> 1.0
let diffuse_weight = 1.0 - metallic_value;
let metal_weight = metallic_value;

let mut mixed = MixedBxDF::new();
if diffuse_weight > 0.01 {
    mixed.add(Box::new(LambertianBxDF::new(albedo)), diffuse_weight);
}
if metal_weight > 0.01 {
    mixed.add(Box::new(GgxBxDF::new(albedo, roughness)), metal_weight);
}

// Can also layer: diffuse base + specular coat
mixed.add(Box::new(LambertianBxDF::new(base_color)), 0.7);
mixed.add(Box::new(GgxBxDF::new(Vec3::ONE, 0.1)), 0.3);  // Clear coat
```

**Files**:

- New file: `src/materials/bxdf/mixed.rs`
- `src/materials/material.rs:87-105` - Update PbrMaterial dispatch

**Implementation notes**:

- Weights must sum to 1.0 (normalized)
- PDF calculation uses weighted sum of all component PDFs (MIS within material)
- Sample selection is importance-driven by weights
- Can represent any physical material as combination of basis BxDFs

**Complexity**: This is the proper solution but requires careful PDF handling. Start simple (two-component mix) before generalizing.

---

## Development Workflow

### Running Examples

```bash
# Build and run
cargo run --release --example simple_render

# Just build
cargo build --release --example simple_render

# View output (Linux)
eog simple_render.ppm
```

### Testing

```bash
# All tests
cargo test

# Specific module
cargo test materials::bxdf::ggx

# With output
cargo test -- --nocapture
```

### Performance Profiling

```bash
# Flamegraph (requires cargo-flamegraph)
cargo flamegraph --example simple_render

# Time breakdown
cargo build --release && time ./target/release/examples/simple_render
```

---

## Common Gotchas

**BxDF Local Space Convention**:

- All BxDF calculations use local shading space
- Normal always points along +Z
- Must transform `wo` and `wi` to/from world space
- Functions: `world_to_local()` and `local_to_world()` in renderer.rs

**Material vs BxDF**:

- Material: Evaluates textures, chooses BxDF based on parameters
- BxDF: Pure scattering function, no surface properties
- Separation allows composition and reuse

**Shadow Ray Epsilon**:

- Use `0.001` for t_min to avoid self-intersection
- For shadow rays to lights, use `distance - epsilon` for t_max

**Transform Caching**:

- Scene graph has dirty flags for transforms
- `build_bvh()` recomputes world transforms for dirty nodes
- Don't call `build_bvh()` every frame if scene is static

**PDF Units**:

- Area lights: Convert area PDF → solid angle PDF (multiply by distance² / cos)
- BxDF: Always solid angle PDF
- Balance heuristic requires same units for both PDFs

**Parallel Rendering**:

- Each thread gets own RNG (seeded differently)
- Rayon handles work distribution
- Order of scanline output is deterministic despite parallelism

---

## File Map - Where to Find Things

### Core Rendering

- `src/renderer.rs` - Main render loop, ray_color() path tracer
- `src/camera.rs` - Ray generation
- `src/ray.rs` - Ray definition
- `src/image.rs` - PPM export

### Geometry

- `src/geometry/primitive.rs` - Primitive trait + TransformedPrimitive
- `src/geometry/sphere.rs` - Sphere with UV mapping
- `src/geometry/rect.rs` - Axis-aligned rectangles
- `src/geometry/plane.rs` - Infinite plane
- `src/geometry/box_primitive.rs` - Box from 6 rects

### Materials & BxDF

- `src/materials/material.rs` - Material trait, PbrMaterial
- `src/materials/bxdf/bsdf.rs` - BxDF trait
- `src/materials/bxdf/lambertian.rs` - Diffuse BRDF
- `src/materials/bxdf/ggx.rs` - Microfacet metal BRDF
- `src/materials/bxdf/dielectric.rs` - Glass/transparent BSDF

### Lighting

- `src/lights/light.rs` - Light trait, LightSample
- `src/lights/point_light.rs` - Delta point lights
- `src/lights/area_light.rs` - Primitive-based area lights

### Scene Management

- `src/scene.rs` - Scene with BVH + lights
- `src/scene_graph/scene_node.rs` - Hierarchical nodes
- `src/scene_graph/instance.rs` - Instancing data
- `src/acceleration/bvh.rs` - BVH acceleration

### Math & Utils

- `src/vec3.rs` - 3D vector
- `src/core/math/` - Transform, Mat4, Quaternion, AABB
- `src/core/intersection.rs` - Rich intersection record

### Textures

- `src/textures/texture.rs` - Texture trait
- `src/textures/constant.rs` - Solid color
- `src/textures/checker.rs` - Procedural checker
- `src/textures/image_texture.rs` - Image file loading

---

## Next Steps Summary

**Immediate** (unlock existing features):

1. Remove area light skip in renderer
2. Create materials showcase example (GGX + Dielectric)
3. Add textured materials example

**High Impact** (quality improvements):
4. Implement MIS for variance reduction
5. Add Russian roulette termination
6. Complete instance material override

**Performance** (speed improvements):
7. SAH for BVH building
8. Material blending/layering system

Each task builds on working infrastructure. The foundation is solid - now it's about integration and refinement.
