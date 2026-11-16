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
