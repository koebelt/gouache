# Legal notes for Phase 0.4 / 0.5 (not legal advice)

_Recorded 2026-11-16._

## Governing terms
- Tiny Glade (Pounce Light, Steam app 2198150) ships **no game-specific EULA** in the install directory, and the Steam
  store page links only the **Steam Subscriber Agreement** (SSA), revision date 2026-09-10.
- SSA §2.G: "Except as otherwise permitted under this Agreement … or under applicable law notwithstanding these
  restrictions, you may not … reverse engineer, derive source code from, modify, disassemble, decompile, create
  derivative works based on … the Content and Services or any software accessed via Steam without the prior consent,
  in writing, of Valve."
- EU Software Directive 2009/24/EC: Art. 5(3) allows observing, studying and testing the program's functioning while
  loading/running/displaying it (black-box observation, i.e. Phase 0.4). Art. 6 allows decompilation only for
  interoperability of an independently created program — not for studying design to build a similar product.

## Decision (Implementer session)
- **Allowed and done:** reading plaintext files the game publishes in its install (`credits-crates.txt`, `build-info/`,
  `third-party-licenses.html`, `default_settings.ron`) and listing directory/file names and sizes. This is not
  disassembly or decompilation. Done by a separate Analyst agent writing prose only to `docs/STUDY/CLEANROOM/`.
- **Not done:** disassembling, decompiling, or string-dumping the executable; opening `compiled-shaders/` or
  `compiled-assets/` contents; GPU frame capture. These fall under SSA §2.G's prohibition and outside the Art. 6
  exception. The user may revisit this with legal advice; the spec's clean-room procedure remains the only allowed
  way if they do.
- **Phase 0.4 (black-box observation while playing)** is consistent with Art. 5(3); it needs screen capture of the
  running game (not available in this session).
- Nothing from the game's files is copied into the repo beyond short factual notes (names, versions, counts).
