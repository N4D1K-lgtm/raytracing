# Production Raytracer - Complete Product Roadmap

**Status**: MVP → Production-Ready Path Tracer
**Current Completion**: ~40% (Phases 1-2 complete)
**Target**: Feature-complete, production-grade renderer

---

## 📊 Current State Assessment

### ✅ **Completed** (Phases 1-2)

**Foundation** (Phase 1):
- ✅ Complete math infrastructure (Vec2, Vec3, Mat4, Quaternion, Transform, AABB)
- ✅ Geometric primitives (Sphere, Plane, Rect, Box)
- ✅ BVH acceleration structure
- ✅ Transform system with caching
- ✅ Rich intersection records (UV, tangents)

**Scene & Materials** (Phase 2):
- ✅ Hierarchical scene graph with parent-child transforms
- ✅ Geometry instancing with material overrides
- ✅ BxDF trait system (Lambertian complete, GGX/Dielectric stubs)
- ✅ PBR material system (metallic/roughness workflow)
- ✅ Texture system (constant, checker, image loading)
- ✅ Lighting system (point lights, area lights)
- ✅ 65 passing tests
- ✅ Example demos

### 🔧 **Needs Completion** (Technical Debt)

**High Priority:**
1. ❌ Complete GGX microfacet BRDF (currently stub)
2. ❌ Complete Dielectric BSDF with refraction (currently stub)
3. ❌ Integrate new scene graph into renderer (still uses old API)
4. ❌ Update Camera to work with new system
5. ❌ Material composition (layering multiple BxDFs)

**Medium Priority:**
6. ❌ Transform support for lights
7. ❌ Material override for instances (marked TODO)
8. ❌ SAH-based BVH construction
9. ❌ Normal mapping support
10. ❌ Russian roulette path termination

### 🚫 **Missing Features** (Phases 3-6)

- ❌ Mesh primitives (triangles)
- ❌ Mesh loading (OBJ, PLY)
- ❌ Advanced integrators (BDPT, VCM)
- ❌ Volumetric rendering
- ❌ Scene file format
- ❌ OpenEXR output
- ❌ Subsurface scattering
- ❌ Spectral rendering
- ❌ Denoising
- ❌ Depth of field
- ❌ CLI interface
- ❌ Comprehensive documentation

---

## 🎯 Revised Product Plan

### **Phase 2.5: Complete Core Rendering** (1-2 weeks)

**Goal**: Finish all stub implementations and integrate new systems

#### Critical Path Items

1. **Complete BxDF Implementations** (Priority 1)
   - Implement full GGX microfacet model
     - GGX normal distribution function (D term)
     - Smith geometric attenuation (G term)
     - Schlick Fresnel approximation (F term)
     - Importance sampling with visible normal distribution
   - Implement Dielectric BSDF
     - Fresnel equations (Schlick approximation)
     - Snell's law refraction
     - Total internal reflection handling
     - Reflection/refraction selection
   - **Est**: 250-300 LOC, 10-15 tests
   - **Value**: Realistic glass and metallic materials

2. **Integrate Scene Graph into Renderer** (Priority 2)
   - Update Renderer to use Scene's new API
   - Replace Hittable with Intersection
   - Use scene.intersect() instead of old hit()
   - Access scene.lights() for light sampling
   - Update all examples to use new API
   - **Est**: 150-200 LOC refactor
   - **Value**: Enables all Phase 2 features in actual rendering

3. **Direct Lighting with MIS** (Priority 3)
   - Sample lights explicitly (area lights + point lights)
   - BSDF importance sampling
   - Multiple Importance Sampling (balance heuristic)
   - Shadow rays for visibility
   - **Est**: 200-250 LOC
   - **Value**: Dramatic quality improvement, proper light handling

4. **Russian Roulette Termination** (Priority 4)
   - Replace fixed max_depth with RR after 3-5 bounces
   - Use throughput as survival probability
   - Unbiased path termination
   - **Est**: 50 LOC
   - **Value**: Better performance, unbiased rendering

5. **Material Composition** (Priority 5)
   - Support layered materials (diffuse + clear coat)
   - Blend multiple BxDFs
   - Fresnel-weighted mixing
   - **Est**: 100-150 LOC
   - **Value**: Complex realistic materials

**Deliverables**:
- Fully functional PBR path tracer
- All Phase 2 features working in renders
- 80+ tests passing
- Updated demos showing GGX metals and glass

**Est. Time**: 1-2 weeks
**Est. LOC**: 750-950 new/refactored

---

### **Phase 3: Production Rendering** (2-3 weeks)

**Goal**: Advanced integrators, volumetrics, sampling

#### Key Features

1. **Integrator Abstraction**
   - Integrator trait (render_pixel)
   - PathIntegrator (current algorithm)
   - DirectIntegrator (direct lighting only)
   - NormalIntegrator, DepthIntegrator (debug)
   - **Est**: 150 LOC

2. **Bidirectional Path Tracing (BDPT)**
   - Light path generation
   - Camera path generation
   - Path connection
   - MIS weights
   - **Est**: 400-500 LOC
   - **Value**: Better caustics, SDS paths

3. **Volumetric Rendering**
   - Volume trait (homogeneous, heterogeneous)
   - Phase function (Henyey-Greenstein)
   - Delta tracking / ray marching
   - Medium interface
   - **Est**: 300-350 LOC
   - **Value**: Fog, smoke, participating media

4. **Advanced Sampling**
   - Low-discrepancy sequences (Sobol, Halton)
   - Stratified sampling
   - Importance sampling for environment maps
   - **Est**: 200 LOC

**Deliverables**:
- Multiple integrator options
- BDPT for complex lighting
- Volumetric fog/smoke
- Better sampling patterns

**Est. Time**: 2-3 weeks
**Est. LOC**: 1,050-1,200

---

### **Phase 4: Content Pipeline** (2-3 weeks)

**Goal**: Scene I/O, mesh support, export formats

#### Key Features

1. **Triangle Mesh Support**
   - Triangle primitive
   - TriangleMesh container
   - BVH for meshes
   - Smooth normal interpolation
   - UV interpolation
   - **Est**: 250 LOC

2. **Mesh Loading**
   - OBJ loader (via `tobj` crate)
   - PLY loader (via `ply-rs`)
   - Material library (MTL) parsing
   - Texture path resolution
   - **Est**: 200 LOC

3. **Scene File Format**
   - RON-based scene description
   - Serialization/deserialization with `serde`
   - Save/load complete scenes
   - Asset path management
   - **Est**: 300 LOC

4. **Advanced Image Output**
   - OpenEXR export (via `exr` crate)
   - HDR output
   - Multichannel output (albedo, normal, depth)
   - Tiled rendering for large images
   - **Est**: 200 LOC

5. **Render Session Management**
   - Resume from checkpoint
   - Progressive rendering
   - Batch render queue
   - **Est**: 150 LOC

**Deliverables**:
- Load complex 3D models
- Save/load scenes from files
- HDR/EXR output
- Production render pipeline

**Est. Time**: 2-3 weeks
**Est. LOC**: 1,100-1,300

---

### **Phase 5: Advanced Features** (2-3 weeks)

**Goal**: Cutting-edge features for realism

#### Key Features

1. **Subsurface Scattering**
   - BSSRDF trait
   - Dipole approximation
   - SubsurfaceMaterial
   - **Est**: 250 LOC
   - **Value**: Realistic skin, wax, marble

2. **Spectral Rendering** (Optional)
   - Spectrum trait abstraction
   - RgbSpectrum implementation
   - SampledSpectrum (hero wavelength)
   - Spectral textures
   - **Est**: 300 LOC
   - **Value**: Dispersion, wavelength-dependent effects

3. **Advanced Camera**
   - Thin lens model (depth of field)
   - Orthographic projection
   - Panoramic/360° camera
   - Motion blur support
   - **Est**: 200 LOC

4. **Denoising Integration** (Optional)
   - G-buffer generation (albedo, normal, depth)
   - Intel OIDN integration
   - Feature buffers
   - **Est**: 150 LOC
   - **Value**: Lower sample count for interactive work

**Deliverables**:
- Subsurface scattering
- Advanced camera effects
- Optional spectral rendering
- Optional AI denoising

**Est. Time**: 2-3 weeks
**Est. LOC**: 900-1,100 (600-700 core + optional features)

---

### **Phase 6: Production Polish** (1-2 weeks)

**Goal**: CLI, optimization, documentation

#### Key Features

1. **Command-Line Interface**
   - Argument parsing (via `clap`)
   - Render settings from CLI
   - Batch mode
   - Progress reporting
   - **Est**: 200 LOC

2. **Optimization**
   - SAH-based BVH construction
   - Thread pool tuning
   - SIMD for ray-box intersections (optional)
   - Memory profiling and reduction
   - **Est**: 300 LOC

3. **Comprehensive Documentation**
   - API documentation (rustdoc)
   - User guide
   - Scene file format reference
   - Material guide
   - Tutorial scenes
   - **Est**: Not LOC, but documentation pages

4. **Production Features**
   - Logging with `log` + `env_logger`
   - Error handling improvements
   - Validation and warnings
   - Example gallery
   - **Est**: 200 LOC

**Deliverables**:
- Professional CLI tool
- Optimized performance
- Complete documentation
- Production-ready release

**Est. Time**: 1-2 weeks
**Est. LOC**: 700-900 + documentation

---

## 📈 Estimated Completion Metrics

### Total Additional Work

| Phase | LOC | Tests | Time | Priority |
|-------|-----|-------|------|----------|
| **2.5 - Core Complete** | 750-950 | 15-20 | 1-2w | **Critical** |
| **3 - Advanced Rendering** | 1,050-1,200 | 20-25 | 2-3w | High |
| **4 - Pipeline** | 1,100-1,300 | 15-20 | 2-3w | High |
| **5 - Advanced Features** | 900-1,100 | 10-15 | 2-3w | Medium |
| **6 - Polish** | 700-900 | 5-10 | 1-2w | Medium |
| **Total** | **4,500-5,450** | **65-90** | **9-13w** | - |

### Current vs Target

- **Current**: ~4,500 LOC, 65 tests, 40% complete
- **Target**: ~9,000-10,000 LOC, 130-155 tests, 100% complete
- **Timeline**: 9-13 additional weeks for full completion

---

## 🎯 Recommended Path Forward

### **Option A: Rapid MVP → Usable Product** (3-4 weeks)

Focus on core functionality to get a working production renderer quickly.

**Sprint 1** (Phase 2.5 - Week 1-2):
1. ✅ Complete GGX and Dielectric BxDFs
2. ✅ Integrate scene graph into renderer
3. ✅ Direct lighting with MIS
4. ✅ Russian roulette termination

**Sprint 2** (Phase 4 Essentials - Week 3):
1. ✅ Triangle mesh primitive
2. ✅ OBJ loader
3. ✅ Basic scene file format
4. ✅ EXR export

**Sprint 3** (Phase 6 Polish - Week 4):
1. ✅ CLI interface
2. ✅ Basic optimization
3. ✅ Essential documentation

**Result**: Functional production path tracer with mesh loading, file I/O, PBR materials

---

### **Option B: Feature-Complete Product** (9-13 weeks)

Full implementation of all phases for a professional-grade renderer.

**Quarter 1** (Weeks 1-4):
- Phase 2.5 complete
- Phase 3 foundation (integrators, basic volumetrics)

**Quarter 2** (Weeks 5-8):
- Phase 3 complete
- Phase 4 complete

**Quarter 3** (Weeks 9-13):
- Phase 5 complete
- Phase 6 complete

**Result**: State-of-the-art path tracer with BDPT, volumetrics, SSS, full pipeline

---

### **Option C: Incremental Improvements** (Ongoing)

Prioritize based on immediate needs, implement features iteratively.

**Immediate** (Next session):
1. Complete GGX BxDF
2. Integrate renderer with scene graph
3. Create first real rendered output with new system

**Short-term** (2-3 sessions):
4. Complete Dielectric BxDF
5. Direct lighting + MIS
6. Triangle meshes + OBJ loading

**Medium-term** (4-6 sessions):
7. BDPT integrator
8. Scene file format
9. EXR export

**Long-term** (As needed):
10. SSS, volumetrics, advanced features
11. CLI + optimization
12. Documentation

**Result**: Flexible, prioritized development based on use cases

---

## 🔧 Critical Next Steps (Recommended Start)

### **Session 1: Make It Render**
**Goal**: First production-quality image with new Phase 2 features

1. Complete GGX BxDF implementation (~2 hours)
   - Implement D, G, F terms
   - Importance sampling
   - Add tests

2. Integrate renderer with scene graph (~1-2 hours)
   - Update Renderer to use new API
   - Test with Phase 2 demo scenes
   - Verify transforms work correctly

3. Render first showcase image (~30 min)
   - Use phase2_showcase example
   - Generate actual PPM/PNG output
   - Validate all features visually

**Deliverable**: Working renderer producing images with:
- PBR materials (diffuse, metal, glass)
- Area lights with soft shadows
- Hierarchical transforms
- Instanced geometry

---

## 📝 Dependencies to Add

### Immediate (Phase 2.5)
```toml
# Already have: rand, rayon, winit, pixels, image

# No new dependencies needed for Phase 2.5
```

### Phase 3
```toml
# Sampling
sobol_burley = "0.2"  # Low-discrepancy sampling
```

### Phase 4
```toml
# Serialization
serde = { version = "1.0", features = ["derive"] }
ron = "0.8"

# Mesh loading
tobj = "4.0"  # OBJ/MTL
ply-rs = "0.1"  # PLY

# Image output
exr = "1.72"  # OpenEXR
```

### Phase 5
```toml
[features]
spectral = []
denoise = ["oidn"]

[dependencies]
# Optional denoising
oidn = { version = "2.0", optional = true }
```

### Phase 6
```toml
clap = { version = "4.5", features = ["derive"] }
log = "0.4"
env_logger = "0.11"
```

---

## 🎬 Success Criteria

### Phase 2.5 Complete
- [ ] GGX materials render correctly (metal highlights)
- [ ] Glass materials with refraction work
- [ ] MIS produces clean images with area lights
- [ ] All demos render without errors
- [ ] 80+ tests passing

### Phase 3 Complete
- [ ] BDPT produces caustics correctly
- [ ] Volumetric fog renders realistically
- [ ] Multiple integrators available
- [ ] Cornell box test scenes

### Phase 4 Complete
- [ ] Can load complex OBJ models (100k+ triangles)
- [ ] Scene files save/load correctly
- [ ] EXR output with HDR values
- [ ] Example asset library

### Phase 5 Complete
- [ ] SSS on test models (dragon, bunny)
- [ ] Depth of field effects
- [ ] Optional spectral dispersion
- [ ] Denoising integration (if enabled)

### Phase 6 Complete
- [ ] CLI renders scenes from command line
- [ ] BVH construction < 1s for 1M primitives
- [ ] Complete rustdoc coverage
- [ ] Tutorial gallery with 10+ scenes

---

## 💡 Recommended Immediate Action

**Start with Option A (Rapid MVP)** focusing on Phase 2.5:

1. **This Session**: Complete GGX BxDF + integrate renderer
2. **Next Session**: Complete Dielectric + MIS
3. **Following Session**: Basic mesh support + OBJ loading
4. **Final Polish**: CLI + documentation

This gets you to a **usable, production-quality path tracer in 3-4 weeks** that can:
- Render PBR materials (diffuse, metal, glass)
- Load 3D models from OBJ files
- Use scene graphs with hierarchies
- Output to multiple formats
- Run from command line

Then evaluate whether to continue with advanced features (Phases 3-5) or use as-is.

---

**End of Roadmap**
