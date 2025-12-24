//! GPU-compatible data structures
//!
//! All structs in this module use #[repr(C)] to ensure consistent memory layout
//! between Rust (host) and CUDA (device) code.
//!
//! Key concepts:
//! - #[repr(C)]: Use C memory layout (required for GPU interop)
//! - Padding: Align structs to cache-friendly sizes (16/32/64 bytes)
//! - f32: GPUs are 2x faster with f32 than f64
//! - Simple types: No Option<>, no enums, no references (GPU can't handle them)

mod params;
mod types;

pub use params::LaunchParams;
pub use types::{GpuMaterial, GpuPointLight, GpuSphere};
