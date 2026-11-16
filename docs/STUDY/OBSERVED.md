# Phase 0.4 — black-box observation notes (partial, session 1)

_Recorded 2026-11-16. Tiny Glade v1.16.5a (Steam, macOS, Apple M5 Pro). Observation only: played as a user,
screenshots only. Captures are private and git-ignored in `docs/reference/captured/` (index: `index.jsonl`).
Prose only here._

## Capture method (and limits)
- The shell has no Screen Recording permission, so `scripts/capture.sh` could not be used (`screencapture` fails).
- Stills: the game's own photo mode (shutter button) saves **3600×2260** images (PNG once the in-game
  "Screenshot format" setting is PNG; the first two were JPEG) to `~/.local/share/Tiny Glade/Steam/<id>/screenshots/`.
  These were moved into `captured/`.
- Sequences and UI frames: computer-use screenshots (**1372×891 JPEG**, a downscale of the 3024×1964 display).
  The effective interval is about 0.3–0.4 s, limited by screenshot latency.
- Input limits: synthetic clicks need an explicit press/hold/release (≥0.1 s). Letter keys only register when
  held (`hold_key`), and number hotkeys did not register, so tools were picked from the toolbar. **No right- or
  middle-button drags were possible, so the camera could not be rotated.** Camera control was limited to WASD/QE and
  scroll zoom (zoom pivots on the cursor). This blocked repeatable framings and some shots.
- "Logo in screenshots" could not be toggled off: photo-mode PNGs carry the logo in the bottom-right.

## Settings recorded [OBSERVED]
- Video › Image: Quality preset (Custom → … → Quality → **Ultra Quality**), Upscaling (No / **TGSR2**),
  Upscaling preset, Depth of field (Yes/No), **Resolution scale** (was 0.4 on Custom; 1.0 on Quality; **1.5** on
  Ultra), Depth of field (Photo mode), Brightness, Gamma, Sharpening.
  Captures use **Ultra Quality** (TGSR2, scale 1.5). The original setting was Custom at 0.4.
- Video › Display: device, Fullscreen, monitor, V-Sync, triple buffering, frame-rate limit 60. No resolution option.
- Game: hide glade border outline, **Time passage speed** (default 1.0, minimum 0.1; set to 0.1, so time still
  drifts), screenshot format JPEG/PNG (set to PNG), always-on input hints.
- UI: icon size, toolbar alignment, auto-hide toolbar, custom cursor, logo in screenshots.
- Controls: rotation/movement speed, invert X/Y, camera rotation pivot = cursor point, zoom pivot = cursor point.
  Key bindings: WASD move, MMB move, Q/E down/up, RMB rotate, scroll zoom; 1–9 tools, P photo, T time;
  Z/X undo/redo; Delete; RMB focus/deselect; Ctrl disables snapping (eraser while a tool is active); H hides UI in
  photo mode; Space opens quick tools (experimental).
- New glade themes: **Summer, Autumn, Olden, Flowery, Winter, Daily theme**. There is no "spring", and **season is
  chosen per glade at creation**. No in-glade season control was found, so a same-scene 4-season sweep is not
  possible as specified.
- Photo mode toolbar: open folder, camera mount, view, aperture (DoF), focus, shutter, exposure, filters/LUT,
  **render style** (three independent toggles: *painterly* brush-stroke filter, *clay* white-albedo render, and an
  untested "eye" style), time panel, close.
  The clay style persists in normal play until toggled off.
- Time panel (photo mode and main toolbar): a 24 h **time dial** (top = noon, left ≈ morning, right ≈ evening,
  bottom = midnight) with a lock; a **sun-direction compass** dial; and fog sliders for density, height and falloff.

## Observations by group [OBSERVED]
**Glade framing.** The playable glade is a clearing (~60–80 m) ringed by dense round-canopy forest that runs to
the horizon. Beyond it are rolling khaki hills with rock clusters. A dashed glade-border line appears near the
edges. The main menu background is the live scene through the painterly brush-stroke filter.

**Lighting by time (summer, mid camera ~18 m).**
- Noon: bright yellow-green grass with warm lit tips; tree shadows soft-edged and **mid-green, not dark** (strong
  sky fill); clear blue sky with flat-bottomed cumulus; mild aerial haze on the hills.
- Morning (dial ≈ −80°): heavy **pale violet-white height fog / mist**; low sun glowing behind the treeline; the
  glade sits in tree shadow and only the foreground bushes are sunlit.
- Golden hour → sunset (dial ≈ +80°): magenta-pink sky; glade fully shaded in lilac-violet; warm rim light only on
  treetops; floating sparkle motes in the air.
- Night (dial at bottom): **saturated blue-violet moonlight, readable rather than black**; fairly crisp moon
  shadows on grass; stars; yellow fireflies; **windows glow warm**.
- Clay render (golden hour): warm peach key against **violet/lilac** shadow and sky fill; soft contact shadows; AO
  under eaves and balconies. This is a clean target for G1 lighting-only matching.

**Depth of field.** Photo mode defaults to a strong **tilt-shift**: a narrow in-focus band at the focus reticle
and heavy round bokeh on near and far foliage. Main mode also applies a mild DoF (foreground bushes soft).

**Tower placement.** The building tool shows a timber-scaffold ghost under the cursor. On click, a plaster tower
with a terracotta shingle cone roof appears within ~0.15 s. The **roof springs taller over ~0.5 s**
(overshoot/settle), and bushes and lavender spawn around the base. **Ivy grows up the walls over the following
tens of seconds.**

**Wall drag.** The castle wall grows live to the cursor along a smooth curve with no visible latency at ~0.3 s
sampling. It is laid in irregular brick courses (bricks ≈ 25–35 cm, rounded edges, warm sandstone/terracotta
tones). Crenellations appear along the top, the wall **tapers lower toward the free end**, and it snaps to the
tower. Ivy appears on it over time.

**Path tool.** The path paints live under a soft ring cursor as packed dirt, with soft edges cutting the grass
away. **Where a path crosses a wall, a round-headed arch opening appears automatically**, with brick voussoirs and
the wall continuing above. This was seen after drawing a second path through the wall; it is unclear which stroke
triggered it.

**Default house (Summer start).** Pink/salmon plaster with blue-grey timber framing, a blue-grey slate roof with
irregular shingles, dormers, a balcony on stilts, a brick chimney with smoke, stone steps, and a flower box.

## Capture index
| label | frames | what |
|---|---|---|
| ui_toolbar | 1 | summer start glade, bottom toolbar (11 tools), top-right feedback/settings |
| default_house_noon_tiltshift | 2 | photo-mode tilt-shift of default house, near noon (JPEG 3600×2260) |
| tool_path_draw | 8 | path tool drag, dirt path appears live (755 px) |
| tool_place_tower | 4 | scaffold ghost → tower pops, roof springs, bushes spawn |
| tool_drag_wall | 8 | castle wall drag from tower, live growth, crenellations, taper |
| tool_path_through_wall | 3 | path across wall → auto arch |
| clay_golden_overview | 1 | photo mode, clay style, golden hour, tilt-shift (3600×2260 PNG) |
| clay_dof_near_focus | 1 | clay style, focus on foreground bush, strong bokeh |
| clay_eye_dusk | 1 | clay style in main mode, ~4 m height, dusk, arch in wall |
| style_painterly_clay | 1 | painterly + clay toggles together |
| sweep_summer_noon_mid | 1 | noon, mid camera |
| sweep_summer_morning_mid | 1 | morning mist, mid camera |
| sweep_summer_golden_hour_mid | 1 | just after sunset, mid camera |
| sweep_summer_night_mid | 1 | night, mid camera |
| sweep_summer_night_high | 1 | night, high camera ~60 m |

## Not captured (remaining)
- Sweeps: summer morning/noon/golden at the high camera, all summer eye-level shots, and every other season.
  Seasons need separate Autumn/Winter/Flowery glades, which breaks the "same scene" requirement.
- Scene items not built: two-storey house with added windows, pond, trees, bushes, flowers.
- Material close-ups (`mat_*`), lighting probes (except the arch/clay frames above), and the tool sequences for
  close loop, raise height, add window, glue, erase, undo/redo, terrain brush, roof change, tree place and water
  carve.
- UI gizmos (house/wall) and the DoF on/off pair.
- Unblocking: grant Screen Recording to the app running the shell (for `capture.sh`) and find a way to issue
  RMB/MMB drags for camera rotation (or a human operator for camera framing).
