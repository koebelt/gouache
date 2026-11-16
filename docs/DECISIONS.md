# Architecture Decision Records

Format: context → decision → consequences. Status: accepted / proposed / superseded.
Hard decisions from MASTER_SPEC §3 are not repeated here and are not reopened.

## ADR-001 Bevy ↔ renderer integration: own the surface (accepted for headless; editor spike pending)
**Context.** §3 requires Bevy for ECS/windowing/input and our own wgpu pipeline (no Bevy PBR). Bevy 0.20.1
pins its own wgpu; crates.io has wgpu 30.0.2.
**Decision.** `gouache_render` is a pure-wgpu library (wgpu 30.0.2) taking a `GpuContext`
(instance/adapter/device/queue) and rendering to its own textures. `gouache_cli` uses it with **no Bevy at all**
(fast headless, deterministic). The editor uses Bevy for ECS, scheduling, winit windowing and input only
(`bevy_render` disabled) and creates its own `wgpu::Surface` from the window's raw handles.
**Why.** Decouples our wgpu version from Bevy's release cadence; no extract/render-world overhead;
identical code path headless and windowed. Keeping `bevy_render` for plumbing would tie us to its wgpu
and its render graph, and gains little.
**Consequences.** bevy_egui (which needs bevy_render) is unavailable → dev panels use `egui` + `egui-wgpu`
directly (pinned to a wgpu-30-compatible release). Risk to verify in the spike: raw-handle access for the
Bevy window without `bevy_render`. Fallback: plain winit app with `bevy_ecs` + `bevy_app` only.

## ADR-002 Deferred G-buffer + compute lighting/post
**Decision.** Thin G-buffer (albedo sRGB8 + baked AO; normal RGBA16F + translucency; Depth32F reversed-Z)
then compute passes for SSAO, lighting, TAA and tonemap.
**Why.** GI, AO and translucency are all screen-space resolves over the same data; compute avoids
render-target format quirks across Metal/Vulkan/DX12 (storage textures restricted to core formats).

## ADR-003 GI baseline: probe grid traced against a CPU-built SDF volume (DDGI-lite)
**Context.** §3: real-time diffuse GI, compute only (no HW RT, Metal). kajiya (Tiny Glade's lineage) uses
HW-RT ReSTIR + an irradiance cache of 12 camera-aligned 32³ clipmap cascades with L1 SH output (see STUDY/RENDERING.md).
**Decision (Phase 0/G1).** World-space probe grid (2 m spacing over the diorama, L1 SH irradiance, validity flag),
updated round-robin each frame (16k probes × 64 rays). Rays sphere-trace an R16F signed distance volume
(0.5 m voxels) + RGBA8 albedo volume, built on the CPU (heightfield columns solid; meshes surface-rasterised;
interior by flood fill; exact separable EDT). Hits are lit by the sun via the shadow map plus last frame's
probes (infinite bounces), misses take sky SH. Lighting interpolates 8 probes with normal-offset, facing
and validity weights; SSAO + bent normal and baked AO modulate it.
**Evolution path.** (1) GPU voxelisation + JFA so edits update the volume live (needed by G2 terrain brushes);
(2) camera-centred clipmap cascades (G11); (3) per-probe depth moments (DDGI visibility) against leaks;
(4) screen-space near-field GI / ReSTIR resampling for contact bounce detail.
**Measured.** Volume build 110 ms for 241×48×241 voxels (release, M5 Pro). 48 accumulated frames at 1280×720: 180 ms.

## ADR-004 Shaders: WGSL from disk with `#import`, naga-validated, hot reload by mtime
**Decision.** `ShaderLibrary` inlines `#import "x.wgsl"` once, validates with naga before creating modules
(errors carry file context instead of device panics), and polls mtimes for hot reload (no `notify`: its
current release is an RC). Packaged builds will embed `assets/shaders` (decided at G10).

## ADR-005 Tonemap: AgX (polynomial fit) + small grade
**Decision.** AgX base handles bright warm skies and sunlit plaster without hue skews into
yellow/cyan that ACES fits produce. Contrast/saturation are look parameters tuned via the match loop.

## ADR-006 Sun shadows: single 4096² orthographic map + PCSS for the Glade
**Decision.** The 150 m diorama fits one map (~4 cm texels). PCSS: 12-tap blocker search, 16-tap Vogel PCF with
penumbra ∝ blocker distance × tan(sun radius) × artistic softness, rotated per frame and accumulated.
**Later.** SDSM-style fitting (Tiny Glade 1.16 does this per its patch notes) and cascades for the
2 m close-up camera and Phase 2.

## ADR-007 Phase 0.4 / 0.5 (observation and clean-room analysis)
**Context.** Tiny Glade is installed on this machine. 0.4 needs screen capture/computer-use driving the game;
0.5 needs a *separate* Analyst session whose output only reaches the Implementer as prose.
**Decision.** This (Implementer) session does neither. 0.5 would break the clean-room separation if done
here, so it is deferred until the user starts a dedicated Analyst session (recording the EULA in
`docs/STUDY/LEGAL.md` first). 0.4 is listed in `docs/STUDY/OPEN_QUESTIONS.md` with the exact capture list;
it needs a session with screen-capture tools. Until then, the three supplied references + public sources are used.

## ADR-008 CI: GitHub Actions, 3 OS, Linux via lavapipe
**Decision.** `.github/workflows/ci.yml`: fmt, clippy -D warnings, tests on ubuntu/windows/macos; Linux installs
Mesa lavapipe for headless GPU smoke tests; cargo-deny. **Status:** the repo has no remote, so 3-OS green
cannot be verified yet; this needs the user (create a GitHub repo/remote). Tracked in PROGRESS.

## ADR-009 Match-loop measurement rules
1. **Dither-free statistics:** metrics run on half-resolution images (triangle filter). Ref 01 is an 8-bit palette
   dither; at full resolution its edge density/local contrast/spectrum measured dither noise (0.43 edge density).
2. **Per-region gating metrics:** a region may list `checks`; G1 lighting regions gate on colour/value metrics
   (palette ΔE, L/chroma/hue histograms, shadow/light ratio, warm/cool separation). Texture metrics stay reported
   ("x (info)") until the gate that builds that texture (grass blades G3, stones G7…).
3. **`ours_rect`:** when block-level composition cannot put the *same subject* at identical pixels (e.g. a cast
   shadow on grass), the region names a rect in our frame isolating that same subject. Allowed only for the same
   subject type; it must never be a cherry-picked patch, and it is removed once the scene is rebuilt with tools (G9).

## ADR-010 GI direction after the GPC 2024 talk (proposed, implement after G1 review)
**Context.** docs/STUDY/TALK_GPC2024.md: Tiny Glade rejected DDGI (density, leaks, ghosting on resize) and ships
quarter-res brute-force hemisphere rays (screen march → software trace fallback), L1 SH, ReBLUR-style denoiser with
GTAO guidance. Our G1 iterations show the same DDGI weaknesses (flat, leaky contact bounce).
**Decision.** Move primary diffuse GI to screen-space-first quarter-res rays with SDF-volume fallback, L1 SH, recurrent
spatiotemporal denoiser; keep the probe grid as the radiance source for off-screen hits. Also adopt: shadow TAA
(variance clip in shadow space), contact shadows, artist-directed sky gradients instead of the physical atmosphere.
**Status.** Proposed; scheduled after the user's G1 review since it changes the lighting under review.
