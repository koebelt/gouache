//! The Gouache frame graph (fixed order for now):
//! sky SH -> sun shadow map -> G-buffer -> GI probe update -> SSAO -> lighting -> TAA -> tonemap.

use glam::{Mat4, Vec3};
use gouache_math::mesh::{MeshData, Vertex};
use gouache_world::{CameraSpec, Look};
use wgpu::util::DeviceExt;

use crate::frame::{FrameUniforms, ViewMode, camera_view_proj, jitter, sun_view_proj, v4};
use crate::volume::{SceneVolume, f32_to_f16};
use crate::{GpuContext, RenderError, ShaderLibrary};

const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const HDR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const SHADOW_RES: u32 = 4096;

/// Renderer settings.
#[derive(Clone, Debug)]
pub struct RenderSettings {
    /// Output width.
    pub width: u32,
    /// Output height.
    pub height: u32,
    /// GI probe spacing (m).
    pub probe_spacing: f32,
    /// Probes updated per frame.
    pub probes_per_frame: u32,
    /// GI hysteresis (0 = no history, 0.97 = slow and stable).
    pub gi_hysteresis: f32,
    /// Debug view.
    pub view: ViewMode,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            probe_spacing: 2.0,
            probes_per_frame: 16384,
            gi_hysteresis: 0.9,
            view: ViewMode::Lit,
        }
    }
}

struct Targets {
    depth: wgpu::TextureView,
    albedo: wgpu::TextureView,
    normal: wgpu::TextureView,
    ao: wgpu::TextureView,
    hdr: wgpu::TextureView,
    hist: [wgpu::TextureView; 2],
    ldr: wgpu::Texture,
    ldr_view: wgpu::TextureView,
}

struct SceneGpu {
    vbuf: wgpu::Buffer,
    ibuf: wgpu::Buffer,
    index_count: u32,
    center: Vec3,
    radius: f32,
    probes: wgpu::Buffer,
    probe_dims: [u32; 3],
    probe_origin: Vec3,
    vol_origin: Vec3,
    vol_voxel: f32,
    vol_dims: [u32; 3],
    sdf_view: wgpu::TextureView,
    alb_view: wgpu::TextureView,
}

struct Pipelines {
    sky_sh: wgpu::ComputePipeline,
    shadow: wgpu::RenderPipeline,
    gbuffer: wgpu::RenderPipeline,
    gi_update: wgpu::ComputePipeline,
    ssao: wgpu::ComputePipeline,
    lighting: wgpu::ComputePipeline,
    accumulate: wgpu::ComputePipeline,
    tonemap: wgpu::ComputePipeline,
}

/// Owns all GPU resources for rendering one view.
pub struct Renderer {
    settings: RenderSettings,
    shaders: ShaderLibrary,
    pipes: Pipelines,
    targets: Targets,
    uniforms: wgpu::Buffer,
    sky_buf: wgpu::Buffer,
    shadow_view: wgpu::TextureView,
    s_linear: wgpu::Sampler,
    s_shadow: wgpu::Sampler,
    scene: Option<SceneGpu>,
    frame_index: u32,
    accum: u32,
    gi_cursor: u32,
    prev_vp: Mat4,
    last_cam: Option<CameraSpec>,
}

fn tex2d(
    device: &wgpu::Device,
    label: &str,
    w: u32,
    h: u32,
    f: wgpu::TextureFormat,
    u: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: f,
        usage: u,
        view_formats: &[],
    })
}

fn view(t: &wgpu::Texture) -> wgpu::TextureView {
    t.create_view(&wgpu::TextureViewDescriptor::default())
}

impl Renderer {
    /// Create pipelines and targets.
    pub fn new(ctx: &GpuContext, settings: RenderSettings) -> Result<Self, RenderError> {
        let d = &ctx.device;
        let mut shaders = ShaderLibrary::new(ShaderLibrary::default_root());
        let pipes = Self::build_pipelines(d, &mut shaders)?;
        let targets = Self::build_targets(d, settings.width, settings.height);
        let uniforms = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("frame"),
            size: std::mem::size_of::<FrameUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sky_buf = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sky_sh"),
            size: 16 * 10,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let shadow = tex2d(
            d,
            "shadow",
            SHADOW_RES,
            SHADOW_RES,
            DEPTH,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let s_linear = d.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("linear"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let s_shadow = d.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow_cmp"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        Ok(Self {
            settings,
            shaders,
            pipes,
            targets,
            uniforms,
            sky_buf,
            shadow_view: view(&shadow),
            s_linear,
            s_shadow,
            scene: None,
            frame_index: 0,
            accum: 0,
            gi_cursor: 0,
            prev_vp: Mat4::IDENTITY,
            last_cam: None,
        })
    }

    fn build_targets(d: &wgpu::Device, w: u32, h: u32) -> Targets {
        use wgpu::TextureUsages as U;
        let rt = U::RENDER_ATTACHMENT | U::TEXTURE_BINDING;
        let st = U::STORAGE_BINDING | U::TEXTURE_BINDING;
        let ldr = tex2d(
            d,
            "ldr",
            w,
            h,
            wgpu::TextureFormat::Rgba8Unorm,
            U::STORAGE_BINDING | U::COPY_SRC | U::TEXTURE_BINDING,
        );
        Targets {
            depth: view(&tex2d(d, "depth", w, h, DEPTH, rt)),
            albedo: view(&tex2d(d, "albedo", w, h, wgpu::TextureFormat::Rgba8UnormSrgb, rt)),
            normal: view(&tex2d(d, "normal", w, h, HDR, rt)),
            ao: view(&tex2d(d, "ao", w, h, HDR, st)),
            hdr: view(&tex2d(d, "hdr", w, h, HDR, st)),
            hist: [view(&tex2d(d, "hist0", w, h, HDR, st)), view(&tex2d(d, "hist1", w, h, HDR, st))],
            ldr_view: view(&ldr),
            ldr,
        }
    }

    fn build_pipelines(d: &wgpu::Device, lib: &mut ShaderLibrary) -> Result<Pipelines, RenderError> {
        lib.reset();
        let compute = |lib: &mut ShaderLibrary, name: &str| -> Result<wgpu::ComputePipeline, RenderError> {
            let m = lib.module(d, name)?;
            Ok(d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(name),
                layout: None,
                module: &m,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            }))
        };
        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Unorm8x4, 3 => Unorm8x4],
        };
        let shadow_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3],
        };
        let gm = lib.module(d, "gbuffer.wgsl")?;
        let gbuffer = d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("gbuffer"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &gm,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[Some(vertex_layout)],
            },
            primitive: wgpu::PrimitiveState { cull_mode: Some(wgpu::Face::Back), ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Greater),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &gm,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::TextureFormat::Rgba8UnormSrgb.into()), Some(HDR.into())],
            }),
            multiview_mask: None,
            cache: None,
        });
        let sm = lib.module(d, "shadow.wgsl")?;
        let shadow = d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &sm,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[Some(shadow_layout)],
            },
            // No culling: thin and open geometry (terrain edges) must still cast.
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: wgpu::DepthBiasState { constant: 2, slope_scale: 2.0, clamp: 0.0 },
            }),
            multisample: Default::default(),
            fragment: None,
            multiview_mask: None,
            cache: None,
        });
        Ok(Pipelines {
            sky_sh: compute(lib, "sky_sh.wgsl")?,
            shadow,
            gbuffer,
            gi_update: compute(lib, "gi_update.wgsl")?,
            ssao: compute(lib, "ssao.wgsl")?,
            lighting: compute(lib, "lighting.wgsl")?,
            accumulate: compute(lib, "accumulate.wgsl")?,
            tonemap: compute(lib, "tonemap.wgsl")?,
        })
    }

    /// Rebuild pipelines if any WGSL changed on disk. Errors keep the old pipelines.
    pub fn hot_reload(&mut self, ctx: &GpuContext) -> Result<bool, RenderError> {
        if !self.shaders.changed() {
            return Ok(false);
        }
        let p = Self::build_pipelines(&ctx.device, &mut self.shaders)?;
        self.pipes = p;
        self.accum = 0;
        Ok(true)
    }

    /// Output size.
    pub fn size(&self) -> (u32, u32) {
        (self.settings.width, self.settings.height)
    }

    /// Resize render targets.
    pub fn resize(&mut self, ctx: &GpuContext, w: u32, h: u32) {
        self.settings.width = w.max(1);
        self.settings.height = h.max(1);
        self.targets = Self::build_targets(&ctx.device, self.settings.width, self.settings.height);
        self.accum = 0;
    }

    /// Final LDR texture (rgba8unorm, display encoded) for presenting or reading back.
    pub fn output(&self) -> &wgpu::Texture {
        &self.targets.ldr
    }

    /// Final LDR view.
    pub fn output_view(&self) -> &wgpu::TextureView {
        &self.targets.ldr_view
    }

    /// Upload scene meshes and the GI volume.
    pub fn set_scene(&mut self, ctx: &GpuContext, meshes: &[MeshData], vol: &SceneVolume) {
        let d = &ctx.device;
        let mut all = MeshData::default();
        for m in meshes {
            all.append(m);
        }
        let (lo, hi) = all.aabb().unwrap_or((Vec3::ZERO, Vec3::ONE));
        let vbuf = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertices"),
            contents: bytemuck::cast_slice(&all.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let ibuf = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("indices"),
            contents: bytemuck::cast_slice(&all.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        // Probe grid spans the volume.
        let sp = self.settings.probe_spacing;
        let ext = Vec3::new(vol.dims[0] as f32, vol.dims[1] as f32, vol.dims[2] as f32) * vol.voxel;
        let pd = [
            ((ext.x / sp).ceil() as u32 + 1).max(2),
            ((ext.y / sp).ceil() as u32 + 1).max(2),
            ((ext.z / sp).ceil() as u32 + 1).max(2),
        ];
        let count = (pd[0] * pd[1] * pd[2]) as usize;
        let mut init = vec![[0f32; 16]; count];
        for p in &mut init {
            p[3] = -1.0; // uninitialised marker
        }
        let probes = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("probes"),
            contents: bytemuck::cast_slice(&init),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let vol_tex = |label: &str, format: wgpu::TextureFormat, bytes: &[u8], bpp: u32| {
            d.create_texture_with_data(
                &ctx.queue,
                &wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: vol.dims[0],
                        height: vol.dims[1],
                        depth_or_array_layers: vol.dims[2],
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D3,
                    format,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                },
                wgpu::util::TextureDataOrder::LayerMajor,
                {
                    debug_assert_eq!(bytes.len() as u32, vol.dims[0] * vol.dims[1] * vol.dims[2] * bpp);
                    bytes
                },
            )
        };
        let sdf16: Vec<u16> = vol.sdf.iter().map(|&x| f32_to_f16(x)).collect();
        let sdf = vol_tex("sdf", wgpu::TextureFormat::R16Float, bytemuck::cast_slice(&sdf16), 2);
        let alb =
            vol_tex("vol_albedo", wgpu::TextureFormat::Rgba8Unorm, bytemuck::cast_slice(&vol.albedo), 4);
        self.scene = Some(SceneGpu {
            vbuf,
            ibuf,
            index_count: all.indices.len() as u32,
            center: (lo + hi) * 0.5,
            radius: (hi - lo).length() * 0.5,
            probes,
            probe_dims: pd,
            probe_origin: vol.origin,
            vol_origin: vol.origin,
            vol_voxel: vol.voxel,
            vol_dims: vol.dims,
            sdf_view: view(&sdf),
            alb_view: view(&alb),
        });
        self.accum = 0;
        self.gi_cursor = 0;
    }

    /// Restart temporal accumulation (call when the scene or look changes).
    pub fn reset_history(&mut self) {
        self.accum = 0;
    }

    fn bind(
        &self,
        d: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        res: &[wgpu::BindingResource<'_>],
    ) -> wgpu::BindGroup {
        let entries: Vec<wgpu::BindGroupEntry<'_>> = res
            .iter()
            .enumerate()
            .map(|(i, r)| wgpu::BindGroupEntry { binding: i as u32, resource: r.clone() })
            .collect();
        d.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout, entries: &entries })
    }

    /// Render one frame. `still` enables unbounded accumulation (headless shots, idle editor).
    pub fn render(&mut self, ctx: &GpuContext, cam: &CameraSpec, look: &Look, still: bool, time_s: f32) {
        let Some(scene) = &self.scene else { return };
        let (w, h) = self.size();
        if self.last_cam.as_ref() != Some(cam) {
            if !still {
                // Moving camera: keep TAA history but clamp; still mode restarts the average.
            }
            self.last_cam = Some(cam.clone());
            if still {
                self.accum = 0;
            }
        }
        let vp = camera_view_proj(cam, w as f32 / h as f32, 0.05);
        let j = jitter(self.frame_index, w, h);
        let jmat = Mat4::from_translation(Vec3::new(j[0], j[1], 0.0));
        let inv = (jmat * vp).inverse();
        let (svp, sext, srange) = sun_view_proj(look, scene.center, scene.radius);
        let sun = Vec3::from(look.sun_dir());
        let probe_count = scene.probe_dims[0] * scene.probe_dims[1] * scene.probe_dims[2];
        let per_frame = self.settings.probes_per_frame.min(probe_count);
        // Fast convergence during the first sweeps, then steady hysteresis.
        let sweeps = self.frame_index.saturating_mul(per_frame) / probe_count.max(1);
        let hyst = if sweeps < 2 {
            0.0
        } else if sweeps < 8 {
            0.6
        } else {
            self.settings.gi_hysteresis
        };
        let u = FrameUniforms {
            view_proj: vp.to_cols_array_2d(),
            inv_view_proj: inv.to_cols_array_2d(),
            prev_view_proj: if self.frame_index == 0 { vp } else { self.prev_vp }.to_cols_array_2d(),
            sun_view_proj: svp.to_cols_array_2d(),
            camera_pos: v4(Vec3::from(cam.eye), 0.05),
            sun_dir: v4(sun, look.sun_radius_deg.to_radians()),
            sun_color: [look.sun_tint[0], look.sun_tint[1], look.sun_tint[2], look.sun_intensity],
            shadow: [sext, srange, look.shadow_softness, 0.0],
            resolution: [w as f32, h as f32, 1.0 / w as f32, 1.0 / h as f32],
            jitter: [j[0], j[1], self.frame_index as f32, hyst],
            params: [look.exposure, look.gi_strength, look.ao_strength, look.sky_intensity],
            params2: [0.0, look.contrast, look.saturation, time_s],
            gi_origin: v4(scene.probe_origin, self.settings.probe_spacing),
            gi_dims: [scene.probe_dims[0], scene.probe_dims[1], scene.probe_dims[2], self.gi_cursor],
            vol_origin: v4(scene.vol_origin, scene.vol_voxel),
            vol_dims: [scene.vol_dims[0], scene.vol_dims[1], scene.vol_dims[2], per_frame],
            debug: [self.settings.view as u32, self.accum, u32::from(still), 0],
        };
        ctx.queue.write_buffer(&self.uniforms, 0, bytemuck::bytes_of(&u));
        self.prev_vp = vp;

        let d = &ctx.device;
        let ub = self.uniforms.as_entire_binding();
        let t = &self.targets;
        let mut enc = d.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });

        // Sky SH + sun colour.
        {
            let bg = self.bind(
                d,
                &self.pipes.sky_sh.get_bind_group_layout(0),
                &[ub.clone(), self.sky_buf.as_entire_binding()],
            );
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("sky_sh"),
                timestamp_writes: None,
            });
            p.set_pipeline(&self.pipes.sky_sh);
            p.set_bind_group(0, &bg, &[]);
            p.dispatch_workgroups(1, 1, 1);
        }
        // Shadow map.
        {
            let bg = self.bind(d, &self.pipes.shadow.get_bind_group_layout(0), std::slice::from_ref(&ub));
            let mut p = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            p.set_pipeline(&self.pipes.shadow);
            p.set_bind_group(0, &bg, &[]);
            p.set_vertex_buffer(0, scene.vbuf.slice(..));
            p.set_index_buffer(scene.ibuf.slice(..), wgpu::IndexFormat::Uint32);
            p.draw_indexed(0..scene.index_count, 0, 0..1);
        }
        // G-buffer.
        {
            let bg = self.bind(d, &self.pipes.gbuffer.get_bind_group_layout(0), std::slice::from_ref(&ub));
            let clear = wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            };
            let mut p = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("gbuffer"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &t.albedo,
                        depth_slice: None,
                        resolve_target: None,
                        ops: clear,
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &t.normal,
                        depth_slice: None,
                        resolve_target: None,
                        ops: clear,
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &t.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            p.set_pipeline(&self.pipes.gbuffer);
            p.set_bind_group(0, &bg, &[]);
            p.set_vertex_buffer(0, scene.vbuf.slice(..));
            p.set_index_buffer(scene.ibuf.slice(..), wgpu::IndexFormat::Uint32);
            p.draw_indexed(0..scene.index_count, 0, 0..1);
        }
        let groups = (w.div_ceil(8), h.div_ceil(8));
        // GI probe update.
        if look.gi_strength > 0.0 {
            let bg = self.bind(
                d,
                &self.pipes.gi_update.get_bind_group_layout(0),
                &[
                    ub.clone(),
                    wgpu::BindingResource::TextureView(&scene.sdf_view),
                    wgpu::BindingResource::TextureView(&scene.alb_view),
                    wgpu::BindingResource::Sampler(&self.s_linear),
                    wgpu::BindingResource::TextureView(&self.shadow_view),
                    wgpu::BindingResource::Sampler(&self.s_shadow),
                    self.sky_buf.as_entire_binding(),
                    scene.probes.as_entire_binding(),
                ],
            );
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("gi_update"),
                timestamp_writes: None,
            });
            p.set_pipeline(&self.pipes.gi_update);
            p.set_bind_group(0, &bg, &[]);
            p.dispatch_workgroups(per_frame, 1, 1);
        }
        // SSAO.
        {
            let bg = self.bind(
                d,
                &self.pipes.ssao.get_bind_group_layout(0),
                &[
                    ub.clone(),
                    wgpu::BindingResource::TextureView(&t.depth),
                    wgpu::BindingResource::TextureView(&t.normal),
                    wgpu::BindingResource::TextureView(&t.ao),
                ],
            );
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ssao"),
                timestamp_writes: None,
            });
            p.set_pipeline(&self.pipes.ssao);
            p.set_bind_group(0, &bg, &[]);
            p.dispatch_workgroups(groups.0, groups.1, 1);
        }
        // Lighting.
        {
            let bg = self.bind(
                d,
                &self.pipes.lighting.get_bind_group_layout(0),
                &[
                    ub.clone(),
                    wgpu::BindingResource::TextureView(&t.depth),
                    wgpu::BindingResource::TextureView(&t.albedo),
                    wgpu::BindingResource::TextureView(&t.normal),
                    wgpu::BindingResource::TextureView(&self.shadow_view),
                    wgpu::BindingResource::Sampler(&self.s_shadow),
                    wgpu::BindingResource::TextureView(&t.ao),
                    self.sky_buf.as_entire_binding(),
                    scene.probes.as_entire_binding(),
                    wgpu::BindingResource::TextureView(&t.hdr),
                ],
            );
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("lighting"),
                timestamp_writes: None,
            });
            p.set_pipeline(&self.pipes.lighting);
            p.set_bind_group(0, &bg, &[]);
            p.dispatch_workgroups(groups.0, groups.1, 1);
        }
        // Temporal accumulation.
        let (src, dst) = ((self.frame_index as usize + 1) % 2, self.frame_index as usize % 2);
        {
            let bg = self.bind(
                d,
                &self.pipes.accumulate.get_bind_group_layout(0),
                &[
                    ub.clone(),
                    wgpu::BindingResource::TextureView(&t.hdr),
                    wgpu::BindingResource::TextureView(&t.hist[src]),
                    wgpu::BindingResource::TextureView(&t.depth),
                    wgpu::BindingResource::Sampler(&self.s_linear),
                    wgpu::BindingResource::TextureView(&t.hist[dst]),
                ],
            );
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("accumulate"),
                timestamp_writes: None,
            });
            p.set_pipeline(&self.pipes.accumulate);
            p.set_bind_group(0, &bg, &[]);
            p.dispatch_workgroups(groups.0, groups.1, 1);
        }
        // Tonemap.
        {
            let bg = self.bind(
                d,
                &self.pipes.tonemap.get_bind_group_layout(0),
                &[
                    ub,
                    wgpu::BindingResource::TextureView(&t.hist[dst]),
                    wgpu::BindingResource::TextureView(&t.ldr_view),
                ],
            );
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("tonemap"),
                timestamp_writes: None,
            });
            p.set_pipeline(&self.pipes.tonemap);
            p.set_bind_group(0, &bg, &[]);
            p.dispatch_workgroups(groups.0, groups.1, 1);
        }
        ctx.queue.submit([enc.finish()]);
        self.frame_index += 1;
        self.accum += 1;
        self.gi_cursor = (self.gi_cursor + per_frame) % probe_count.max(1);
    }

    /// Copy the LDR output to CPU memory as tightly packed RGBA8.
    pub fn read_output(&self, ctx: &GpuContext) -> Vec<u8> {
        let (w, h) = self.size();
        let row = (w * 4).div_ceil(256) * 256;
        let buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: u64::from(row * h),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = ctx.device.create_command_encoder(&Default::default());
        enc.copy_texture_to_buffer(
            self.targets.ldr.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buf,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row),
                    rows_per_image: Some(h),
                },
            },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        ctx.queue.submit([enc.finish()]);
        let slice = buf.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        ctx.wait_idle();
        let data = slice.get_mapped_range().expect("readback buffer mapped");
        let mut out = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            let s = (y * row) as usize;
            out.extend_from_slice(&data[s..s + (w * 4) as usize]);
        }
        out
    }
}
