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
