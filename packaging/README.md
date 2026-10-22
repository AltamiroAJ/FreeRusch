# packaging

Scripts and metadata consumed by the Craft family release pipeline
(`.github/workflows/release.yml` → `storytold/craft-common-actions`).

## Present

| File | Purpose |
|---|---|
| `env.sh` | Shared setup: `ROOT`, `VERSION` (from `[workspace.package]`), `DIST`, build SHA/date, `warn`, `copy_docs`, `sha256` |

## Still required before the first signed release

The shared workflow expects per-platform packaging inputs. None of these exist
yet, which is why the `release` branch must not be pushed:

| Path | Needed for |
|---|---|
| `macos/Info.plist` (templated with bundle id) | macOS `.app`, DMG, notarization |
| `windows/*.wxs`, `*.wxl`, WiX helper script | MSI (x64 / x86 / ARM64) |
| `linux/*.desktop`, AppStream metainfo, AppImage entrypoint | AppImage, deb, rpm, tar.gz |
| `freebsd/package.sh` | FreeBSD tarball, when `run-package-check` is used |
| `icons.sh` | regenerating platform icon sets from `assets/app-icon/` |

Two inputs are owned by the maintainer and cannot be generated here:

1. **App icon artwork** — see [`assets/app-icon/README.md`](../assets/app-icon/README.md).
2. **Signing secrets** — Apple Developer ID + notarization credentials and Azure
   Trusted Signing, configured in the repository's `release` environment.

The **bundle identifier** is also still open; changing it after the first
release breaks upgrades, so it should be decided before packaging lands.

## Verifying `env.sh`

```sh
bash -c 'source packaging/env.sh && echo "$VERSION"'
```

It must print the same value as `[workspace.package] version` in the root
`Cargo.toml`; the workflow's `resolve-version` action rejects anything that is
not valid semver.
