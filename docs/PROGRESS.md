# Progress

_Last updated: 2026-11-16 (session 1)._ Resume: read this, then DECISIONS.md and LOOKDEV.md.

## Current phase: PHASE 0 (Foundations + Study) → gate G0

### Done
- Workspace (8 crates per §9), toolchain pinned 1.98.1, rustfmt/clippy clean, lints in workspace.
- `gouache_math`: PCG32 (golden-tested), integer hashes, f64 `WorldPos`, `TileCoord` quadtree, Catmull-Rom with
  arc-length, `MeshData` (validate, content hash).
- `gouache_procgen`: deterministic gradient noise/fBm, `Heightfield` (smooth normals, curvature AO, slope colouring).
- `gouache_world`: `SceneDoc`, `Look`, `Bookmark` JSON. `gouache_buildings`: rounded beveled boxes, scene builder.
- `gouache_render`: headless wgpu (Metal verified on M5 Pro), WGSL lib with `#import` + naga validation + mtime
  hot reload, CPU SDF/albedo voxeliser, frame graph: sky SH, PCSS shadows, G-buffer, GI probes (SDF ray march,
  multi-bounce), SSAO+bent normals, deferred lighting, TAA/accumulation, AgX tonemap. Debug views.
- `gouache_cli`: `shot`, `contact`, `match` (full §11.2 metrics + images), `golden`, `hash`.
- First lit test scene renders: docs/shots/g0/g0_debug_sheet.png.

### In progress
- Reference `.match.json` for refs 01/02/03 (camera/sun estimates, regions).
- Study docs (`docs/STUDY/*`).

### Next (Phase 0 order, §14)
1. Study docs from research done (kajiya GI summary, Tiny Glade sources) → STUDY/{RENDERING,PROCEDURAL,UX,OPEN_QUESTIONS,SOURCES,PLAN}.md
2. CI workflow + deny.toml (needs a GitHub remote to verify 3-OS green — ask user at G1 stop).
3. GPU timestamps + frame-time overlay; editor surface spike (ADR-001).
4. Experiment (b) stone coursing along a curve; (d) stronger contact AO; (a) GI vs reference.
5. G0 report → G1 lighting sandbox (USER STOP).

### Known issues
- SSAO weak at contacts; shadows not violet enough; sky grey (see LOOKDEV).
- No git remote: CI unverified.
- Tiny Glade GPC 2024 talk (youtube jusWW2pPnA0) has no reachable transcript; content unknown beyond titles.
