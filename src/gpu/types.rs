/// Sphere primitive
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GpuSphere {
    /// Center position in world space (x, y, z)
    pub center: [f32; 3],
    /// Sphere radius
    pub radius: f32,
    /// Index into materials array
    pub material_idx: u32,
    /// Padding to align struct to 32 bytes
    _padding: [u32; 3],
}

impl GpuSphere {
    /// Create a new GPU sphere
    fn new(center: [f32; 3], radius: f32, material_idx: u32) -> Self {
        Self {
            center,
            radius,
            material_idx,
            _padding: [0; 3],
        }
    }
}

/// Material properties for Lambertian surfaces
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GpuMaterial {
    /// Base color (reflectance) in RGB
    /// Range: [0.0, 1.0] per channel
    pub albedo: [f32; 3],

    /// Emitted light in RGB
    /// most materials: [0.0, 0.0, 0.0]
    /// light sources: [intensity, intensity, intensity]
    pub emission: [f32; 3],

    /// Pad to 32 bytes
    _padding: [f32; 2],
}

impl GpuMaterial {
    /// Create a diffuse material
    pub fn diffuse(albedo: [f32; 3]) -> Self {
        Self {
            albedo,
            emission: [0.0, 0.0, 0.0],
            _padding: [0.0; 2],
        }
    }

    /// Create an emissive material
    pub fn emissive(emission: [f32; 3]) -> Self {
        Self {
            albedo: [0.0, 0.0, 0.0],
            emission,
            _padding: [0.0; 2],
        }
    }
}

/// Point Light
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GpuPointLight {
    /// position in world space
    pub position: [f32; 3],
    /// Light intensity (color x brighness)
    /// [10.0, 10.0, 10.0] = white light at intensity 10
    pub intensity: [f32; 3],
    /// Padding to 32 bytes
    _padding: [f32; 2],
}

impl GpuPointLight {
    /// Create a new point light
    pub fn new(position: [f32; 3], intensity: [f32; 3]) -> Self {
        Self {
            position,
            intensity,
            _padding: [0.0; 2],
        }
    }
}

// compile time size check
const _: () = {
    const fn assert_size<T>(expected: usize) {
        assert!(std::mem::size_of::<T>() == expected);
    }

    assert_size::<GpuSphere>(32);
    assert_size::<GpuMaterial>(32);
    assert_size::<GpuPointLight>(32);
};
