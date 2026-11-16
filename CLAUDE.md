# Gouache — working notes for Claude

Procedural map-editor engine in Rust (wgpu + Bevy ECS) for the MMORPG **Mandate**, aiming at the
*Tiny Glade* diorama look. Master spec: `docs/MASTER_SPEC.md` (do not edit it; record deviations in DECISIONS).

## At session start, read (in this order)
1. `docs/MASTER_SPEC.md` — mission, phases, gates, hard decisions, Reference-Match Loop (§11)
2. `docs/PROGRESS.md` — done / in progress / next / known issues (resume from here)
3. `docs/DECISIONS.md` — ADRs; don't reopen without new evidence
4. `docs/LOOKDEV.md` — per-iteration visual findings (top-5 differences, causes, fixes)

## Commands
```sh
cargo build --release -p gouache_cli                       # headless tools (use release: GI is heavy)
cargo test --workspace                                     # unit tests (no GPU needed except render smoke)
cargo clippy --workspace --all-targets -- -D warnings      # must be clean
cargo fmt --all                                            # rustfmt.toml: width 110
cargo deny check                                           # licenses/advisories (deny.toml)

# Render a bookmark (camera + scene) to PNG; views: lit|indirect|ao|shadow|normals
./target/release/gouache_cli shot --bookmark bookmarks/g0_blocks_overview.json --out docs/shots/tmp/x.png
# Reference-Match Loop iteration (writes docs/shots/match/<ref>/iter_NN/…)
./target/release/gouache_cli match --ref 01_tinyglade_town
./target/release/gouache_cli contact --out sheet.png a.png b.png …
./target/release/gouache_cli golden --candidate a.png --golden b.png
./target/release/gouache_cli hash --scene scenes/g0_blocks.json   # cross-OS determinism
```
Shaders hot-reload from `assets/shaders/` (`GOUACHE_SHADERS` overrides the path).

## Architecture map
```
crates/gouache_math      f64 WorldPos, TileCoord quadtree, PCG32/hash, Catmull-Rom, MeshData   [no GPU]
crates/gouache_procgen   gradient noise/fBm, Heightfield + terrain colouring                    [no GPU]
crates/gouache_world     SceneDoc, Look, Bookmark (JSON intent documents)                        [no GPU]
crates/gouache_buildings beveled primitives, scene_build (intent -> meshes)                        [no GPU]
crates/gouache_paint     procedural materials/palettes (stub)
crates/gouache_render    wgpu renderer: context, shader lib (#import + hot reload), volume (CPU SDF
                         voxeliser), renderer (frame graph), frame (uniforms/camera)
crates/gouache_editor    Bevy app (stub until G1/G2)
crates/gouache_cli       shot / contact / match (metrics.rs, matcher.rs) / golden / hash
assets/shaders/          WGSL: common, atmosphere, sky_sh, shadow, gbuffer, gi_common, gi_update,
                         ssao, lighting, accumulate (TAA), tonemap (AgX)
scenes/  bookmarks/      committed JSON scene docs and camera bookmarks
docs/reference/          reference images (git-ignored, local only) + <name>.match.json (committed)
docs/shots/              match iterations (iter_* ignored), side-by-sides referenced from LOOKDEV
docs/STUDY/              Phase 0 research
```
Frame graph: sky SH → sun shadow map (PCSS) → G-buffer → GI probe update (compute, SDF ray march) →
SSAO/bent normal → deferred lighting (compute) → temporal accumulation → AgX tonemap. Reversed-Z, Y up,
north = −Z. `Frame` uniform in `common.wgsl` must mirror `FrameUniforms` in `frame.rs` exactly.

## Hard rules
- Never claim something "looks good" without a saved side-by-side vs a reference; open and look at it.
  Record top-5 differences in `docs/LOOKDEV.md` every match iteration; run the critic sub-agent (§11.3).
- Never bake/project reference pixels into scenes; never hand-tune one scene. Fix engine root causes.
- 100% procedural art. No imported assets. Reference images are never committed (`.gitignore`).
- Deterministic generation: `gouache_math` hashes/PCG, `libm`, no HashMap iteration in generators.
- First four crates (math/world/procgen/buildings) must never depend on wgpu/GPU.
- Metal is the common denominator: no bindless, no MULTI_DRAW_INDIRECT_COUNT, ≤16 KB workgroup memory.
- fmt + clippy `-D warnings` clean; typed errors (`thiserror`); no panics on user-input paths; no `unsafe`.
- USER STOP gates: G1, G6, G9 — stop and wait for review. Other gates: report and continue.
- Commit at every green checkpoint; tag gates (`g0`, `g1`, …); keep `docs/PROGRESS.md` resumable.
- Clean-room rule (0.5): no decompiler output, binary strings or analysis databases in the repo, ever.
