//! Gouache renderer: custom wgpu pipeline (no Bevy PBR).
//!
//! Frame: sky SH -> sun shadow map -> G-buffer -> GI probe update -> SSAO -> lighting -> TAA -> tonemap.
//! Everything after the G-buffer is compute so Metal, Vulkan and DX12 behave identically.

pub mod context;
pub mod frame;
pub mod renderer;
pub mod shader;
pub mod volume;

pub use context::{BackendInfo, GpuContext};
pub use frame::ViewMode;
pub use renderer::{RenderSettings, Renderer};
pub use shader::ShaderLibrary;

/// Renderer errors.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// No suitable GPU adapter.
    #[error("no GPU adapter: {0}")]
    NoAdapter(String),
    /// Device creation failed.
    #[error("device request failed: {0}")]
    Device(String),
    /// WGSL load/validation error.
    #[error("shader error: {0}")]
    Shader(String),
}
