# Progress

_Last updated: 2026-11-16 (session 1, stopped at G1 for user review)._ Resume: read this, then DECISIONS.md and LOOKDEV.md.

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

### In progress — G1 lighting sandbox: **USER STOP reached (awaiting review)**
- Match loop on ref 01: 10 iterations (00-09), critic run once (iter 07). Evidence: docs/shots/g1/
  (side_by_side_iter09.png, regions_iter09.png, report_iter09.md, progress.png). Gated regions do NOT all pass yet.
- G0 status: done except CI 3-OS verification (no remote), GPU timestamps/overlay, editor surface spike.
- Experiments: (a) GI ✔, (b) stone courses ✔ docs/shots/g0/exp_b_vs_ref02.png, (d) PCSS+AO ✔ (AO weak-ish).

### Phase 0.3-0.5 follow-up (2026-11-16)
- GPC 2024 talk studied → docs/STUDY/TALK_GPC2024.md, ADR-010 (GI direction change, proposed).
- 0.5 pass 1 done (shipped plaintext only) → docs/STUDY/CLEANROOM/. Pass 2 (static binary analysis) is for a separate
  Analyst session started by the user: docs/STUDY/ANALYST_PROMPT.md. Implementer must not read its raw output.
- 0.4 observation: run docs/STUDY/OBSERVATION_PROMPT.md in the Claude desktop app with Computer use; captures go to
  docs/reference/captured/ (git-ignored) via scripts/capture.sh; prose to docs/STUDY/OBSERVED.md.
- Spec instruction from user: finish 0.4 and 0.5 before going further.

### Next (after user review)
1. Critic items: sky void → warm gradient/painted backdrop ring; bloom + highlight roll-off; contact-hardening (SDSM/
   tighter shadow fit + smaller min penumbra near contact); GTAO.
2. Re-run critic each iteration until regions >= 4 and thresholds pass (max 25 iterations).
3. G0 leftovers: GPU timestamps + frame-time overlay; editor window spike (ADR-001); CI once a remote exists.

### Known issues
- SSAO weak at contacts; shadows not violet enough; sky grey (see LOOKDEV).
- No git remote: CI unverified.
- Tiny Glade GPC 2024 talk (youtube jusWW2pPnA0) has no reachable transcript; content unknown beyond titles.
