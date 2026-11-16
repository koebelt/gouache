# Lookdev journal

Every Reference-Match iteration records: side-by-side path, top-5 visible differences (region-referenced),
hypothesised cause (shader / geometry / lighting / palette / tool logic), fix applied, critic verdict.

## G0 — first lit test scene (2026-11-16)
`gouache_cli shot --bookmark bookmarks/g0_blocks_overview.json` (no reference comparison yet; sanity only).
Debug sheet: docs/shots/g0/g0_debug_sheet.png (lit / indirect / ao / shadow).
- Indirect view shows correct colour bleeding (terracotta → ground, sunlit ground → box bases) and sky fill.
- Contact-hardening shadow edges present; sharp at contact, softer with distance.
- Issues: (1) sky is dull grey — sky radiance not scaled with sun intensity [fixed]; (2) whole image blurred by
  TAA history re-sampling each still frame [fixed: still-camera accumulates at the same pixel]; (3) SSAO
  too weak at contacts; (4) shadows read olive, not violet (ground albedo × blue sky); (5) sky colour cold
  grey vs. warm lavender/peach haze of ref 01.

## G1 — Reference-Match Loop on ref 01 (scene scenes/g1_lighting.json)
Images: docs/shots/match/01_tinyglade_town/iter_NN/ (local, git-ignored); key side-by-sides copied to docs/shots/g0/match01_iter{00,04,07}.png.

| iter | change (1-2 per iteration) | gated-region result |
|---|---|---|
| 00 | baseline (sun az 71, default look) | all fail; frame grey-beige, no warm/cool split (0.010 vs ref 0.066) |
| 01 | warm sun tint, violet ambient tint, sat 1.25; tool: dither-free stats + per-region gating (ADR-009) | mustard, too saturated; region content mismatched |
| 02 | ref-01 massing laid out in camera space; aerial haze + diorama edge fade | haze far too strong (washed out) |
| 03 | camera closer, haze ÷3 | framing overshoot; walls in shade read neutral grey |
| 04 | **front-left key light** (az 200, el 35) + GI ×1.3 (hypothesis: ref lights camera-facing surfaces) | first "warm clay diorama" read; grass palette passes |
| 05 | `ours_rect` for shadow_edge (same subject); sky tint peach; sat 1.3 | shadow_edge palette passes (5.1) |
| 06 | haze = sky light + sun in-scatter; min penumbra 3 texels (fence serration) | backdrop ΔE 16→10, warm/cool matches |
| 07 | sky fill ×0.75, display contrast 1.15 | backdrop shadow/light passes; whole-frame warm/cool still 0.003 |

### Top 5 differences after iter 07 (region-referenced)
1. whole frame / shadow_edge: darkest tones are warm brown; ref darks are lavender-violet (canopy gaps, shaded walls). Cause: lighting — sky ambient too weak vs warm ground bounce in shade; no cool contribution in occluded areas. Fix: GTAO multi-bounce with cool tint + separate ambient for occluded vs. open (G1 next).
2. shadow_edge: ours shadow/light ratio 0.66 vs ref 0.47 — shadows too filled. Cause: GI ×1.3 + haze lift. Fix: reduce haze on near pixels, stronger AO at contacts.
3. grass_with_bounce_light: value histogram narrow (no blade texture). Cause: geometry/content (G3). Informative.
4. backdrop_haze: chroma 0.04 vs 0.09 — ref backdrop is orange canopy + lavender gaps. Cause: content (G4 trees) + backdrop is plain haze. Fix: painted backdrop ring (G1/G6).
5. blocks: lit vs shaded faces too similar in value; ref has crisper form. Cause: lighting — exposure/haze compress range. Fix: lower haze near, raise sun/sky ratio.

### Experiment (b) stone courses — docs/shots/g0/exp_b_vs_ref02.png
Believable irregular beveled stones conforming to terrain. Vs ref 02: too brick-regular, low hue variance, mortar not dark enough, no multi-course stones, flat top cap → G7.
| 08 | `shade_tint` (cool ambient where sun doesn't reach), no haze < 30 m, AO^2 | whole-frame palette passes; shadow_edge warm/cool passes (0.040) |
| 09 | shade more lavender, warm sky tint (critic #1/#2) | shaded faces rose-lavender; shadow_edge palette 4.9 ok |

### Critic (blind, iter 07): identified the WIP with 100% confidence. Scores: shadow_edge 2/5, grass 3/5, backdrop 1/5.
Gives away: warm (not violet) shadows; flat pink-beige backdrop with hard grey sky wedge; haze reads as fog not light;
no bloom/highlight roll-off; flat ground; uniformly soft shadows (weak contact hardening). Iter 08-09 address #1.
Unresolved: grey sky void, bloom, contact hardening, backdrop content.
