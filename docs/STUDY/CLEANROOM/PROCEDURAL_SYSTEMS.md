# Tiny Glade: content pipeline and procedural systems (from names only)

Only file and folder names, counts and sizes were used. No file under `assets/` or
`compiled-assets/` was opened. The one exception is `default_settings.ron`, which is on the
permitted list. [C] = confirmed by a name or permitted plaintext file. [I] = inferred.

## Top-level layout and sizes [C]
- `assets/` 1.9 GB. Most of it is `audio/` (1.4 GB, 37 FMOD `.bank` files), then
  `meshes/` (251 MB, 482 `.json` and 1 `.glb`) and `fonts/` (178 MB, 85 `.ttf`).
- `compiled-assets/` 801 MB, 753 `.bin` files, all named by hash:
  - `texture-package/` 274 MB, 506 files.
  - `foliage-package/` 528 MB, split by theme: `autumn` (49), `flowery` (50), `olden` (50),
    `summer` (50), `winter` (48), each 93-117 MB.
- Meshes are stored as JSON, not glTF, except a single `brick.glb` [C]. The source format is
  probably a custom JSON export [I].

## Themes / biomes [C names]
Five glade themes appear consistently in `assets/glade/`, `compiled-assets/foliage-package/`
and the audio banks: **summer, autumn, winter, flowery, olden**. Each `glade/<theme>/`
holds `audio.json`, `grading.json`, `plants.json`, `settings.json`, `textures.json`,
`trees.json` and subfolders `plants/` (flowers, leafy_plants, terrain_plant1/2) and `trees/`
(bg_terrain_trees, branchy_trees_1/2, bushes, central_trees, clearing_edge1-3,
mid_distance_trees_1/2). Each theme therefore carries its own grading (colour LUT selection),
vegetation sets placed in concentric rings (central, clearing edge, mid-distance, background
terrain), and texture sets [I]. Seasonal audio banks also exist for Halloween and Christmas [C].

## Procedural building kit [C names, I purpose]
- Rule-driven parts with no authored building meshes. The shader and mesh names point to
  buildings assembled from small repeated parts: `wooden_plank` (+ `_lod1`, `_straight`),
  `roof_tile` (+ `_lod1`, `_backface`), `roof_support_beam`, `roof_spire`, `brick` / brick
  walls, `stair_step`, `stairs_pebble`, plaster, stone foundation, curved half-timber,
  gable planks, wooden arch frames.
- Matching crates: `system-roof`, `system-wall-constructor`, `timberframe`,
  `system-decorator`, `system-glade-border` [C].
- `meshes/decorators/` holds 228 files. Each decorator (windows cottage/gothic, dormer, half-
  dormer, arrow slits, balcony doors, doors, trap doors, chimneys, vent pipes, lanterns,
  flags, clothlines, flower pots and beds, window sills and lintels) comes in several variant
  files: `_collision`, `_interaction`, `_outline`, `_glass`, `_hat`, `_rails`, size suffixes
  such as `1x1`/`1x2`/`1x3`, and `rank1-3`. Each decorator therefore bundles a visual mesh, a
  physics collider, a picking volume, a selection outline, optional glass, and size or "rank"
  variants chosen procedurally by wall height or available space [I].
  `*_flowerbed_locations` meshes are likely marker meshes for spawn points.
- `nani_meshes.ron` (16 KB) is a registry of the parts drawn by the GPU-driven "nani"
  instancing system [I].

## Terrain, paths, water [C names]
- Terrain: `terrain.json`, `terrain_rocks.json`, `forest_terrain.json`, `terrain_rt/` (32
  `chunk_N.json`, probably a ray-tracing proxy split into chunks [I]), and `data/rocky_terrain.json` (27 MB,
  probably a precomputed rocky-cliff dataset [I]).
- Terrain is edited with brush strokes rasterised on the GPU: shaders `terrain_editing/*`, and
  meshes `brush_circle`, `brush_arrow*`, `brush_*_arrows` [C].
- Paths: shaders `paths/*` and `path_pebble(s)`. Paths are mask textures that exclude grass and
  plants and scatter pebbles [I].
- Water: `water_plane`, `water_stones`, `water_grass_winter`, `lilypad(_flower)`, `reed(_autumn)`,
  `floatie`, `fish`, `duck_*`, `sewage_gate_bar`, and GPU spawn shaders for lilypads and reeds [C].

## Vegetation [C names]
- Trees in tiers: `central_tree`, `mid_tree_v3-5` (+ `_trunk`), `mid_distance_tree_v1/2`
  (+ `_trunk`, `_particles`, `_particle_spawner`), `branchy_tree_v1/2`, `far_distance_tree`,
  `background_tree`, `tree_billboard`, `branch_billboard`, `olden_tree(_large)`,
  `naked_tree_1/2` (variants with and without mushrooms, plus a low-poly one) [C]. The
  `_particle_spawner` / `_particles` pairs suggest canopy leaves are particles or cards spawned
  on a source mesh rather than authored leaf geometry [I]. Distant tiers fall back to billboards [I].
- Ground cover: `clover(_flowers)`, `lavender`, `plant_thistle`, `plant_leafy`,
  `meadow_lowpoly_flowers`, `bush(_body, _flowers)`, `fallen_leaves`, `fallen_petals`;
  `prefabs/clearing/{tree,naked_tree}` (28 `.ron` prefabs) [C].
- Gardens are generated on the GPU (marching squares over a garden mask, then spawn bushes and
  flowers) [I, from shader names].

## Life and set dressing [C names]
- Critters: sheep (`sheep_animation/` with 31 frames as JSON, which suggests vertex animation
  [I]), `sheep_heart`, `duck_flying`, `duck_heart`, `small_bird`, `butterfly`, `fish`, fireflies.
- Clutter: `meshes/clutter/` holds 59 props (barrels, carts, crates, benches, pumpkins, tools,
  market stalls, window flower beds) and a `missing_mod_placeholder`. The UI list of clutter is
  in `ui/clutter.ron`, and icons are baked offline by `clutter-icon-pipe` [C/I].

## Lighting and look data [C names]
- `tod/default.json`: a single time-of-day definition [C]. Combined with `suncalc`, sun position
  is probably computed from a date/time/latitude [I].
- `luts/`: 28 grading LUTs (21 `.cube` film looks, 7 `.png`) [C].
- `colors.json`: the palette for the colour-picker feature (`system-color`) [I].

## Saves, starting builds, mods [C names]
- `starting-builds/full/` holds 30 numbered builds plus `intro`, and `demo/` holds 3 plus
  `intro`. Each build has `history.json`, `screenshot.jpg` and a `snapshots/` folder. Saves are
  stored as an edit history with periodic snapshots (undo/redo replay) [I].
- Mods are supported (`tg-mods`, zip) and packages are integrity-checked against
  `build-info/manifest.json` (2,078 entries) [C].
- `vulkan_allowlist_defaults.ron`: a default GPU/driver allow-list [I].

## default_settings.ron (permitted file; meaning only)
It contains two settings [C]: a video section with a render resolution scale of 0.4, and a
flag that enables the tutorial. By default the 3D scene is therefore rendered at 40% of output
resolution per axis and reconstructed by the temporal upscaler ("tgsr"/FSR) [I]. This is
strong evidence that the soft, painterly look tolerates and relies on aggressive temporal
upscaling.

## Offline pipeline summary [I]
Source assets (JSON meshes, images, glTF) go through `asset-pipe` and `turbosloth` (a cached,
content-hashed build graph) and come out as hashed `.bin` packages. Textures are BCn-compressed
via `intel_tex_2`/ISPC. Foliage is preprocessed per theme by `foliage-pipe` (uses `meshopt`,
`gltf` and `grass2-shader`). Clutter icons are rendered offline with dedicated shaders.
