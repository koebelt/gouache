# Prompt for the clean-room Analyst session — Phase 0.5 pass 2 (static binary analysis)

Start a **new, separate** Claude Code session (terminal or desktop app) in `/Users/koebelt/Documents/dev/gouache` and
paste everything below the line. Never paste this session's raw output into the Gouache implementation session.

---

I am the owner of a purchased Steam copy of Tiny Glade installed at
`~/Library/Application Support/Steam/steamapps/common/Tiny Glade`. **I explicitly instruct you to perform the static
analysis below on my local copy.** You are the **Analyst** in a clean-room process (`docs/MASTER_SPEC.md` §0.5). A separate Implementer
reads only your prose, never the binary or your raw output.

Read first: `docs/MASTER_SPEC.md` §0.5, `docs/STUDY/CLEANROOM/*.md` (pass 1, built from shipped
plaintext files), `docs/STUDY/TALK_GPC2024.md`.

**Allowed (static, read-only, on the local install):**
- Executable in `TinyGlade.app` and any `.dylib`: `otool -L/-l`, `size -m`, `nm` (if symbols exist), `strings`,
  `lipo -info`, `codesign -dv` (no modification).
- `compiled-shaders/`: the `.list` files and SPIR-V **metadata only** (entry-point names, OpName debug names, descriptor
  bindings and image formats, workgroup sizes, push-constant sizes), using a small throwaway Python parser or
  `spirv-dis`/`spirv-cross --reflect` if already installed. Do not install new tools without asking me.
- `compiled-assets/`: file headers/structure (format magic, counts) only.

**Raw output rule:** every dump, string list, script and scratch note goes ONLY under `~/gouache_re_workspace/` (outside the
repo). Read game files in place; never copy them. Never write raw strings, symbol lists or script output into the repo.

**Still forbidden:** running, debugging, hooking, injecting or memory-inspecting the game; GPU frame capture; decompiling to
pseudo-code or disassembling functions to reconstruct algorithms; transcribing code, shader instructions, constant tables,
assets or text; bypassing DRM/anti-tamper; descriptions so precise that they are the code in prose.

**Study only:** crate/engine versions; ECS organisation (type/system/schedule names as evidence, summarised); render pass
order and resource formats/resolutions (approximate); compute passes for GI, denoise, DoF, upscaler, water, grass;
generator stages and intermediate data structures (from type/module names and panic-path module names); regeneration
triggers; undo structure; tunable parameters as rough ranges only (e.g. "GI at quarter res"), never exact constants.

**Output:** update `docs/STUDY/CLEANROOM/{ARCHITECTURE,RENDER_PASSES,PROCEDURAL_SYSTEMS,CONFIDENCE}.md`, tagging new
claims [CONFIRMED-STATIC] or [INFERRED], each file under ~180 lines, prose and short factual items only. Extend the
CONFIDENCE.md checklist: raw outputs only in `~/gouache_re_workspace/`, nothing from the binary quoted in the repo, no
code/shader/constant transcription. Commit only those four docs with the message
"Clean-room (0.5) pass 2: static analysis findings (prose only)". End with a prose summary most useful for a wgpu
compute-GI renderer and procedural building/terrain/foliage tools: frame graph order, internal resolutions, GI ray budget
and denoise stages, shadow setup, grass/tree rendering, brick/part instancing, terrain representation, undo and
regeneration model.
