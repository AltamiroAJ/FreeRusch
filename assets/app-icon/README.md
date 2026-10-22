# App icon — required assets (not yet supplied)

The Craft family release pipeline packages the icon per target platform. The
artwork itself is a release prerequisite owned by the maintainer; this directory
documents the exact file set so packaging can be wired without guessing.

| Path | Used by |
|---|---|
| `freerusch.svg` | source artwork, Linux scalable, documentation |
| `freerusch-1024.png` | macOS `.icns` generation, store listings |
| `freerusch.icns` | macOS `.app` bundle |
| `freerusch.ico` | Windows executable / MSI |
| `hicolor/<size>x<size>/apps/<bundle-id>.png` | Linux desktop entry and AppStream |

Sizes for `hicolor/`: 16, 24, 32, 48, 64, 128, 256, 512, plus `scalable`.

## Bundle identifier — open decision

The bundle id is **not fixed yet** and must be decided before the first signed
release, because changing it later breaks upgrades:

- `ai.storyteller.freerusch` — matches the family convention, but that namespace
  belongs to the ArtCraft organisation.
- a domain the maintainer actually controls, if FreeRusch ships independently.

Do not hardcode either value in packaging until it is confirmed.

## Reference

See `storytold/gridcraft` → `assets/app-icon/` for the layout the shared
packaging scripts expect.
