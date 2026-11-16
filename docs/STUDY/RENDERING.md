# Rendering study

## What Tiny Glade does (public evidence)
- Custom Vulkan renderer; Bevy (modified) kept for ECS; compute-heavy, GPU-side procedural generation [SOURCE S3].
- Real-time GI via a "simplified-geometry ray-traced GI"; roofs added to it in 1.16 [SOURCE S4]. Talk title stresses
  heavy **ray marching** (GPU-driven rendering, shadows, GI, water, DoF) [SOURCE S5].
- Shadows: shadow maps with **Sample Distribution Shadow Maps + receiver-mask culling**; tiny ray-traced cascaded
  shadow maps for fog; fog is froxel participating media [SOURCE S4].
- Distant brick LOD is an **SDF** [SOURCE S4] → strong hint that SDFs are a core scene representation [INFERRED].
- Custom temporal upscaler (TGSR, small neural net) [SOURCE S4]; custom DoF ("novel") [SOURCE S5, no details].
- macOS port: ~half the graphics engine rewritten [SOURCE S4] → Metal constraints were real for them too.

## kajiya GI (lineage) [SOURCE S1]
- Geometry pass → ReSTIR GI for first bounce (half-res, 1 ray / 2×2 tile, temporal M≈10, two spatial passes 8 and 5
  samples) → irradiance cache for the rest of the path → denoise → TAA. 8.4 ms total at 1080p on RX 6800 XT.
- Irradiance cache: 12 camera-aligned clipmap cascades of sparse 32³ grids, ≤64k entries (typ. <16k); entries
  allocated on demand by screen queries ("votes"), rank-based voting to stop outside→inside leaks; each entry runs
  temporal reservoir resampling over a 4×4 octahedral map and is converted to **L1 SH**; 4 rays + 4 validation +
  16 short accessibility rays per entry per frame; cache reads itself (multi-bounce), no double buffering.
- Sky: small 64×64×6 cube regenerated each frame; 16×16×6 convolved cube for diffuse rays.
- Admitted weaknesses: cache temporal instability, light leaks with small geometry, blurry indirect shadows.

## Our compute-only adaptation (wgpu/Metal) [INFERRED → ADR-003]
| kajiya piece | Gouache now (G0/G1) | Next |
|---|---|---|
| HW ray tracing | sphere tracing an R16F SDF volume (0.5 m) + albedo volume | GPU voxelise + JFA, live edits; per-object SDF bricks (matches Tiny Glade's SDF hint) |
| irradiance cache clipmaps | fixed world grid of L1-SH probes (2 m), round-robin 16k/frame | camera clipmaps (G11), on-demand allocation by screen votes |
| ReSTIR screen GI | SSAO + bent normal + probe interpolation | half-res screen-space traces + temporal/spatial reservoir reuse for contact bounce |
| rank voting vs leaks | normal offset, facing weight, validity (inside-geometry) | DDGI depth moments; rank voting |
| sky cubes | sky SH (L2) from analytic atmosphere | sky-view LUT + cube for reflections |

## Other techniques reviewed (from prior knowledge; to verify per gate) [INFERRED]
PCSS (Fernando 2005) used now; GTAO (Jimenez 2016) planned to replace hemisphere SSAO; DDGI (Majercik 2019) visibility
for probes; SDSM (Lauritzen 2011) for shadow fitting; TAA variance clipping (Salvi) used; bokeh DoF (Jimenez 2014) and
tilt-shift at G1/G6; CDLOD/geometry clipmaps for Phase 2 terrain; straight skeleton (Felkel–Obdržálek) for roofs.
