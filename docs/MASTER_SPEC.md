# 1. Mission

Build **Gouache**: a cross-platform (Windows, macOS, Linux) **map editor engine in Rust** for the future MMORPG **Mandate**. It must:

1. **Look and feel as close to *Tiny Glade* as possible**: the soft, warm, lit-from-within diorama look *and* the gridless, reactive, procedural building tools that animate and conform to everything around them.
2. **Scale to a world hundreds of kilometers wide** (default 256 km × 256 km, configurable to ≥1024 km) while buildings stay **human scale** (bricks ≈ 20–30 cm, doors ≈ 2 m), with seamless camera zoom from ~100 km altitude to 1.7 m eye level.
3. Use **no hand-made art**. The user is not an artist and has no one to help. Every texture, material, building part, plant and icon is generated programmatically. Beauty comes from lighting, geometry detail, color harmony and procedural rules.
4. Establish the **rendering foundations** (shaders, GI, LOD, streaming, culling) for Mandate and keep the data model server-ready (§9).

**Quality bar: "hero shots indistinguishable at a glance from Tiny Glade screenshots in composition, lighting mood and material charm."** A previous attempt that built breadth first (large world, painterly filter, many milestones) produced nothing convincing. Do the opposite: **depth first on one tiny diorama, lighting first, verified against references, then scale up.**

# 2. What actually creates the Tiny Glade look (design from this, in this order of importance)

1. **Lighting.** Its devs describe the look as the intersection of "stylized and physical". It has custom real-time global illumination (lineage: Tomasz Stachowiak's open-source *kajiya* renderer: ReSTIR resampling + sparse irradiance cache on clipmaps + denoising), soft shadows, AO, a custom depth of field, software ray tracing/ray marching for reflections. Warm sun against cool violet shadows with visible **bounce light** is the signature.
2. **Irregular, beveled, procedural geometric detail:** individual stones/tiles with rounded edges, jittered size/rotation/color, courses that follow curves and conform to terrain; roofs with individual shingles; timber framing; arches; crenellations.
3. **Cohesive, restrained color** (autumn/spring/summer/winter palettes), soft gradients, subtle painterly feel from rounded shapes and texture, **not** from a heavy post filter.
4. **Fluffy, layered foliage canopies**, grass, flowers, ivy, leaf litter, wind.
5. **Reactive tools:** everything regenerates live, systems interact procedurally (arches appear where paths cross walls, stones arrange along paths, walls stick to the ground, buildings can be glued, windows erode nearby plaster using a signed-distance-to-opening vertex attribute), spring-like animations, animated undo/redo.
6. **Life:** smoke, drifting leaves, birds/ducks, water, bunting/banner motion.

Verify these claims during Phase 0 research; tag each as sourced/observed/inferred.

# 3. Hard decisions (already made by the user: do not reopen)

| Topic | Decision |
|---|---|
| Language | Rust (stable, pinned `rust-toolchain.toml`) |
| Platforms | Windows (DX12/Vulkan), **macOS native Metal via wgpu** (not MoltenVK; macOS has no native Vulkan), Linux (Vulkan). All three build, run, pass CI. Treat Metal's limits as the common denominator. |
| Framework | **Bevy ECS + custom renderer**: use Bevy for ECS, scheduling, windowing, input, tasks. Write our own wgpu render pipeline. Do not use Bevy's PBR renderer. Time-boxed spike in Phase 0 decides the integration style (keep `bevy_render` only for device/surface/extract plumbing, or own the surface); ADR it. Pin latest stable versions (check crates.io; don't trust memory). |
| World | Flat, chunked, quadtree-tiled. Topology hidden behind a small trait; no sphere now. |
| Lighting | **Real-time diffuse GI from the start, software/compute only** (no hardware RT; Metal). Baseline = probe clipmaps + SSAO/bent normals + temporal accumulation; evolve toward irradiance-cache + resampling quality. |
| Art | 100% procedural. No imported art. |
| Verification | Headless screenshot harness + **Reference-Match Loop** (§11) + golden images. |
| Hardware floor | Apple M1 (8-core GPU) / GTX 1660-class: 60 fps @1080p in the editor with typical content, 30 fps worst-case stress. Quality presets degrade gracefully. |
| Names | Project **Gouache**, crates `gouache_*`. **Mandate** is the MMORPG; export format is the "Mandate world pack". |

# 4. Phases and gates (strict order)

```
PHASE 0  Foundations + Study                         -> G0
PHASE 1  The Glade (one tiny diorama)
   1A  Look + NATURE first                           -> G1 lighting (STOP), G2 terrain brushes, G3 grass+flowers,
                                                        G4 trees+bushes, G5 water+cascades, G6 nature hero shot (STOP)
   1B  Buildings                                     -> G7 wall tool, G8 buildings, G9 full hero shot (STOP), G10 perf+packaging
PHASE 2  Scale to the world                          -> G11..G14
```
**Nature is done to high quality BEFORE any building work.** Buildings start only after G6 passes and the user approves.
Large-world work (Phase 2) does not start until G10 passes. Keep coordinate/data abstractions clean in Phase 1 so scaling is additive, but render only the Glade until then.

## PHASE 0: Foundations + Study (G0)

**0.1 Workspace & tooling.** Cargo workspace (crates in §9), CI matrix (ubuntu/windows/macos; Linux uses software Vulkan/lavapipe for headless GPU tests), fmt/clippy `-D warnings`, `cargo deny`, tracing, profiling hooks, GPU timestamp queries, WGSL with `#import` preprocessor + hot reload, frame-time overlay.

**0.2 Headless screenshot harness.** `gouache_cli shot --scene X --camera bookmark --out png` (offscreen render, fixed resolution, deterministic seeds), camera bookmarks (committed JSON), contact-sheet generator, golden-image comparison with tolerant perceptual thresholds per backend.

**0.3 Public-source research** (`docs/STUDY/`): read in full and summarize, with links and dates, each tagged [SOURCE]/[OBSERVED]/[INFERRED]:
- Tomasz Stachowiak's Tiny Glade rendering talk (GPC, Breda, Nov 2024), his posts, and his open-source **kajiya** renderer (check license; learn technique, reuse code only per license with attribution). Produce a **wgpu/Metal compute-only adaptation plan** for the GI.
- Anastasia Opara's breakdowns (X/Mastodon/Bluesky threads, talks, 80.lv articles, Steam devlogs) of walls, windows, roofs, tower roofs, clutter, path/wall interaction, terrain, undo/redo, gluing buildings.
- Bevy render internals relevant to a custom renderer; wgpu per-backend limits (Metal: no bindless texture arrays, verify multi-draw-indirect support).
- Techniques: ReSTIR GI, radiance caches, DDGI, screen-space tracing, SDF ray marching, GT-VBAO, PCSS, TAA, bokeh DoF, straight skeleton, geometry clipmaps / CDLOD / virtual texturing (`bevy_terrain`, `terra`), Veloren worldgen.
Deliverables: `docs/STUDY/{RENDERING,PROCEDURAL,UX,OPEN_QUESTIONS,SOURCES,PLAN}.md`.

**0.4 Black-box observation** (only if Tiny Glade is installed and computer-use/screen tools are available; else skip and list what's needed). Play it as a user and capture reference packs to `docs/reference/captured/` (private use only, never shipped): same-scene sweeps (4 times of day × 4 seasons × 3 camera heights), close-ups of each material (stone, roof, plaster, timber, foliage, grass, flowers, water, path), lighting probes (sunlit colored wall beside grass; deep shadow corner; arch tunnel; room with one window), and **frame sequences of each tool's behavior** (drag wall, close loop, raise height, add window, glue buildings, path through wall, erase, undo/redo, terrain brush). Index with metadata.

**0.5 (OPT-IN, user has chosen this) Clean-room static analysis of the Tiny Glade binary.** Allowed only as follows:
- Use the user's own purchased copy, locally only. Never upload or distribute the binary or analysis databases.
- **Two sessions:** an *Analyst* (decompiles/inspects, writes **prose design descriptions only** to `docs/STUDY/CLEANROOM/`) and an *Implementer* (all other work; reads only those descriptions, never the binary or decompiler output).
- Decompiler/disassembler outputs, databases, symbol dumps and raw strings live **outside the repo** (git-ignored) and are never committed, pasted, or quoted.
- Study only: crate/engine versions, ECS organization, render pass order and resource formats/resolutions (approximate), compute passes for GI/denoise/DoF, generator stages and intermediate data structures, regeneration triggers, undo structure, tunable parameters as rough ranges. Rust release builds are optimized/stripped: expect partial results and fill gaps with in-game observation (0.4) rather than going deeper.
- **Forbidden:** copying/transcribing code, pseudo-code, constant tables, shaders, assets, text; debugging/hooking/injecting/memory-inspecting the running game; GPU frame capture to extract shaders/meshes/textures; bypassing DRM/anti-tamper; descriptions so precise they are the code in prose.
- Outputs: `ARCHITECTURE.md`, `RENDER_PASSES.md`, `PROCEDURAL_SYSTEMS.md`, `CONFIDENCE.md` (each claim tagged [CONFIRMED-STATIC]/[CONFIRMED-OBSERVED]/[INFERRED]) and a checklist confirming nothing from the binary is in the repo.

**0.6 Isolated experiments** in `experiments/` for the riskiest techniques, each ending in a screenshot vs reference: (a) compute-only diffuse GI with probe clipmaps + temporal accumulation on wgpu/Metal, (b) irregular beveled stone-course generator along a curve over uneven terrain, (c) straight-skeleton roofs with shingles, (d) soft shadows + AO grounding, (e) tilt-shift/bokeh DoF.

**G0 done when:** CI green on 3 OS; headless shot of a lit test scene viewed by you; study docs + `PLAN.md` written; experiments (a)(b)(d) produce believable images.

## PHASE 1: The Glade (one ~150 m x 150 m diorama; f32 is fine; orbit camera 2-150 m from focus; edges fade into a soft painted backdrop of distant hills/trees/sky)

Implement and gate **in this order**. Nature (G1-G6) reaches shippable quality before buildings (G7+).

### PHASE 1A: Look + Nature

**G1 Lighting sandbox. ★ USER STOP ★** HDR pipeline (rgba16f), physically plausible sun+sky, filmic tonemap, correct linear color; **soft contact-hardening shadows** (CSM + PCSS-like); **real-time diffuse GI** (probe clipmaps via compute ray marching against a scene SDF/heightfield + coarse voxel scene, temporal accumulation, denoise) + SSAO/bent normals; warm-sun/cool-violet-shadow split; sky ambient; foliage translucency; TAA. Scene: terrain patch + simple blocks. *Pass:* a Tiny Glade clay-render look with convincing bounce light, verified by Reference-Match on ref 01 shadow/grass/sky regions. **Stop, send side-by-sides, wait for approval.**

**G2 Terrain & terrain brushes (high quality).**
- Smooth, high-resolution heightfield (0.25 m samples near the camera, no faceting, correct smooth normals, no cracks), live mesh update <16 ms, soft cursor ring projected on the terrain, picking via analytic plane intersect + ray march.
- Brushes with Tiny-Glade-style soft falloff and pressure feel: raise, lower, smooth, flatten-to-height, plateau/terrace, ramp/slope between two points, roughen/noise, cliff/rock-ridge stamp, pit/pond carve. Undo/redo (animated), brush size/strength gizmo, symmetric stroke interpolation (no gaps/steps on fast drags).
- Procedural terrain material system: slope/height/curvature/noise-driven blend of grass, dirt, sand, rock, pebbles, snow, mud; paintable material brush; rock auto-appears on steep slopes; soft macro color variation; cohesive palette per season; micro-detail that never looks tiled.
- Terrain conform hooks (flatten under future buildings, grass flattening) designed now so G7+ can use them.
- *Pass:* sculpt a hill, valley, cliff and plateau in under 2 minutes and the result looks like a lit clay/diorama landscape, matching terrain regions of the references.

**G3 Grass & flowers (Zelda BotW-style, high quality).** Research first (BotW grass analyses, GDC/SIGGRAPH talks on large-scale GPU grass, e.g. Ghost of Tsushima, Horizon, Genshin-style; record in `docs/STUDY/NATURE.md`). Requirements:
- **GPU-driven instanced blades/clumps** (vertex-shader bending, compute culling, density by distance); blades are simple 3-5 triangle strips with a smooth bend curve; **millions of blades visible** at the quality preset, stable framerate.
- **Color:** blade base takes the terrain color, tip shifts to a lighter, warmer/more saturated hue (gradient base to tip), per-clump hue/brightness variation, subtle painted-in AO at the base so grass sits in the ground, sun-backlit translucency and rim glow, soft self-shadowing look.
- **Wind:** layered noise-driven rolling waves across the field (large slow gusts + small flutter), direction follows a global wind vector, flowers and bushes sway in sync.
- **Interaction:** camera focus/cursor/characters bend and trample grass through a world-space interaction texture with springy recovery; tools (terrain brush, placing props) disturb grass.
- **View-dependent fullness:** blades widen/thicken at grazing angles so fields look dense from low and high cameras; no see-through bald spots from above.
- **LOD (Section 7):** 0-20 m full blades, 20-50 m thinned and bolder, 50-100 m clumped cards, 100-400 m merged painted patches, beyond 400 m vegetation is carried by painted terrain color. Coverage and average color constant across transitions; dithered cross-fade.
- **Density painting** brush + rule-based scatter (slope, height, moisture, proximity to water/paths), grass variants (short, long, dry, clover, reeds, ferns).
- **Flowers:** procedural species (daisy, poppy, buttercup, lavender, bluebell, wild rose...), stylized soft petals with translucency, stem/leaf variation, clusters and drifts, big readable foreground flowers (ref 03), butterflies/bees as simple procedural motion, seasonal palettes.
- *Pass:* a meadow shot matches ref 03's foreground and ref 01's lavender/shrub regions, with visible wind waves and bounce light on grass. Stable at 60 fps on the hardware floor with the preset.

**G4 Trees & bushes (high quality).**
- Procedural species via space-colonization/L-system skeleton + **layered leaf-cluster canopies** (fluffy, clumped, soft-edged shells like ref 01): oak/broadleaf, birch, pine/fir, willow, small fruit/blossom tree, autumn variants, shrubs and hedges. Per-tree seed/age/size/lean/health controls; tree brush (scatter with spacing rules) and single-place tool with in-world size/shape gizmo and spring "growth" animation.
- Canopy shading: translucency, bounce light inside canopy, soft AO between leaf clusters, GI-aware ambient, subtle color variation (season palettes), leaf texture synthesized procedurally (no imported art), wind sway (trunk, branch, leaf flutter), falling leaves and leaf litter beneath.
- Trunks: bark synthesized procedurally, roots flaring into terrain, branch forks with smooth blends.
- LOD chain for trees as in Section 7; forests to the horizon at budget.
- *Pass:* canopy region of ref 01 passes the loop (fluffy layered masses, autumn palette with warm-light/cool-shadow separation, no "noise cards" look).

**G5 Water & cascades (high quality).**
- Ponds, lakes, rivers, sea; carve/fill with terrain brushes; shoreline foam and wet-sand/dark mud edge; depth-based tint and transparency; **reflections** (sky + SSR + cheap software tracing for nearby objects), soft refraction, gentle ripples, flow-map streaks on rivers, sun glints, floating leaves, lily pads, reeds/cattails along banks, ducks (simple procedural creatures with swim animation and wake).
- **Cascades/waterfalls:** drag a river over a drop or draw a cascade on a cliff/terrace; stepped drops, foam strokes, mist particles, plunge-pool ripples, flow-map streaks, terrain/stone carving at the base; rocks procedurally placed around.
- Water must be lit by the same GI/sky as the rest and match the dark reflective ponds with leaf litter in ref 02.
- *Pass:* a pond with reeds and ducks and a small waterfall passes the loop against the water regions of ref 02.

**G6 Nature hero shot. ★ USER STOP ★** Build, with the editor tools only, a nature-only glade: sculpted terrain, meadow with flowers and wind, autumn and spring trees, a pond with a cascade, leaf litter, ambient life (birds, butterflies, drifting leaves). Reference-Match against ref 03 (foreground/midground/distance, aerial perspective), ref 01 (foliage/shrubs) and ref 02 (water, leaf litter). **Stop, send images, wait for approval. Buildings do not start until approved.**

### PHASE 1B: Buildings (only after G6 approval)

**G7 Wall tool.** Draw a curve -> irregular **beveled stones** (size/rotation/color jitter, mortar recess, courses along the curve, per-vertex baked AO/curvature) conforming to terrain ("sticks to the ground", grass flattens around foundations), corners/ends/height handle, regenerate **<16 ms perceived** (worker-thread generation, atomic swap, spring-in animation), animated eraser, animated undo/redo.

**G8 Buildings.** Closed loops -> rooms/towers; doors/windows that snap and carve the wall (frames, shutters, glazing bars, flower boxes), plaster erosion near openings via SDF vertex attribute; roofs from arbitrary footprints (straight skeleton -> hip/gable/cone/pyramid/crow-step), individual irregular shingles with color variation and moss patches, ridge caps, eaves, dormers, chimneys + smoke; timber framing with jetties; arches, balconies, stairs, platforms, crenellations; glue buildings together; paths that flatten terrain and spawn arches/gates through walls; ivy; bunting/banners; in-world gizmos (ref 02) and bottom icon bar (procedurally drawn icons).

**G9 Full hero shot. ★ USER STOP ★** Rebuild ref 01 with the editor tools and ref 02's UX (gizmos, bottom toolbar). Pass the full Reference-Match Loop and checklist. **Stop, send images, wait for approval.**

**G10 Performance & packaging.** 60 fps @1080p on the hardware floor with the hero scenes; quality presets; Windows/macOS/Linux CI green; polish ambient life (smoke, birds, leaves, bunting).

## PHASE 2: Scale to the world (G11-G14)

Must retain the same lighting/look quality at every LOD.

**G11 Large-world core (§6):** coordinates, tile quadtree, deterministic procedural terrain base + sparse edit layers, streaming, LOD terrain with geomorphing, camera-relative rendering, reversed-Z, seamless zoom from 150 km to 1.7 m. GI probe clipmaps scale with distance.
**G12 LODs & settlement scale (§7):** nature and building LODs; LOD0–LOD4, merged chunk meshes, GPU culling/instancing, dithered cross-fades; stress scene of 100k buildings.
**G13 Macro tools for Mandate:** intent-map painting (ocean/plains/hills/mountains…), spline roads/rivers with terrain carve, biome/material masks, deterministic erosion *bake*, **settlement generator** (road network → plots → buildings using the same procedural architecture; per-building override layer that regeneration never destroys), prefabs/stamps, whole-world overview map, heightmap import/export (16-bit PNG/RAW). Painterly distance rendering ("matte painting" look from ref 03: aerial perspective, brush-stroke clouds, mist).
**G14 World pack & server-readiness:** `Mandate world pack` export/import, headless queries (height, slope, collision), `gouache_cli validate`, format docs (`docs/WORLD_FORMAT.md`), bit-identical results across OSes in CI; optional stretch: higher-quality GI.

# 5. Phase 1 look requirements (details)

- **Geometry:** every stone/brick/tile has rounded bevels, jittered rotation and size, mortar gaps; no two segments identical; corners use quoins; walls have slight batter; roofs have individual shingles; nothing razor-sharp or perfectly straight. Mesh budget: typical house ≤ 30k tris, large keep ≤ 150k (LOD0).
- **Materials:** procedural albedo with soft low-frequency color blotches and gentle stroke-like variation (hand-made feel), mostly high roughness, painted-in AO pockets, moss/stain/damage masks (more at bottoms/ledges/north). Material synthesizer lives in `gouache_paint` (CPU rayon and/or compute; cached on disk by hash). Output must be **soft and harmonious**, not visible stroke patterns.
- **Post stack (minimal and tasteful):** TAA, soft DoF (tilt-shift feel at diorama zoom), gentle bloom, vignette, color-grade LUT, filmic tonemap. **No heavy Kuwahara/outline/grain filters in Phase 1.** A painterly strength control may be added in G13 for distance/matte-painting only if it passes the loop.
- **Palette discipline:** curated limited palettes per season; no pure saturated primaries.

# 6. Scale architecture (Phase 2, design now, build after G10)

- **Precision:** CPU positions `f64` (`WorldPos`, meters). Rendering camera-relative: subtract camera in f64 on CPU, upload f32 offsets relative to a per-chunk origin. Reversed-Z, 32-bit float depth, adaptive near plane (0.05 m at ground up to hundreds of meters at altitude).
- **Hierarchy:** World → Region (16.384 km) → tile quadtree (power-of-two meters). LOD0 terrain tile = 32 m × 32 m with 129×129 samples (0.25 m). Root = 2¹⁸ m (extendable). Unit-test conversions, neighbor lookups, edge continuity.
- **Sparse by construction:** terrain = deterministic procedural base function + sparse user delta tiles (only touched tiles stored). GPU adds sub-meter visual detail that never affects gameplay height. Everything is a tile pyramid (height, material masks, color bake, building intent index) streamed by priority = screen-space error × editor attention.
- **Seamless zoom:** one camera 100+ km → 1.7 m; LOD by screen-space error (≤1 px geometry); geomorphing for terrain; dithered cross-fades for buildings/foliage. Draw distance ≥ 60 km with aerial perspective.
- **Streaming:** async jobs (Bevy tasks/rayon) → staging → per-frame upload byte budget; double-buffered swaps; never block render; configurable memory budgets (defaults 2 GB VRAM / 4 GB RAM at High).
- **Storage:** project directory, chunked key-value tile store (evaluate `redb`/`sled`/LMDB; ADR), zstd, versioned schema + migrations; open huge worlds instantly.

# 7. LOD systems (Phase 2)

**Buildings** (hysteresis + dithered cross-fade): LOD0 <30 m full geometry; LOD1 30–150 m decimated, merged per building with baked atlas; LOD2 150–600 m massing mesh merged per chunk; LOD3 600 m–3 km per-settlement impostors; LOD4 >3 km painted into terrain color tiles (tall silhouettes keep proxies). LOD meshes/atlases are generated by the engine (mesh simplification via `meshopt` or own quadric; baking via offscreen render) and cached by intent hash.

**Small foliage (grass, bushes, flowers) uses much tighter distances:** LOD0 <20 m full blades/petals/leaf cards + wind; LOD1 20–50 m thinned but bolder (coverage preserved); LOD2 50–100 m clumped cards/clusters, bushes as leaf-blob meshes; LOD3 100–400 m merged per-chunk painted tuft/flower-patch billboards or blobs; >400 m no instances, vegetation lives in painted terrain color/material tiles. Config constants (presets may scale them); coverage and average color constant across transitions.
**Trees:** near leaf cards → cluster blobs → merged canopy masses → painted into terrain, with their own longer chain.

# 8. Editor behavior & tools

Principles: everything reacts live; no grid; the system fills in the beauty; no precision demanded; in-world gizmos; spring/squish animation; ambient life; undo/redo for everything (command log); no modal dialogs for creative actions.

Tool belt (bottom-center icon row like ref 02, icons procedurally drawn): 1 Build (walls, towers, platforms, arches, stairs) · 2 Roofs · 3 Doors/Windows/Props (snap & adapt) · 4 Paths/Roads · 5 Paint (palette-constrained materials/colors) · 6 Terrain · 7 Vegetation (flowers, bushes, trees, ivy, grass) · 8 **Water (lakes, rivers, sea, and cascades/waterfalls)** · 9 Erase · 10 Photo mode (free camera, DOF, tiled hi-res export) · 11 Time of day/season.

Macro tools for Mandate (Phase 2): see G13.

# 9. Data model, determinism, crate layout

- **Store intent, not meshes.** A building/prop/path = a small parametric description (control curves, params, style seed, overrides), <1 KB. Meshes/textures are derived caches (keyed by intent hash).
- **Document = base generator seed + layered edit log:** procedural base ← generated settlements ← user edits/overrides. Undo/redo via command log with snapshots. Edits spatially indexed by tile.
- **Deterministic generation:** same intent + seed ⇒ bit-identical geometry on all OSes: own integer-hash RNG (PCG/xxhash-style), `libm` transcendentals, no HashMap-iteration dependence, no fast-math, no GPU-dependent results on gameplay data. CI compares mesh hashes across OSes.
- **Dependency graph with dirty propagation** for incremental, local, cancellable regeneration (wall edit → neighbors, roof, windows, terrain conform, vegetation clearing, LOD caches).

```
gouache_math      f64 world coords, tile coords, deterministic math (libm), RNG/hash, curves/splines
gouache_world     data model, tile quadtree, intent layers, edit log, storage, streaming policy   [NO GPU]
gouache_procgen   deterministic terrain base, noise, erosion bake, biome rules, settlement gen     [NO GPU]
gouache_buildings procedural architecture (walls, roofs, openings, props), LOD mesh generation    [NO GPU]
gouache_paint     procedural material/palette synthesis (CPU), look parameters
gouache_render    wgpu renderer, passes, GI, GPU-driven culling, virtual texturing, shaders
gouache_editor    Bevy app: tools, gizmos, UI, camera, input, undo/redo, photo mode
gouache_cli       headless: shot, match, bake, bench, validate, export world pack, hash-check
```
The first four must run headless with no GPU so the future Mandate server can reuse them (collision, heightfield queries, navmesh inputs). UI: egui for dev panels; player-facing UI is custom (minimal icon bar, in-world gizmos). Robustness: property-based tests and fuzzing for generators (random curves/footprints never panic, manifold meshes, no NaNs, bounded triangle counts).

# 10. Rendering pass sketch (initial; refine from study)

Depth/normal/material-ID prepass → cascaded + soft shadows → GI update (probe clipmaps, compute) → opaque pass with GI-lit shading → SSAO/bent normals → sky/atmosphere/clouds → water (reflections via SSR + sky + cheap tracing) → particles/smoke → translucent foliage/banners → TAA → DoF → bloom/tonemap/grade → UI (never filtered). GPU-driven culling with indirect draws where the backend supports it, CPU-batched chunk draws as fallback; no bindless arrays.

# 11. The Reference-Match Loop (mandatory; build as a real tool in Phase 0)

**You must compare your renders against reference pictures and keep reworking until they match.**

**11.1 Reference library.** `docs/reference/` holds the supplied images and any captured/collected ones (web-researched stills saved with source URLs in `docs/reference/SOURCES.md`; never invent references). Each gets `<name>.match.json`: `{image, target_scene, camera{pos,target,fov,time_of_day,season}, regions[{name,rect,what_it_should_show}], thresholds, notes}`. Regions isolate one thing (roof_teal_slates, stone_wall_base, tree_canopy, grass_with_bounce_light, shadow_edge, sky_backdrop, ui_gizmo). Estimate each reference's camera and sun direction (from shadows) and **reproduce it**. Recreate scenes with the editor tools or recorded tool-input scripts, never hard-coded meshes.

**11.2 `gouache_cli match --ref <name>`:** renders the scene at the reference camera/resolution; writes `docs/shots/match/<ref>/iter_NN/` with `ours.png`, `ref.png`, **`side_by_side.png`**, `flicker.png`, `diff_heatmap.png`, per-region side-by-side crops; computes perceptual-statistics metrics (exact pixel metrics are meaningless since geometry differs): OKLab k-means dominant colors + ΔE, luminance-histogram EMD and shadow/light ratio, saturation/hue histogram distance, warm-lit vs cool-shadow separation, per-region local contrast and edge density, radial frequency spectrum, SSIM/LPIPS-like only for regions meant to match (sky, UI gizmos); writes `report.json` and `report.md` with pass/fail per threshold.

**11.3 Visual judgement (required each iteration):** open `side_by_side.png` and every region crop; record in `docs/LOOKDEV.md` the **top 5 visible differences** (specific, region-referenced), the hypothesized cause (shader / geometry / lighting / palette / tool logic) and the fix. Then spawn an **independent critic sub-agent** that sees only `ref.png`, `ours.png`, region crops and the checklist and answers: "Which is the real game, which is ours? List what gives ours away; score each region 1–5."

**11.4 Rework loop:**
```
repeat: match -> open images -> list top 5 differences -> fix the top 3 (lighting first, then geometry, materials, foliage, post) -> re-run
until: all thresholds pass AND critic scores every region >= 4 AND no unresolved "gives ours away" item
```
Min 5, max 25 iterations per reference, then change technique and record why. One or two changes per iteration. Keep a filmstrip `progress.png`. Fix root causes in the engine; never hand-tune the single test scene, never bake or project reference pixels into the scene (that's cheating and invalid). Passing references become goldens; later changes must not regress them (`match --all` in CI, soft-fail on shared runners).

**Checklist (all must pass):** warm-sun/cool-shadow separation with visible bounce light · soft contact-hardening shadows, no acne/peter-panning · contact AO grounds everything · no two stones/tiles/segments identical · rounded edges, never razor-sharp · roofs with individual shingles, color variation, moss; chimneys with smoke · foliage reads as fluffy layered masses · cohesive palette · depth cues (DoF, atmospheric falloff, soft backdrop) · nothing "programmer art" (no flat shading, untextured surfaces, z-fighting, obvious tiling).

**11.5 Mapping:** G1 -> sky_backdrop, shadow_edge, grass_with_bounce_light (ref 01) · G2 -> terrain regions of all refs · G3 -> meadow/flowers (ref 03 foreground, ref 01 lavender/shrubs) · G4 -> tree_canopy (ref 01) · G5 -> water/leaf litter (ref 02) · G6 -> whole nature frames of refs 03, 01, 02 · G7/G8 -> stone_wall_base, roof_teal_slates, timber_frame, windows (refs 01, 02) · G9 -> whole frames of refs 01 and 02 · G13 -> ref 03 distance/atmosphere.

**11.6 Reporting:** at each gate send: path to latest `side_by_side.png`, metric table (before → after), critic verdict, 3 remaining differences, and the exact image files to open.

# 12. Verification beyond looks

Tile-edge continuity/no-crack tests, NaN/inf mesh scans, depth-precision tests at altitude extremes, cross-OS determinism hashes, temporal-stability test (60-frame camera move, shimmer metric), scripted-input replay tests for tools (drag wall → expected mesh/state, latency budget), perf benchmarks (`gouache_cli bench`: p50/p95/p99 frame times, per-pass GPU timings, memory) with budgets in repo.

# 13. Autonomy & engineering rules

- **Decide, don't ask.** Record ADRs in `docs/DECISIONS.md`. Revisit with evidence only. Blockers worth stopping for: missing hardware to validate a platform, a legal/licensing decision, a contradiction you can't resolve. (Plus the three USER STOP gates: G1, G6, G9.)
- **Vertical slices, not plans.** Data → generation → render → screenshot → test, end to end, before polishing. Never say "looks good" without the saved side-by-side.
- **Two honest attempts** at a technique; if it isn't delivering, change technique and record why.
- **Resumability:** keep `docs/PROGRESS.md` (done/in progress/next/known issues), `docs/BACKLOG.md`, `docs/LOOKDEV.md`, short `CLAUDE.md`. Small green commits; tag gates.
- **Quality:** fmt, clippy -D warnings, no `unsafe` without justification, public API docs, typed errors (`thiserror`), no panics on user-input paths, profiling built in from day one. Permissive-license deps only (`cargo deny`); avoid heavy C++ deps unless ADR'd.
- **Cross-platform discipline:** query wgpu limits/features; maintain `docs/BACKENDS.md`; test on all three OSes.
- **Scope guard:** no gameplay, networking, NPCs, inventory.
- **If a gate proves infeasible as written,** document why, deliver the best partial result with tests, continue; never silently drop acceptance criteria.

# 14. First actions (Phase 0, in order)

1. `cargo new --workspace`; crates per §9; `rust-toolchain.toml`; `.gitignore` (including the re_workspace path); license placeholder; `CLAUDE.md`; `docs/` skeleton.
2. Research (0.3) → `docs/STUDY/*`.
3. Bevy/wgpu version selection + integration spike → ADR-001.
4. Headless offscreen render + `gouache_cli shot` + PNG output; verify here.
5. CI (3 OS), fmt/clippy/deny.
6. Camera bookmarks, golden compare, contact sheets, **`gouache_cli match` skeleton** with the metrics in §11.2.
7. Hot-reload WGSL, GPU timestamps, frame-time overlay.
8. `gouache_math` basics with tests.
9. Experiments (0.6) a, b, d first (GI probes, stone coursing, soft shadows/AO), each compared to references.
10. If installed: 0.4 observation; if opted in: 0.5 analysis in a separate Analyst session.
11. Update `docs/PROGRESS.md`, tag `g0`, proceed to G1 and **stop for user review** when G1's side-by-sides are ready; then G2-G6 (nature) with the next stop at G6.
