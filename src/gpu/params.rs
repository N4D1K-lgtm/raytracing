/// Parameters passed to OptiX kernel on each launch
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LaunchParams {
    /// Device pointer to output bugger [width x height x RGBA]
    /// each pixel is [f32; 4] = 16 bytes
    pub output_buffer: u64, // raw device pointer

    // === Camera ===
    /// Camera position
    pub camera_eye: [f32; 3],
    // align next field
    _pad0: f32,

    /// Camera basis vector U (horizontal)
    pub camera_u: [f32; 3],
    // align next field
    _pad1: f32,

    /// Camera basis vector V (vertical)
    pub camera_v: [f32; 3],
    // align next field
    _pad2: f32,

    /// Camera basis vector W (look)
    pub camera_w: [f32; 3],
    // align next field
    _pad3: f32,

    // === Image dimensions ===
    pub width: u32,
    pub height: u32,

    // === Render settings ===
    pub samples_per_pixel: u32,
    pub max_depth: u32,

    // === Scene data ===
    /// OptiX GAS handle (acceleration structure)                                                                            
    pub traversable: u64,

    /// Device pointer to materials array                                                                                    
    pub materials: u64,
    pub num_materials: u32,
    pub _pad4: u32,

    /// Device pointer to lights array                                                                                       
    pub lights: u64,
    pub num_lights: u32,

    /// Frame number (for RNG seeding)                                                                                       
    pub frame_number: u32,

    /// Device pointer to spheres array (for hit program to access)                                                          
    pub spheres: u64,
    pub num_spheres: u32,
    pub _pad5: u32,
}
