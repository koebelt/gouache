//! GPU device creation (headless or for a surface) and capability reporting.

use crate::RenderError;

/// Optional features we use when present. Everything else must be core WebGPU
/// so that Metal (our common denominator) works.
pub const OPTIONAL_FEATURES: wgpu::Features = wgpu::Features::TIMESTAMP_QUERY
    .union(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS)
    .union(wgpu::Features::FLOAT32_FILTERABLE);

/// Owned wgpu instance, adapter, device and queue.
pub struct GpuContext {
    /// Instance (needed to create surfaces).
    pub instance: wgpu::Instance,
    /// Adapter in use.
    pub adapter: wgpu::Adapter,
    /// Logical device.
    pub device: wgpu::Device,
    /// Submission queue.
    pub queue: wgpu::Queue,
}

/// Human-readable summary of the adapter, recorded in shot metadata.
#[derive(Clone, Debug, serde::Serialize)]
pub struct BackendInfo {
    /// Adapter name.
    pub name: String,
    /// Backend (Metal, Vulkan, Dx12...).
    pub backend: String,
    /// Device type.
    pub device_type: String,
    /// Optional features that were enabled.
    pub features: Vec<String>,
}

impl GpuContext {
    /// Create a device with no surface (screenshots, tests, CI).
    pub fn headless() -> Result<Self, RenderError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
        Self::from_instance(instance, None)
    }

    /// Create a device compatible with `surface` (editor window).
    pub fn from_instance(
        instance: wgpu::Instance,
        surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Self, RenderError> {
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: surface,
            apply_limit_buckets: false,
        }))
        .map_err(|e| RenderError::NoAdapter(e.to_string()))?;
        let features = adapter.features() & OPTIONAL_FEATURES;
        let al = adapter.limits();
        let limits = wgpu::Limits {
            max_storage_textures_per_shader_stage: al.max_storage_textures_per_shader_stage.min(8),
            max_texture_dimension_2d: al.max_texture_dimension_2d.min(16384),
            max_texture_dimension_3d: al.max_texture_dimension_3d.min(2048),
            max_storage_buffer_binding_size: al.max_storage_buffer_binding_size.min(1 << 30),
            max_buffer_size: al.max_buffer_size.min(1 << 30),
            max_compute_invocations_per_workgroup: 256,
            max_compute_workgroup_size_x: 256,
            max_compute_workgroup_size_y: 256,
            max_compute_workgroup_size_z: 64,
            ..wgpu::Limits::default()
        };
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("gouache"),
            required_features: features,
            required_limits: limits,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }))
        .map_err(|e| RenderError::Device(e.to_string()))?;
        Ok(Self { instance, adapter, device, queue })
    }

    /// Adapter summary.
    pub fn info(&self) -> BackendInfo {
        let i = self.adapter.get_info();
        BackendInfo {
            name: i.name,
            backend: format!("{:?}", i.backend),
            device_type: format!("{:?}", i.device_type),
            features: self.device.features().iter_names().map(|(n, _)| n.to_string()).collect(),
        }
    }

    /// Block until all submitted GPU work has finished.
    pub fn wait_idle(&self) {
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
    }
}
