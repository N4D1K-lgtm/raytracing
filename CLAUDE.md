# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Test Commands

```bash
# Build the project
cargo build
cargo build --release

# Run tests
cargo test
cargo test --lib                    # Library tests only
cargo test materials::bxdf::ggx    # Specific module

# Run offline examples (render to file)
cargo run --release --example simple_render
cargo run --release --example materials_showcase

# Run interactive example (real-time window)
cargo run --release --example interactive

# View rendered output (PPM files)
eog simple_render.ppm              # Linux
open simple_render.ppm             # macOS
```

## Architecture Overview

This is a physically-based path tracer written in Rust. The codebase is in **Phase 2 complete** state - advanced features (GGX materials, area lights, scene graph) are implemented but underutilized in examples.

### Rendering Pipeline Flow

```
Camera → Renderer → Scene → BVH → Primitive::intersect() → Intersection
                                                                ↓
                                    Material::compute_scattering_functions()
                                                                ↓
                                                        BxDF::sample_f()
                                                                ↓
                            Direct Lighting (sample lights) + Indirect (BSDF sample)
```

### Key Architectural Concepts

**Material vs BxDF Separation**:
- **Material** (`src/materials/material.rs`): Evaluates textures at surface point, selects appropriate BxDF based on material parameters
- **BxDF** (`src/materials/bxdf/`): Pure bidirectional scattering function - handles light transport math
- This separation enables composition and reuse

**Scene Graph System** (`src/scene_graph/`):
- Arena-based storage using `HashMap<NodeId, SceneNode>`
- Hierarchical transforms with dirty flags for efficient updates
- `build_bvh()` flattens hierarchy into BVH with baked world-space transforms
- Lights are extracted during BVH build into `Vec<Arc<dyn Light>>`

**BVH Acceleration** (`src/acceleration/bvh.rs`):
- Recursive enum structure: `BvhNode::Internal{left, right, bbox}` | `BvhNode::Leaf{primitives}`
- Currently uses random axis selection for splits (not optimal - SAH would improve this)
- Both `intersect()` and `intersect_p()` (shadow ray optimization) implemented

**Coordinate Spaces**:
- All BxDF calculations use **local shading space** where normal = +Z, tangent = +X, bitangent = +Y
- Must transform `wo` (outgoing) and `wi` (incoming) vectors to/from world space
- Helper functions: `world_to_local()` and `local_to_world()` in `renderer.rs`

### Material Dispatch Logic

`PbrMaterial` (`src/materials/material.rs:87-105`) uses hard thresholds to select BxDF:
- `metallic > 0.5` → `GgxBxDF` (microfacet metal with VNDF sampling)
- `roughness < 0.1 && metallic < 0.5` → `DielectricBxDF` (glass/refraction)
- Otherwise → `LambertianBxDF` (diffuse)

**Known limitation**: Hard thresholds instead of blending. Real materials need `MixedBxDF` for layering (e.g., rough metal, clear coat).

### Lighting Model

Direct lighting (`src/renderer.rs:198-232`):
- Loop over `scene.lights()`, sample each light
- Cast shadow ray to verify visibility
- Evaluate BSDF at sampled direction

Indirect lighting (`src/renderer.rs:234-254`):
- Sample BSDF for scattered direction
- Recursively trace ray with `depth-1`

**Known issue**: No Multiple Importance Sampling (MIS) - treats direct/indirect independently, causing high variance on specular surfaces.

## Module Organization

### Core Rendering
- `src/renderer.rs` - Main render loop, `ray_color()` path tracer with parallel rendering
- `src/camera.rs` - Ray generation from camera parameters
- `src/image.rs` - PPM image export

### Geometry System
- `src/geometry/primitive.rs` - `Primitive` trait + `TransformedPrimitive` wrapper
- `src/geometry/sphere.rs` - Analytic sphere with UV mapping
- `src/geometry/rect.rs` - Axis-aligned rectangles (for area lights)
- `src/geometry/plane.rs` - Infinite plane
- `src/geometry/box_primitive.rs` - Box constructed from 6 rectangles

### Materials & Scattering
- `src/materials/material.rs` - `Material` trait, `PbrMaterial`, `DiffuseMaterial`
- `src/materials/bxdf/bsdf.rs` - `BxDF` trait definition
- `src/materials/bxdf/lambertian.rs` - Cosine-weighted diffuse BRDF
- `src/materials/bxdf/ggx.rs` - Microfacet metal BRDF (Smith G, GGX D, Fresnel conductor)
- `src/materials/bxdf/dielectric.rs` - Fresnel glass BSDF with Snell's law refraction

### Lighting
- `src/lights/light.rs` - `Light` trait, `LightSample` struct
- `src/lights/point_light.rs` - Delta point lights (infinite small)
- `src/lights/area_light.rs` - Primitive-based area lights with importance sampling

### Scene Management
- `src/scene.rs` - `Scene` struct managing BVH + lights
- `src/scene_graph/scene_node.rs` - Hierarchical scene nodes
- `src/scene_graph/instance.rs` - Instancing data (geometry reuse with transforms)
- `src/acceleration/bvh.rs` - BVH acceleration structure

### Textures
- `src/textures/texture.rs` - `Texture` trait
- `src/textures/constant.rs` - Solid color texture
- `src/textures/checker.rs` - Procedural checkerboard
- `src/textures/image_texture.rs` - PNG/JPG loading via `image` crate

## Important Implementation Details

**Shadow Ray Epsilon**:
- Always use `t_min = 0.001` to avoid self-intersection
- For shadow rays to lights, use `t_max = distance - epsilon`

**Transform Caching**:
- Scene graph uses dirty flags to track transform changes
- `build_bvh()` recomputes world transforms only for dirty nodes
- Don't call `build_bvh()` every frame if scene is static

**PDF Units**:
- Area lights convert area PDF → solid angle PDF (multiply by `distance² / cos_theta`)
- BxDF always returns solid angle PDF
- When implementing MIS, ensure both PDFs use same units

**Parallel Rendering**:
- Uses Rayon for parallel scanline rendering
- Each thread gets its own RNG (seeded differently)
- Scanline output order is deterministic despite parallelism

**Gamma Correction**:
- Applied in `renderer.rs:83-85` as `sqrt()` (gamma 2.0)
- Applied per-channel before writing to PPM

## Known Issues and Quick Wins

### Area Lights Not Working
**Location**: `src/renderer.rs:201`
```rust
if light.is_delta() {
    continue;  // This incorrectly skips area lights!
}
```
**Fix**: Remove or invert this check. Area lights are already fully implemented in `src/lights/area_light.rs` with proper surface sampling and PDF conversion.

### Advanced Materials Exist But Unused
GGX (microfacet metal) and Dielectric (glass) BxDFs are complete but no examples demonstrate them.

**To use GGX**:
```rust
let metal_mat = Arc::new(PbrMaterial::new(
    Vec3::new(1.0, 0.8, 0.3),  // gold albedo
    1.0,   // metallic = 1.0 triggers GGX
    0.2,   // roughness controls highlight sharpness
));
```

**To use Dielectric**:
```rust
let glass_mat = Arc::new(PbrMaterial::new(
    Vec3::new(1.0, 1.0, 1.0),
    0.0,   // metallic = 0.0
    0.01,  // roughness < 0.1 triggers Dielectric
).with_ior(1.5));
```

### Texture System Underutilized
`ImageTexture` and `CheckerTexture` exist but only `ConstantTexture` is used in examples.

**Image textures**:
- V coordinate is flipped: `let v = 1.0 - v` (`image_texture.rs:37`)
- Uses nearest-neighbor sampling (no bilinear filtering)
- UV wrapping via `.fract()` for repeat mode

### Instance Material Override Broken
**Location**: `src/scene.rs:198`

Scene graph instances can reference geometry templates with material overrides, but the override is currently ignored. The `TransformedPrimitive` wrapper doesn't support material replacement.

**Recommended fix**: Store `(primitive, material)` tuples in BVH leaves instead of just primitives, allowing material override at intersection time.

## Performance Notes

**BVH Split Strategy**: Currently uses random axis selection (`src/acceleration/bvh.rs:92`). Surface Area Heuristic (SAH) would improve traversal by 30-50%.

**Missing MIS**: Direct and indirect lighting are sampled independently without weighting. Implementing power heuristic balance would reduce variance by ~50%.

**No Russian Roulette**: Hard depth limit (`max_depth`) causes bias. RR path termination with throughput-based survival probability would be unbiased and faster.

## Common Gotchas

- **BxDF space**: All scattering calculations assume normal = +Z. Always transform to/from world space.
- **Material selection thresholds**: `metallic > 0.5` and `roughness < 0.1` are hard-coded. Not physically accurate, just design choices.
- **Glass convergence**: Dielectric materials create caustics that need high sample counts (500+) to converge.
- **Texture V flip**: Image V coordinate is inverted to match standard texture coordinate conventions.
- **Build before render**: Must call `scene.build_bvh()` after adding geometry and before rendering.

## Development Workflow

When adding new features:
1. Geometry goes in `src/geometry/` implementing `Primitive` trait
2. Materials go in `src/materials/` implementing `Material` trait
3. BxDFs go in `src/materials/bxdf/` implementing `BxDF` trait
4. Lights go in `src/lights/` implementing `Light` trait
5. Examples go in `examples/` and should call `scene.build_bvh()` before rendering

The renderer uses Rayon for parallelism automatically - no special handling needed.

## Interactive/Online Rendering

The codebase supports both offline (render-to-file) and online (interactive window) rendering:

**Key Components**:
- `src/app.rs` - Application state for interactive rendering
- `src/accumulator.rs` - Progressive sample accumulation
- `src/camera_controller.rs` - FPS-style camera controls

**Progressive Rendering Flow**:
1. `render_sample()` generates one sample per pixel
2. `Accumulator` sums samples and tracks convergence
3. On camera movement, accumulator resets and starts fresh
4. Quality presets control target sample count

**Interactive Example** (`examples/interactive.rs`):
- Real-time window using `winit` + `pixels`
- WASD + mouse for FPS-style navigation
- Progressive accumulation with live preview
- Dynamic scene editing (add/remove spheres)
- Quality presets (fast/medium/high)

**Usage**:
```bash
cargo run --release --example interactive

# Controls:
# WASD - Move camera
# Mouse - Look around
# Space/Shift - Up/Down
# R - Add random sphere
# 1/2/3 - Fast/Medium/High quality
# ESC - Exit
```

**Performance Considerations**:
- Fast quality (16 samples) for real-time navigation
- Medium (64 samples) for preview
- High (256 samples) for final quality
- MIS and Russian Roulette work seamlessly in progressive mode
- Lower samples per `render_sample()` call for faster feedback
