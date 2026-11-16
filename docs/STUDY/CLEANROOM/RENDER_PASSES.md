# Tiny Glade: render passes inferred from shader file names

Only file and folder names and sizes under `compiled-shaders/` were listed. No file there was
opened, including the small `shaders.list` and `crate-deps.list` text files. Each folder
`<crate>/compiled/` holds `.precomp` blobs, one per shader entry. Everything below that goes
beyond a literal name is [INFERRED]. The whole folder is 8.7 MB.

## Shader crates and counts (.precomp files)
| Folder | Files | Reading |
|---|---|---|
| country-core | 259 | Game content: materials, foliage, water, procedural GPU generation, UI |
| rhapsody-core | 11 | Engine utilities: reprojection map, BRDF LUT, mips, depth min/max |
| rhapsody-dof | 11 | Depth of field |
| rhapsody-gi | 8 | Screen-space radiance GI and GTAO |
| rhapsody-taa | 6 | TAA plus a custom temporal super-resolution ("tgsr") |
| rhapsody-shadow | 2 | Shadow setup, including volume shadows |
| rhapsody-post | 2 | Blur and reverse blur (bloom-style pyramid) |
| rhapsody-deferred | 1 | Half-resolution depth and normal extraction |
| clutter-icon-pipe | 3 | Offline icon baking for the clutter UI (bake, light, compose) |
| grass2-shader, tiny-glade | 0 | Only list files. Their shaders are folded into country-core |

## Likely frame structure [INFERRED]
1. **GPU-driven culling and draw building ("nani")**. Folder `nani/` contains `cull_and_bucket`
   variants per procedural part (wooden plank, gable plank, roof tile, plaster, stone foundation
   surface, curved half-timber, stairs, stair rope, wooden arch frame, bush, flowers, smoke,
   grass lod0), plus histogram, prefix-sum, `generate_draw_lists`,
   `allocate_draw_commands` and `draw_indexed_indirect_to_dispatch`. These look like
   compute-generated, bucketed indirect draws. Matching `nani_*` material shaders draw them,
   and they are the largest blobs at 100-160 KB each (wooden plank, gable plank, roof floor,
   plaster, smoke, curved half-timber, instanced roof). A separate `wall/` folder (brick
   chunk prefix sums, draw lists for main and shadow views, brick LOD0 and LOD1) handles
   procedural brick walls the same way. Generic `prefix_scan/` utilities support both.
2. **Depth pyramid**: `generate_hzb` (hierarchical Z for occlusion culling), `depth_min_max`,
   and `grass2/debug_hiz_tiles`.
3. **G-buffer / geometry**: per-material raster shaders for terrain (`distant_terrain`,
   `rocky_terrain`, `terrain_editing/floor`), buildings, roofs, ivy, flags/cloth,
   critters (sheep, duck, wren, butterfly, firefly), billboards (forest, plants, distant),
   clearing vegetation (tree canopy, trunk, meadow plant/flower), water props (lilypad, reed).
   `nani_velocity_only` and `grass2/velocity_reconstruct` imply motion vectors for TAA.
4. **Grass ("grass2")**: `card_splats`, `card_splat_cull`, `card_splat_resolve`, scan
   gather/finalize, shadow variants, `build_draws`, `lod0` and `bake_world_albedo_tint`. This
   looks like grass drawn as cards or splats with GPU culling and a scan-compaction step, a
   separate shadow path, and grass colour tinted from a baked world-albedo texture.
   `grass_object_push` implies grass bending around objects, and `combine_grass_removal`
   implies masks that remove grass under paths and buildings.
5. **Shadows**: `rhapsody-shadow/setup_shadows` and `setup_volume_shadows`,
   `shadow_receiver_frustum`, `shadow_mask_mean`, `apply_screen_shadows` (a large screen-space
   shadow application pass), `mei_shadow`, and `compute_volumetric_sm` (shadow map for
   volumetrics).
6. **GI and AO** (`rhapsody-gi`): `trace_gi_candidates` (also a larger game-specific copy in
   `country-core/gi/`), `update_gi_radiance`, `eval_screen_radiance`,
   `reproject_screen_radiance`, `blur_screen_radiance`, plus `gtao`, `gtao_temporal` and
   `gtao_upsample` (XeGTAO-derived). This reads as ray-traced GI candidates (probably compute
   rays against a BVH or SDF, since `GPU-Raytracer`, `obvhs` and `mesh_to_sdf` are credited),
   resolved into screen-space radiance with temporal reprojection and blur. `rdi/temporal` may
   be a related temporal filter ("radiance denoise"?). `primary_rt_debug` and `terrain_rt`
   meshes (32 chunks) suggest a ray-tracing proxy of the terrain. `compute_sky_visibility_volume`
   suggests a volumetric sky-occlusion term.
7. **Deferred lighting**: `deferred/{solid,grass,plant,canopy}_light_sun_sky`. Four lighting
   models (opaque, grass, foliage, tree canopy), each lit by sun and sky.
   `terrain_lighting_texture` points to pre-lit or baked terrain lighting. A clustered light
   list exists: `light_binning/bin_lights_to_cells_*` and cell prefix sums (lanterns,
   fireflies). `compute_brdf_lut` is a standard split-sum BRDF LUT.
8. **Water and ice**: `water_plane` (one of the largest, ~115 KB), `water_plane_refraction_copy`,
   `water/deferred_rt` and `calc_deferred_rt_args` (ray-traced or marched water
   reflections/refractions dispatched indirectly), `water_x_paths`, `draw/read_water_mask`,
   ice (`ice_plane`, `ice_deferred_rt`, `ice_refl_march`, reflection and refraction resolves,
   `ice_cracks`), `water_snow_plane`, `lut/water_noise`, and GPU spawning of lilypads and reeds.
9. **Atmosphere and volumetrics**: `skydome`, `cloud`, `star`, `bake_fog_noise`,
   `compute_froxel_lighting` (froxel volumetric fog), `apply_volumetrics` (+ half-res).
10. **Particles and effects**: leaf, snow, splash, sparkles, firefly glow with a visibility
    query, chimney smoke as instanced geometry.
11. **Painterly stylisation**: two families, `painterly/` and `vermeer/`, each with
    acquire target, convolve target, generate strokes and raster strokes (vermeer adds
    combine). These are GPU brush-stroke generation and rasterisation, used for a painterly
    mode or thumbnails (`session_selection/thumbnail_painterly`).
12. **Temporal AA / upscale**: `rhapsody-taa/taa`, `tgsr`, `tgsr2`, `tgsr2_decide`,
    `tgsr_reproject` and `tgsr_blur_lum`. "tgsr" plausibly means "Tiny Glade Super
    Resolution", a custom temporal upscaler. The FSR helpers `fsr_copy_history` and
    `fsr_init_reactive` show FSR as an alternative. Screenshot shaders `tgsr_capture_hires`,
    `_native` and `_ref` suggest high-resolution captures via the upscaler. `rhapsody-core/
    calculate_reprojection_map` feeds these passes.
13. **Depth of field** (`rhapsody-dof`): analyze, prepare, init buffers, half-res and full-res
    gathers (LOD0/LOD1 variants), fused, upsample. `country-core/dof/calc_autofocus` sets the
    focus distance automatically (tilt-shift / miniature look).
14. **Post**: `blur_downsample`, `rhapsody-post/blur` and `rev_blur` (downsample/upsample
    bloom chain), `gauss_blur*`, `fxaa`, and `final_post` (~53 KB: tonemap, grading LUT and
    vignette are likely). 28 LUTs ship in `assets/luts` (21 `.cube`, 7 `.png`).
    Finally `resolve_to_swapchain` and `resolve_msaa_to_swapchain`.
15. **UI and overlays**: `ui_2d*`, `text`, `egui`, `custom_cursor`, `dashed_line`,
    `thick_line`, `timed_terrain_gizmo`, logo animations, loading screen.

## Procedural GPU generation (not frame passes)
- `garden/marching_squares` (~87 KB), `garden/spawn_*` for bushes and flowers around
  buildings and gardens, and `read_bush_positions_for_cpu`.
- `paths/` (path mask with margin, directional contract, exclusions), `path_pebble(s)`.
- `rocky_terrain/displace_rocky_terrain`, `rocky_terrain/stairs/*` (indirect stairs).
- `terrain_editing/rasterize_terrain_stroke`, `cache_stroke`, `calc_highlight_hill_mask`, and
  GPU readbacks (`read_terrain_height`, `read_terrain_rock_mask`, `mouse_terrain_raycast`,
  `read_mouse_depth_value`).
- `rasterize_buildings(_animate)` writes a building footprint mask used by grass and
  foliage exclusion.
- `roof_tiles_gravity` suggests falling or settling roof-tile animation.

---
# Pass 2: static metadata (SPIR-V reflection, binary module paths)
Only SPIR-V module headers were parsed (stage, workgroup size, resource debug names, bindings,
image formats); no instruction was read. Raw output stays in `~/gouache_re_workspace/`.
Tags: [CS] = CONFIRMED-STATIC, [I] = INFERRED.

## Global facts
- [CS] Only vertex, fragment and compute stages. No mesh, task, tessellation or ray-tracing
  stages. All "ray tracing" is software compute traversal of a BVH stored in buffers.
- [CS] Most compute is 8×8 workgroups. Scans use 256–512 wide groups; culling uses 32–64.
- [CS] The G-buffer is one packed 64-bit unsigned-integer target (two 32-bit channels) plus
  depth. Normals are re-extracted to a half-resolution signed-8-bit view-normal texture, and
  depth to a half-resolution 32-bit float texture, by a dedicated pass.
- [CS] HDR scene colour, TAA/upscaler history, DoF and bloom targets are R11G11B10 float.
  GI/radiance intermediates are RGBA16F. AO is R16F, and R8 after temporal filtering.
- [CS] Shared resources visible in most game shaders: a frame-constants block, a per-view
  block, a bindless-style texture handle table, blue-noise textures, a terrain heightmap and
  a "visual" terrain heightmap, a colour library (palette data) and cascaded sun shadows.

## Frame graph, refined order [I unless noted]
1. Upload world rasters / dirty edits; GPU procedural spawners (garden, paths, lilypads, reeds).
2. Reprojection map from depth + previous depth (+ velocity) [CS inputs]. Depth min/max
   reduction feeds shadow setup.
3. GPU culling of instanced parts against the previous frame's visibility bitmask and the
   HZB, histogram-bucketed into merged indirect draw lists (two-phase occlusion culling;
   [CS] prev/next visibility bitmasks and HZB are bound). Walls use their own chunked brick
   lists with a separate shadow-view list culled against a shadow-receiver HZB.
4. Shadow setup on the GPU: cascade splits computed from the main view's depth min/max, a
   handful of cascade views (setup runs as a single tiny workgroup of ~4 threads, so probably
   ~4 cascades). Shadow casters are culled against the receiver set ("shadow receiver
   frustum"). A separate low-resolution 16-bit volumetric shadow map is also produced, partly
   by tracing the BVH.
5. Geometry raster into the packed G-buffer; grass far field is drawn by a compute "splat"
   rasteriser into a 32-bit-uint depth target (atomic min), then resolved by a full-screen pass.
6. Half-res depth/normal extraction; HZB build for next frame.
7. Screen shadows (16×16 groups): cascaded map + a shadow stencil map + terrain heightmap,
   temporally accumulated with a history texture (shadow TAA), output R8 mask; a histogram of
   the mask feeds a mean value used later in final post.
8. GTAO at half res → temporal → bilateral upsample to full res.
9. GI (half res, ReSTIR-style): trace candidates → reservoir update → screen evaluation →
   reprojection with temporal statistics → blur. Details below.
10. Lantern lights: binned into a grid of cells (prefix-summed lists), with a ReSTIR-DI
    reservoir for direct lighting from many small lights.
11. Deferred lighting as full-screen fragment passes, one per lighting model (solid, grass,
    plant, canopy). The solid pass reads G-buffer, depth, screen-space GI, screen shadows,
    indirect radiance, a split-sum BRDF LUT, the light grid and the DI reservoir.
12. Sky dome (colour-library driven, moon texture) and cloud sprites (four cloud textures),
    stars.
13. Froxel volumetrics: fog noise baked into a small 3D texture; froxel lighting uses
    cascades, the volumetric shadow map, a ray-traced sky-visibility volume, the terrain
    lighting texture and lanterns, with temporal history; applied full or half res.
14. Water/ice: reflection rays compacted into a list and traced on the BVH with indirect
    dispatch; ice uses a screen march then BVH, with temporal resolve.
15. Temporal upscaler (tgsr / tgsr2) → DoF → bloom pyramid → final post → swapchain.

## GI in detail [CS resources, I behaviour]
- Everything runs at half resolution (half depth, half normals, half-res outputs).
- Candidate trace: one candidate per half-res pixel per frame (one hit texture and one
  radiance texture), guided by blue noise. It reuses the previous TAA output as a
  screen-space radiance source, then falls back to a BVH over triangles (bottom-level BVH +
  triangle buffer + per-triangle metadata) and the terrain heightmap. Hits are shaded using
  the cascaded sun shadow, the terrain lighting texture and palette ids for wall/roof colours,
  so the BVH carries material ids, not textures.
- Reservoir update: ReSTIR-style resampling storing a packed reservoir, ray direction and
  radiance per pixel.
- Evaluation resolves reservoirs (spatial reuse with blue-noise offsets) into screen radiance.
- Temporal: reprojection with a temporal-statistics (moments) texture, then an
  edge-aware blur guided by half depth and normals. This matches the talk's "ReBLUR-like"
  denoiser in structure.
- Total effective budget: about one ray per quarter-resolution-area pixel per frame, before
  reuse. Exact sample counts not determined.

## Grass [CS resources, I behaviour]
Grass instances (clumps) are culled on the GPU against the HZB and a removal mask (paths,
water, buildings, gardens, rock, foliage exclusion combined into one RG8 mask). Near grass is
real instanced geometry (LOD0) through the same culling system as building parts; far grass
is card splats rasterised in compute and resolved. An object-push texture bends grass.
Tint comes from a baked world-albedo texture.

## Post [CS resources]
TAA/upscaler splits history into luminance (R16F) and chroma (RG16F) after reprojection, uses
a blurred-luminance and a velocity-spread texture, and a per-pixel decision texture (tgsr2).
DoF is tiled: CoC prepare, tile dilation, indirect dispatch of half-res and full-res gathers
with LOD variants. Final post combines the scene, the blur pyramid (bloom), the shadow-mask
mean and a 3D LUT (grading/ICC).
