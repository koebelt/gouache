# "Rendering Tiny Glades With Entirely Too Much Ray Marching" — Tomasz Stachowiak, GPC 2024 (S5)

Source: talk transcript supplied by the user (local only, not committed). Everything here is **[SOURCE S5]** unless
tagged [INFERRED]. Timestamps refer to the YouTube video (jusWW2pPnA0).

## Engine
- Custom engine: Rust + Vulkan + HLSL, mostly from scratch on open-source crates; **Bevy ECS** runs gameplay logic (55:14).
- Generators are deterministic; they emit **hundreds of thousands of individual instances** (bricks, planks, tiles,
  pebbles, plants, shutters), not synthesised meshes; no skinning/deformation (04:50). Up to ~1M bricks, going to 2M.
- Min-spec ~10-year-old GPUs, no mesh shaders, no HW ray tracing (01:39, 05:38, 23:22).

## GPU-driven rendering ("N9")
- GPU culling, sorting, LOD selection and draw generation. **Two-pass occlusion culling** (draw last frame's visible
  set via stable mesh IDs → depth pyramid → draw newly visible) (06:05).
- All meshes of one material share one vertex buffer; a mesh = sub-slice of the index buffer; LOD switch = index +1.
  One `draw_indexed_indirect` per material, draw count = mesh count, instance count = visible instances (07:10).
- **Far brick LOD = 6-triangle half-cube** flipped to face the camera; the pixel shader analytically intersects a
  rounded box to synthesise bevels so far bricks still catch rim light (07:40).
- Trees: billboards expanded from geometry with **depth writes** to fake individual leaves; overdraw-heavy → approximate
  front-to-back sort (24 depth buckets, single-pass counting sort), per-group mesh histograms → `draw_indexed_indirect_count`
  (08:07-10:36). Pre-Turing NVIDIA pays a heavy tax for indirect-count.
- Deferred with a **very thin G-buffer** and a handful of handcrafted materials; material passes pre-sorted (grass before
  buildings, etc.) (10:36).

## Shadows
- Sun shadows are soft: VSM/MSM rejected (light leaking with layered UGC scenes) → **PCF + PCSS**, normal-offset bias (11:33).
- **Shadow TAA:** time-of-day sweeps cause shadow-map aliasing. Track first and second moments of the PCF kernel
  samples (already fetched) → variance → bounding box in *shadow-map space* to clip shadow history (Salvi-style); shrink
  the box for soft shadows (12:34-14:30).
- **Contact shadows:** screen-space ray march toward the light, ~3 taps (15:02). Bricks had to be shrunk once contact
  shadows exaggerated them.
- Ray-march acne trick: sample depth with **both point and linear samplers**; intersection test uses the *further* of the
  two, thickness check uses the *closer* — removes stair-step acne without bias (17:40-19:05; shared as a gist).
- Local lights use the same ray marcher; currently capped at 32 casting lights (19:32, 57:23).

## Sky
- **No physical atmosphere** — artistic control: hand-made LUTs and time-of-day timelines per level (19:59).
- Clouds = sprites on a cylinder, pre-lit (Houdini, from several light angles) and blended by sun angle (20:28).
- The sky is the main GI light source.

## Global illumination (21:00-32:35)
- History: top-down lightmap with cone rays (too flat) → **DDGI rejected** (insufficient probe density, small shapes
  missing probes, leaking, ghosting when buildings are resized, time of day forces constant updates anyway) →
  screen-space probes à la Lumen/GI-1.0 (finicky, flat, probes stuck in creases) → ReSTIR GI (overkill: outdoor scenes
  are low variance; temporal part alone suffices) → **final: brute-force hemisphere rays + spatiotemporal filter**.
- Tracing: **1 ray per 16 pixels (quarter-res)**. Screen-space ray march first; on failure (behind geometry/off-screen),
  **software BVH ray tracing against coarse collision proxy geometry** (originally without roofs); if the BVH hit is
  on-screen, use the screen's radiance (hides crude proxies). BVH from a CUDA path tracer's wide BVH; built with Embree,
  now the Rust `obvhs` crate.
- Radiance → **L1 SH (4 coeffs/channel)**; spatial filter gathering 8 quarter-res samples to reconstruct; SH evaluated with
  a cross-bilateral filter using the full-res normal.
- Denoiser after **ReBLUR** ("Fast Denoising with Self-Stabilizing Recurrent Blurs", Zhdan): 3 passes — generate,
  temporal reprojection (+ gap fill, races accepted), small recursive spatial blur; kernel size driven by accumulated
  sample count. **GTAO (XeGTAO) as a cross-bilateral guide** in the denoiser, plus a tiny extra SSAO (~×0.2).
- Infinite bounces from temporal feedback; reference mode = screen-space path tracer with ~3 bounces (58:34).
- Goal stated: "the same general impression in the lighting, not matching bit by bit".

## Water and ice (32:35-40:39)
- Forward pass after deferred lighting; water is flat; waves are normal-only from precomputed textures + creature ripples.
- Reflection: SSR (8 linear steps + 3 bisection + 1 secant) → compact misses → half-res BVH trace; quad-level hit
  "hallucination" fills salt-and-pepper misses; BVH fallback returns **black** (blocking matters more than accuracy).
- Refraction faked: distort the ray by waves, short ray march (3 + 2 bisection) against a copy of underwater depth.
- Caustics faked by thresholding wave textures, projected using the sun angle.
- Ice: rough reflection with spherical-cap GGX sampling (~70% bias), denoised with BRDF-lobe-projected kernels; rough
  refraction = flipped reflection lobe; cracks are meshes with normals biased toward the light.

## Depth of field (40:39-50:33)
- Started with a single-pass DoF (Gustafsson); moved to a proper optical DoF for **tilt-shift** ("cuteness-driven
  development": tilt-shift makes the diorama read tinier).
- Final: screen-space "path-traced" DoF via ray marching — 1D radial slices of the kernel, CoC/depth prefetched into
  shared memory, per-slope bitmask intersections; fallback to a pre-blurred background when the hit is far from the
  focal plane; tile classification for half-res/simple regions. ~1.3-1.5 ms at 1440p on an RTX 2080.
- Frame treated as a painting (Ghibli inspiration); low dynamic range.

## Image formation (50:33)
- Avoid per-channel tonemapping "notorious six" hue skews. They use **Tony McMapface** (Stachowiak's display transform, a
  3D LUT applied after Reinhard; sRGB only); AgX recommended as a similar, more neutral alternative.

## Implications for Gouache [INFERRED → ADRs]
1. **GI (ADR-010, supersedes the DDGI baseline of ADR-003 as the primary path):** quarter-res hemisphere rays; screen
   ray march first, then fall back to our SDF volume (our "proxy geometry"); reuse on-screen radiance; L1 SH; ReBLUR-style
   recurrent denoiser guided by GTAO. Keep the probe grid only as a cheap radiance source for off-screen hits.
2. **Shadows:** add shadow-space variance clipping (shadow TAA), screen-space contact shadows, point+linear depth trick.
3. **Sky:** artist-directed LUT/gradient sky per season and time of day, not physical atmosphere — this directly
   addresses our "grey sky void" and warm haze problems. Clouds as procedurally painted cylinder sprites (our art rule:
   generated, not Houdini-authored).
4. **Geometry:** instanced bricks/stones with shared per-material vertex buffers and GPU culling (our G7 wall should
   emit stone *instances*, not merged meshes); far LOD as analytic rounded-box ray intersection in the pixel shader.
   Metal note: `draw_indexed_indirect_count` is unavailable → per-bucket `multi_draw_indexed_indirect` (BACKENDS.md).
5. **DoF:** tilt-shift is part of the look; gather-based DoF first, ray-marched DoF later.
6. **Tonemap:** stay with AgX (already chosen); evaluate Tony McMapface (check license before use) at G1 review.
