# Tiny Glade: engine and crate architecture (clean-room, static names only)

Sources used: `credits-crates.txt`, `build-info/Cargo.lock` (crate names, versions, dependency edges),
`build-info/toolchain-info.json`, `build-info/manifest.json` (file path list only),
`third-party-licenses.html` (license headings and project names only), and directory listings
of `TinyGlade.app`. No binary was opened. Tags: [C] = confirmed from a plaintext file or file name,
[I] = inferred. CONFIDENCE.md has the full claim list.

## Build and toolchain
- Rust 1.95.0, LLVM 22.1, opt-level 3 with debug info on, target and host aarch64-apple-darwin
  (native Apple Silicon build) [C].
- The lockfile has 960 packages. 53 are local workspace crates and 39 come from git, most of them
  forks hosted by the studio's own GitHub organisation [C].
- The renderer comes from a private git repository named `rhapsody2`, pinned to one revision [C].

## Platform layer
- Graphics API: Vulkan through `ash` 0.38 (Vulkan headers 1.3.281) plus `ash-window` 0.13 [C].
  On macOS the bundle ships `libMoltenVK.dylib`, `libvulkan.dylib` and a MoltenVK ICD manifest
  under `Resources/vulkan/icd.d`, so Metal is reached through MoltenVK, not directly [C].
- There is no wgpu, naga, metal-rs, bevy_render or bevy_pbr in the lockfile [C]. The renderer is
  fully custom [I].
- GPU memory comes from `gpu-allocator` 0.26 (a fork vendored in rhapsody2), barriers from
  `vk-sync-fork`, and GPU timing from `gpu-profiler` [C]. Crates named `aftermath-rs` (NVIDIA
  crash dumps), `dxgi-enumerate-gpus` and `tg-gpu-selector` point to GPU-selection and crash
  tooling on Windows [C names / I purpose].
- Windowing and input: `winit` 0.29.15 (studio fork), plus `sdl2` 0.38 (local copy) with
  `libSDL2` bundled, and a local `usdl` wrapper [C]. SDL is probably used for gamepads or Steam
  Deck input [I]. The asset folder `steam-controller/` holds 3 `.vdf` files [C].
- Audio: FMOD Studio (`libfmod` 2.222.6, with `libfmod.dylib` and `libfmodstudio.dylib`
  bundled) and 37 `.bank` files [C].
- Steam: `steamworks` 0.11 (fork) and `libsteam_api.dylib` [C]. A `geforcenow` crate and a
  `ui/gfn` folder show GeForce NOW support [C].

## Game framework (ECS)
- Bevy 0.17.3, used only for the core: `bevy_app`, `bevy_ecs`, `bevy_input`, `bevy_state`,
  `bevy_time`, `bevy_tasks`, `bevy_math`, `bevy_reflect`, `bevy_platform`, `bevy_utils` [C].
  There is no Bevy rendering, asset, audio, window or UI crate [C]. Bevy serves as the ECS and
  scheduler, and everything else is in-house [I].
- Gameplay is split into `system-*` crates that each depend on `bevy_app`/`bevy_ecs` and
  `country-core` [C]:
  `system-clutter`, `system-color`, `system-critters`, `system-decorator`, `system-glade-border`,
  `system-onboarding`, `system-roof`, `system-tree`, `system-wall-constructor`, `system-water`,
  plus `timberframe`, `forest`, `paths` and `toolbar` [C].
- `country-core` 0.2 is the central game and render-integration crate. It depends on every
  `rhapsody-*` crate, on `rapier3d`, `obvhs`, `mesh_to_sdf`, `hexasphere`, `bracket-noise`,
  `suncalc`, `libfmod` and `steamworks` [C].
- `system-decorator` depends on `rapier3d`, `petgraph` and `priority-queue` [C]. That suggests
  physics or collision queries and graph searches for placing decorations [I].

## Renderer stack ("rhapsody")
- `rhapsody` (core Vulkan abstraction; depends on ash, gpu-allocator, vk-sync, shaderface,
  shabriri-rt) [C]
- `rhapsody-core`, `rhapsody-deferred`, `rhapsody-gi`, `rhapsody-shadow`, `rhapsody-taa`,
  `rhapsody-dof`, `rhapsody-post` [C]. Each one is a feature module with its own shader folder [I].
- `rhapsody-gi` depends on `rhapsody-deferred` [C]. GI therefore runs on deferred G-buffer data [I].
- Shader toolchain: `shabriri` (build side, uses `rspirv` 0.12, `guppy` for workspace
  discovery, `speedy` serialisation), `shabriri-rt` (runtime loader) and `shaderface`
  (interface reflection) [C]. Shaders are compiled to SPIR-V at build time into per-crate
  `.precomp` blobs [I, from rspirv and the file extension]. `hotwatch` and `notify` point to
  hot-reload during development [I].
- `kajiya` 0.2 appears as a local crate with no dependencies. The license file links to
  EmbarkStudios/kajiya [C]. It is probably vendored for credit or for shader snippets, and is
  not a full renderer dependency [I].
- Credited GPU techniques that appear as local, dependency-free packages: `FidelityFX` (link to
  AMD FidelityFX SDK), `XeGTAO` (link to Intel GameTechDev/XeGTAO) and `GPU-Raytracer` (link
  to jan-van-bergen/GPU-Raytracer) [C]. These are license-attribution entries for ported
  algorithms: FSR upscaling, GTAO ambient occlusion, and BVH ray traversal [I].
- `fsr` appears as a dependency name of `tiny-glade` and `country-core` [C].
- BVH crates: `obvhs` (a fork from the kajiya author's account), `bvh` 0.10 [C]. These suggest
  software or compute ray tracing against a CPU-built BVH [I].

## Maths, geometry, procedural helpers
- `glam` (several versions: 0.20, 0.25, 0.27, 0.30), `macaw` 0.17, `nalgebra` 0.33 (for
  rapier/parry), `half` (f16) [C].
- Physics and collision: `rapier3d` 0.22 (fork), `parry3d` 0.17 [C].
- Geometry: `meshopt` 0.6, `mesh_to_sdf` 0.4, `hexasphere` 12, `spade` 2.11 (Delaunay),
  `rstar` 0.12 (R-tree), `kurbo`/`euclid` (2D curves, via UI) [C].
- Noise and randomness: `bracket-noise` 0.8, `fastrand`, `rand_xorshift`, `perchance` [C].
- Sun position: `suncalc` 0.4 [C]. This implies astronomically driven sun direction for
  time of day [I].
- Camera: `dolly` 0.6 (camera rig smoothing, by the kajiya author) [C].
- Colour: `palette` 0.7, `fast-srgb8`, and an `lcms2` dependency name (ICC colour management) [C].

## Data, serialisation, assets
- `serde`, `ron` 0.8/0.12, `serde_json` (fork), `rkyv` 0.8 (zero-copy archives), `bincode`,
  `speedy`, `toml`, `csv`, `yaml-rust2` (localisation `.yml`) [C].
- Asset pipeline: local `asset`, `asset-pipe`, `asset-pipe-build`, `foliage-pipe`,
  `clutter-icon-pipe`, `clutter-pack-pipe`. They use `turbosloth` (lazy, cached async build
  graph with a `redb` database), `blake3` hashing, `intel_tex_2` and `ispc_rt` (BCn texture
  compression), `ddsfile`, `gltf`, `image` and `meshopt` [C].
- Compression: `zstd`, `brotli`, `flate2`, `lzma-rs`, `zip` (mods), `ruzstd` [C].
- Integrity and mods: `tg-mods`, `tg-package-integrity`, `tg-integrity-report` and
  `manifest.json` (2,078 file-path and hash pairs) [C].

## UI, text, tooling
- In-game UI is driven by `.ron` layout files (75 under `assets/ui`) and probably a custom
  renderer (`ui_2d*` shaders) [C names / I]. `iced` 0.12 (fork) with `tiny-skia` is used for
  separate windows such as `crash-reporter-gui` and `tg-iced` [I]. `egui` 0.27 is used for
  debug and dev tools [I]. `taffy` handles layout [C].
- Text: `cosmic-text` 0.11 (fork), `rustybuzz`, `swash`, `ttf-parser` and `fontdb`, with 85
  `.ttf` files [C].
- Profiling: `puffin` with `puffin_http` [C]. Crash reporting: `sentry`, `minidump*` and
  `crash-handler` [C].
- Concurrency: `rayon`, `smol`, `tokio` (via `reqwest` for HTTP), `crossbeam`,
  `bumpalo` arenas [C].

## Architectural reading [I]
Bevy ECS sits on top of a bespoke Vulkan renderer ("rhapsody"). The renderer is split into
feature crates, each owning precompiled SPIR-V. Game-specific GPU work (procedural meshes,
foliage, water, painterly effects) lives in `country-core`'s shader set, and engine-generic
passes live in the `rhapsody-*` sets. Heavy GPU-driven rendering is implied by the indirect-draw
and culling shader names (see RENDER_PASSES.md).
