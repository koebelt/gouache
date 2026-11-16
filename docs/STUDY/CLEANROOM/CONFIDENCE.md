# Clean-room confidence ledger

[CONFIRMED-STATIC] means the fact is stated directly in a permitted plaintext file
(credits-crates.txt, default_settings.ron, build-info/*, third-party-licenses.html headings and
links) or by a file or folder name, size or count. [INFERRED] means it is an interpretation of
those names.

## Architecture
- [CONFIRMED-STATIC] Rust 1.95.0, LLVM 22.1, opt-level 3, debug info on, aarch64-apple-darwin.
- [CONFIRMED-STATIC] 960 locked packages, 53 local, 39 from git.
- [CONFIRMED-STATIC] Vulkan via ash 0.38 (+1.3.281); MoltenVK and libvulkan bundled on macOS.
- [CONFIRMED-STATIC] No wgpu, naga, metal-rs, bevy_render or bevy_pbr in the lockfile.
- [CONFIRMED-STATIC] Bevy 0.17.3 core crates only (app, ecs, input, state, time, tasks, math, reflect).
- [INFERRED] Bevy is used only as ECS and scheduler. The renderer is fully custom.
- [CONFIRMED-STATIC] Renderer crates rhapsody, rhapsody-core, -deferred, -gi, -shadow, -taa, -dof, -post.
- [CONFIRMED-STATIC] rhapsody-gi depends on rhapsody-deferred.
- [CONFIRMED-STATIC] Shader tooling crates shabriri, shabriri-rt and shaderface. shabriri uses rspirv 0.12.
- [INFERRED] Shaders are precompiled to SPIR-V into `.precomp` blobs.
- [CONFIRMED-STATIC] gpu-allocator 0.26, vk-sync-fork and gpu-profiler come from the rhapsody2 repo.
- [CONFIRMED-STATIC] winit 0.29.15 fork; sdl2 0.38 + libSDL2; FMOD (libfmod 2.222.6); steamworks 0.11.
- [CONFIRMED-STATIC] License file links kajiya, FidelityFX SDK, XeGTAO, GPU-Raytracer.
- [INFERRED] These were ported as algorithms (FSR, GTAO, BVH traversal), not linked as libraries.
- [CONFIRMED-STATIC] rapier3d 0.22, parry3d 0.17, obvhs, bvh 0.10, mesh_to_sdf 0.4, meshopt 0.6,
  hexasphere 12, spade 2.11, rstar 0.12, bracket-noise 0.8, suncalc 0.4, dolly 0.6, palette 0.7.
- [CONFIRMED-STATIC] Serialisation: serde, ron, rkyv 0.8, bincode, speedy; turbosloth uses redb.
- [CONFIRMED-STATIC] UI and text crates: iced 0.12 fork, egui 0.27, taffy, cosmic-text 0.11 fork.
- [INFERRED] iced serves auxiliary windows (crash reporter). egui serves dev tools.
- [CONFIRMED-STATIC] Gameplay crates are named system-* and depend on country-core and bevy_ecs.
- [INFERRED] system-decorator uses physics plus graph search for placement.

## Render passes
- [CONFIRMED-STATIC] All shader folder names, file names and counts listed in RENDER_PASSES.md.
- [CONFIRMED-STATIC] Named groups nani/, wall/, grass2/, gi/, deferred/, water/, painterly/,
  vermeer/, light_binning/, prefix_scan/, garden/, paths/, terrain_editing/, rocky_terrain/, dof/.
- [INFERRED] GPU-driven culling and bucketed indirect draws ("nani").
- [INFERRED] HZB occlusion culling.
- [INFERRED] Ray-traced GI candidates resolved into temporally reprojected screen radiance.
- [INFERRED] XeGTAO-style AO with temporal and upsample steps.
- [INFERRED] Four deferred lighting models: solid, grass, plant, canopy.
- [INFERRED] Clustered light binning.
- [INFERRED] Froxel volumetric fog with its own shadow map; sky visibility volume.
- [INFERRED] Ray-marched water and ice reflections and refractions.
- [INFERRED] Grass rendered as GPU-culled card splats with separate shadow splats.
- [INFERRED] Two painterly stroke-based stylisation modes.
- [INFERRED] Custom temporal upscaler "tgsr" plus an FSR path.
- [INFERRED] Bloom-style blur pyramid; final_post handles tonemap and LUT.
- [INFERRED] DoF with autofocus.

## Procedural / content
- [CONFIRMED-STATIC] Sizes and counts: assets 1.9 GB, compiled-assets 801 MB, 753 hashed `.bin`.
- [CONFIRMED-STATIC] Theme names summer, autumn, winter, flowery, olden, and the per-theme file names.
- [CONFIRMED-STATIC] 228 decorator mesh files with suffixes _collision, _interaction, _outline, _glass,
  _hat, _rails, rankN and NxM.
- [INFERRED] Decorators carry physics, picking and outline proxies, and size or rank variants are
  chosen procedurally.
- [CONFIRMED-STATIC] Building-part mesh names: plank, roof tile, beam, spire, brick, stair step.
- [INFERRED] Buildings are assembled from instanced parts.
- [CONFIRMED-STATIC] 32 terrain_rt chunks. [INFERRED] They are a ray-tracing proxy.
- [CONFIRMED-STATIC] 28 LUTs, 1 tod file, 30+1 full starting builds with history.json and snapshots/.
- [INFERRED] Saves are edit-history replays.
- [CONFIRMED-STATIC] default_settings.ron: resolution scale 0.4; tutorial enabled.
- [INFERRED] The default render runs at 40% resolution and is temporally upscaled.

## Compliance checklist
- [x] No executable, .dylib, .metallib, .spv or other binary file was opened, hexdumped, run through
      `strings`/`otool`/`nm`, disassembled or decompiled. Bundle binaries were only listed by name and size.
- [x] No file inside `compiled-shaders/` or `compiled-assets/` was opened, including the small
      `shaders.list` / `crate-deps.list` text files. Only names and sizes were listed.
- [x] No file under `assets/` was opened. Only names, counts and sizes were listed.
- [x] Only permitted plaintext files were read: credits-crates.txt, default_settings.ron,
      build-info/{toolchain-info.json, Cargo.lock, manifest.json (structure and path names only)},
      and third-party-licenses.html (headings, license names and project links only, via grep and sed).
- [x] The game was not run.
- [x] No game file was copied anywhere. All commands were read-only.
- [x] These docs contain no code, shader text, constant tables or verbatim file contents beyond crate
      names and versions, file and folder names, counts and sizes. The two settings in
      default_settings.ron are paraphrased in prose.
