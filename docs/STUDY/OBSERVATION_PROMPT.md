# Prompt for the Claude desktop app — Phase 0.4 black-box observation of Tiny Glade

Paste everything below the line into a new Claude desktop session on this Mac. First turn on
**Settings → This computer → Computer use**, and give your terminal **Screen Recording** permission
(System Settings → Privacy & Security).

---

You are helping with **Phase 0.4 (black-box observation)** of the Gouache project at
`/Users/koebelt/Documents/dev/gouache`. Read `CLAUDE.md`, `docs/MASTER_SPEC.md` §0.4 and §11,
`docs/STUDY/TALK_GPC2024.md`, and `docs/STUDY/CLEANROOM/*.md` first so you know what we already know and what we need.

**Goal.** Play Tiny Glade (my own Steam copy; Steam must be running, the game is
`~/Library/Application Support/Steam/steamapps/common/Tiny Glade/TinyGlade.app`) as a normal user and capture a private
reference pack so we can match its lighting, materials and tool behaviour with our own procedural engine.

**Rules.**
- Observation only: play the game and take screenshots. Do not modify game files, install mods, attach debuggers,
  inspect memory, use GPU frame-capture tools, or open the game's files.
- Captures are private reference material. Save them only under `docs/reference/captured/` (git-ignored). Never commit,
  upload or share them. Commit only prose notes.
- Use computer-use tools to drive the game (request access to Tiny Glade and Steam). Save full-resolution frames with the
  repo's helper from a shell: `scripts/capture.sh <label> <frames> <interval_s> "<note>"` (it waits 3 s, then captures the
  screen with `screencapture`; it appends metadata to `docs/reference/captured/index.jsonl`). For motion sequences, start
  the script in the background, then perform the action during its capture window.
- Before capturing, record the game's settings screens in your notes (graphics options, render scale, upscaler, DoF,
  photo mode controls, time-of-day and season controls). Prefer 1920×1080. Hide the UI for material and lighting shots
  (photo mode) unless the shot is about UI.

**Build one test scene** in a new glade: a few walls, a tower with a roof, a two-storey house with windows and
timber framing, an arch where a path crosses a wall, a small pond, some trees, bushes, flowers and grass. Keep this same
scene for all sweeps.

**Capture list** (label → what).
1. Same-scene sweeps: `sweep_<season>_<time>_<height>` for 4 seasons (spring, summer, autumn, winter) × 4 times
   (morning, noon, golden_hour, night) × 3 camera heights (eye ~2 m, mid ~15 m, high ~60 m) — 48 frames, same framing per height.
2. Material close-ups (UI hidden, ~2–4 m away, noon and golden hour): `mat_stone`, `mat_roof`, `mat_plaster`,
   `mat_timber`, `mat_foliage`, `mat_grass`, `mat_flowers`, `mat_water`, `mat_path`.
3. Lighting probes: `probe_sunlit_wall_grass` (a sunlit coloured wall beside grass), `probe_deep_shadow_corner`,
   `probe_arch_tunnel` (looking through an arch), `probe_room_one_window` (an enclosed space lit through one window).
4. Tool behaviour sequences (12–30 frames at 0.15–0.25 s): `tool_drag_wall`, `tool_close_loop`, `tool_raise_height`,
   `tool_add_window`, `tool_glue_buildings`, `tool_path_through_wall`, `tool_erase`, `tool_undo_redo`,
   `tool_terrain_brush`, plus `tool_roof_change`, `tool_tree_place`, `tool_water_carve`.
5. UI: `ui_toolbar`, `ui_gizmo_house`, `ui_gizmo_wall`, `ui_settings` (one frame each).
6. Depth of field/tilt-shift: `dof_tiltshift_on`, `dof_tiltshift_off` at the same framing.

**Notes to write** (commit these; prose only):
- `docs/STUDY/OBSERVED.md`: for every capture group, what you observed, tagged **[OBSERVED]**: e.g. how a wall
  regenerates while dragging (latency, spring/overshoot, stones appearing), how windows erode plaster, how arches
  appear where paths cross walls, terrain conforming, undo/redo animation, grass trampling and wind, shadow softness and
  colour, bounce light, sky and haze colours per season/time, DoF behaviour, settings available. Include rough timings
  and sizes (e.g. "spring-in ≈ 0.3 s", "bricks ≈ 25 cm").
- Add an index table of all labels with a one-line description to `docs/STUDY/OBSERVED.md`.
- Update `docs/STUDY/OPEN_QUESTIONS.md` (mark 0.4 done or what remains) and add a line to `docs/PROGRESS.md`.
- Commit with message "Phase 0.4: black-box observation notes" (captures stay uncommitted). Then tell me which
  captures exist and the five most important observations for matching the look.
