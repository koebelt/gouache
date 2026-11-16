# Backend notes (wgpu 30.0.2)

| Item | Metal (macOS) | Vulkan (Linux/Win) | DX12 (Win) |
|---|---|---|---|
| Verified adapter | Apple M5 Pro, Metal 4 (2026-11-16) | — (CI lavapipe pending) | — |
| TIMESTAMP_QUERY | yes (feature-gated) | yes | yes |
| MULTI_DRAW_INDIRECT_COUNT | **no** → avoid | yes | yes |
| multi_draw_indirect (plain) | emulated as loop | native | native |
| Bindless texture arrays | **no** → avoid | yes | yes |
| Workgroup shared memory | 32 KB, we budget ≤16 KB | ≥16 KB | 32 KB |
| R16F 3D textures filterable | yes | yes | yes |

Rules: request only `OPTIONAL_FEATURES` (context.rs) when present; never require them.
