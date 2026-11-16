# Procedural systems study

- One generator per element ("not one algorithm but many"), each with creation/assembly/morphing/deforming stages
  [SOURCE S3]. Every rule hand-authored by Anastasia Opara [SOURCE S3].
- Started as a Bevy wall generator: walls, archways, pathways, grass [SOURCE S10].
- Windows: multi-panel blending, auto damage; **one float vertex attribute = signed distance to windows**, used to
  erode plaster near openings [SOURCE S7]. Windows on roofs and wall/roof edges [SOURCE S7/S8].
- Tower roofs procedurally from footprint [SOURCE S8]; buildings can be glued together [SOURCE S9].
- Bricks poke through plaster; far LOD bricks are an SDF [SOURCE S4, S6].
- Ref 01/02 [OBSERVED]: stones rounded, size/colour jitter, mortar recess, crenellations, flagstone paving as irregular
  polygons (Voronoi-like), shingles individually jittered with moss blotches, timber frames with diagonal braces,
  jetties, stepped (crow-step) gables, flower boxes, bunting catenaries, chimney smoke as thin wavy ribbons.

Gouache status: beveled primitives; stone courses along curves conforming to terrain (experiment b,
docs/shots/g0/exp_b_vs_ref02.png). Gaps vs ref 02: stones too brick-like/regular, need larger hue variance, multi-course
stones, irregular top cap, deeper mortar shadow → G7.
