# Plan (Phase 0 → 1)

G0 (now): foundations ✔, headless shot ✔, match tool ✔, experiments a (GI) ✔ b (stone courses) ✔ d (PCSS + AO) ✔,
study ✔; CI written but unverified (needs remote). Remaining polish: GPU timestamps/overlay, editor surface spike.

G1 lighting sandbox (USER STOP): iterate ref 01 regions with the loop. Planned engine work in priority order:
1. GTAO replacing hemisphere SSAO (stronger, correct contact AO); 2. probe depth moments (leaks);
3. screen-space near-field GI (contact bounce colour); 4. SDSM shadow fitting (sharper contact shadows, no serration);
5. sky-view LUT + painted backdrop ring (haze gaps lavender, not grey); 6. DoF (tilt-shift) + bloom;
7. foliage translucency test object (sphere clusters) to validate back-lighting before G4.
G2 terrain brushes → needs GPU SDF updates (ADR-003 evolution 1) and the editor window (ADR-001 spike).
G3-G6 nature per spec; buildings only after G6 approval.
