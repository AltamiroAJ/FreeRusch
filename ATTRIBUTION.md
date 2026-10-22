# Attribution

FreeRusch is licensed under the MIT License **or** the Apache License, Version
2.0, at your option. See `LICENSE-MIT` and `LICENSE-APACHE`.

This file records third-party material bundled with or required by FreeRusch.
The dependency list below is generated from `cargo metadata`; regenerate it with
`tools/generate_attribution.py` whenever `Cargo.lock` changes.

## Embedded Rüsch coefficient tables

The tabulated coefficients under `crates/freerusch-core/data/` are
engineering data transcribed from the DIN/IBSW Rüsch tables as compiled in the
academic workbook **Tabelas Rusch Final - IME.xls** (SHA-256 `4847235DB64434E1D86E49A5C45624180D3264EAEB8DB56EBE378651E1CB676E`), through
`tools/extract_rusch_tables.py`.

- The workbook itself is **not** redistributed with this repository.
- The extraction is mechanical: values are copied verbatim, `-` cells become
  "absent", and no coefficient is interpolated or invented at extraction time.
- Coverage is partial: only a subset of the workbook's tables is embedded so
  far. See `docs/ROADMAP.md` (item R1).

## Normative references

NBR 7188:2013 (road traffic loads on bridges) and NBR 6118:2023 (design of
concrete structures) are cited as engineering references. Their text is not
distributed with this software and no excerpt is reproduced in the source tree.

## Runtime dependencies

466 packages outside this workspace, grouped by the licence declared in their package metadata.


### (Apache-2.0 OR MIT) AND BSD-3-Clause

| Package | Version | Source |
| --- | --- | --- |
| `encoding_rs` | 0.8.42 | registry |

### (MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0

| Package | Version | Source |
| --- | --- | --- |
| `epaint_default_fonts` | 0.36.2 | registry |

### (MIT OR Apache-2.0) AND Unicode-3.0

| Package | Version | Source |
| --- | --- | --- |
| `unicode-ident` | 1.0.27 | registry |

### 0BSD OR MIT OR Apache-2.0

| Package | Version | Source |
| --- | --- | --- |
| `adler2` | 2.0.1 | registry |

### Apache-2.0

| Package | Version | Source |
| --- | --- | --- |
| `codespan-reporting` | 0.13.1 | registry |
| `flagset` | 0.4.7 | registry |
| `gethostname` | 1.1.0 | registry |
| `gl_generator` | 0.14.0 | registry |
| `glutin` | 0.32.3 | registry |
| `glutin_egl_sys` | 0.7.1 | registry |
| `glutin_glx_sys` | 0.6.1 | registry |
| `glutin_wgl_sys` | 0.6.1 | registry |
| `khronos_api` | 3.1.0 | registry |
| `sigstore-bundle` | 0.6.6 | registry |
| `sigstore-crypto` | 0.6.6 | registry |
| `sigstore-merkle` | 0.6.6 | registry |
| `sigstore-rekor` | 0.6.6 | registry |
| `sigstore-trust-root` | 0.6.6 | registry |
| `sigstore-tsa` | 0.6.6 | registry |
| `sigstore-types` | 0.6.6 | registry |
| `sigstore-verify` | 0.6.6 | registry |
| `sync_wrapper` | 1.0.2 | registry |
| `unicode-general-category` | 1.1.0 | registry |
| `winit` | 0.30.13 | registry |

### Apache-2.0 / MIT

| Package | Version | Source |
| --- | --- | --- |
| `fnv` | 1.0.7 | registry |

### Apache-2.0 AND ISC

| Package | Version | Source |
| --- | --- | --- |
| `ring` | 0.17.14 | registry |

### Apache-2.0 AND MIT

| Package | Version | Source |
| --- | --- | --- |
| `dpi` | 0.1.2 | registry |

### Apache-2.0 OR BSL-1.0

| Package | Version | Source |
| --- | --- | --- |
| `ryu` | 1.0.23 | registry |
| `ryu-js` | 1.0.3 | registry |

### Apache-2.0 OR GPL-2.0-only

| Package | Version | Source |
| --- | --- | --- |
| `self_cell` | 1.3.0 | registry |

### Apache-2.0 OR ISC OR MIT

| Package | Version | Source |
| --- | --- | --- |
| `hyper-rustls` | 0.27.10 | registry |
| `rustls` | 0.23.46 | registry |
| `rustls-native-certs` | 0.8.5 | registry |

### Apache-2.0 OR MIT

| Package | Version | Source |
| --- | --- | --- |
| `atomic-waker` | 1.1.2 | registry |
| `autocfg` | 1.5.1 | registry |
| `base64ct` | 1.8.3 | registry |
| `bit-set` | 0.10.0 | registry |
| `bit-vec` | 0.9.1 | registry |
| `cmpv2` | 0.2.0 | registry |
| `cms` | 0.2.3 | registry |
| `color` | 0.3.3 | registry |
| `concurrent-queue` | 2.5.0 | registry |
| `const-oid` | 0.9.6 | registry |
| `crmf` | 0.2.0 | registry |
| `der` | 0.7.10 | registry |
| `der_derive` | 0.7.3 | registry |
| `equivalent` | 1.0.3 | registry |
| `fastrand` | 2.5.0 | registry |
| `fearless_simd` | 0.4.1 | registry |
| `glifo` | 0.2.0 | registry |
| `idna_adapter` | 1.2.2 | registry |
| `indexmap` | 2.14.2 | registry |
| `kurbo` | 0.13.1 | registry |
| `linebender_resource_handle` | 0.1.1 | registry |
| `multiversion_no_op` | 1.0.0 | registry |
| `nohash-hasher` | 0.2.0 | registry |
| `pem-rfc7468` | 0.7.0 | registry |
| `peniko` | 0.6.1 | registry |
| `pin-project` | 1.1.13 | registry |
| `pin-project-internal` | 1.1.13 | registry |
| `pin-project-lite` | 0.2.17 | registry |
| `polling` | 3.11.0 | registry |
| `portable-atomic` | 1.15.0 | registry |
| `portable-atomic-util` | 0.2.8 | registry |
| `rustc-hash` | 2.1.3 | registry |
| `signature` | 2.2.0 | registry |
| `simd_cesu8` | 1.2.0 | registry |
| `spki` | 0.7.3 | registry |
| `tls_codec` | 0.4.2 | registry |
| `tls_codec_derive` | 0.4.2 | registry |
| `utf8_iter` | 1.0.4 | registry |
| `uuid` | 1.28.0 | registry |
| `vello_common` | 0.1.0 | registry |
| `vello_cpu` | 0.1.0 | registry |
| `x509-cert` | 0.2.5 | registry |
| `x509-tsp` | 0.1.0 | registry |
| `zeroize` | 1.9.1 | registry |
| `zeroize_derive` | 1.5.0 | registry |

### Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT

| Package | Version | Source |
| --- | --- | --- |
| `linux-raw-sys` | 0.12.1 | registry |
| `linux-raw-sys` | 0.4.15 | registry |
| `rustix` | 0.38.44 | registry |
| `rustix` | 1.1.5 | registry |
| `wasi` | 0.11.1+wasi-snapshot-preview1 | registry |
| `wasip2` | 1.0.4+wasi-0.2.12 | registry |
| `wit-bindgen` | 0.57.1 | registry |

### Apache-2.0/MIT

| Package | Version | Source |
| --- | --- | --- |
| `rustc-hash` | 1.1.0 | registry |

### BSD-2-Clause OR Apache-2.0 OR MIT

| Package | Version | Source |
| --- | --- | --- |
| `zerocopy` | 0.8.63 | registry |
| `zerocopy-derive` | 0.8.63 | registry |

### BSD-3-Clause

| Package | Version | Source |
| --- | --- | --- |
| `subtle` | 2.6.1 | registry |
| `winprint` | 0.2.1 | git |

### BSD-3-Clause OR Apache-2.0

| Package | Version | Source |
| --- | --- | --- |
| `moxcms` | 0.8.1 | registry |
| `pxfm` | 0.1.30 | registry |

### BSD-3-Clause OR MIT OR Apache-2.0

| Package | Version | Source |
| --- | --- | --- |
| `num_enum` | 0.7.6 | registry |
| `num_enum_derive` | 0.7.6 | registry |

### BSL-1.0

| Package | Version | Source |
| --- | --- | --- |
| `clipboard-win` | 5.4.1 | registry |
| `error-code` | 3.4.0 | registry |

### CC0-1.0 OR MIT-0 OR Apache-2.0

| Package | Version | Source |
| --- | --- | --- |
| `dunce` | 1.0.5 | registry |

### CDLA-Permissive-2.0

| Package | Version | Source |
| --- | --- | --- |
| `webpki-root-certs` | 1.0.10 | registry |

### ISC

| Package | Version | Source |
| --- | --- | --- |
| `libloading` | 0.8.9 | registry |
| `rustls-webpki` | 0.103.16 | registry |
| `untrusted` | 0.7.1 | registry |
| `untrusted` | 0.9.0 | registry |

### ISC AND (Apache-2.0 OR ISC)

| Package | Version | Source |
| --- | --- | --- |
| `aws-lc-rs` | 1.18.2 | registry |

### ISC AND (Apache-2.0 OR ISC) AND Apache-2.0 AND MIT AND BSD-3-Clause AND (Apache-2.0 OR ISC OR MIT) AND (Apache-2.0 OR ISC OR MIT-0)

| Package | Version | Source |
| --- | --- | --- |
| `aws-lc-sys` | 0.46.0 | registry |

### MIT

| Package | Version | Source |
| --- | --- | --- |
| `android-properties` | 0.2.2 | registry |
| `block2` | 0.5.1 | registry |
| `block2` | 0.6.2 | registry |
| `bytes` | 1.12.1 | registry |
| `calloop` | 0.13.0 | registry |
| `calloop` | 0.14.5 | registry |
| `calloop-wayland-source` | 0.3.0 | registry |
| `calloop-wayland-source` | 0.4.1 | registry |
| `cfg_aliases` | 0.2.2 | registry |
| `combine` | 4.6.9 | registry |
| `crunchy` | 0.2.4 | registry |
| `dispatch` | 0.2.0 | registry |
| `dlib` | 0.5.3 | registry |
| `fax` | 0.2.7 | registry |
| `fs_extra` | 1.3.0 | registry |
| `generic-array` | 0.14.7 | registry |
| `glutin-winit` | 0.5.0 | registry |
| `h2` | 0.4.20 | registry |
| `harfrust` | 0.12.0 | registry |
| `http-body` | 1.1.0 | registry |
| `http-body-util` | 0.1.5 | registry |
| `hyper` | 1.12.0 | registry |
| `hyper-util` | 0.1.21 | registry |
| `libm` | 0.2.16 | registry |
| `libredox` | 0.1.25 | registry |
| `mio` | 1.2.4 | registry |
| `objc-sys` | 0.3.5 | registry |
| `objc2` | 0.5.3 | registry |
| `objc2` | 0.6.5 | registry |
| `objc2-app-kit` | 0.2.2 | registry |
| `objc2-cloud-kit` | 0.2.2 | registry |
| `objc2-contacts` | 0.2.2 | registry |
| `objc2-core-data` | 0.2.2 | registry |
| `objc2-core-image` | 0.2.2 | registry |
| `objc2-core-location` | 0.2.2 | registry |
| `objc2-encode` | 4.1.0 | registry |
| `objc2-foundation` | 0.2.2 | registry |
| `objc2-foundation` | 0.3.2 | registry |
| `objc2-link-presentation` | 0.2.2 | registry |
| `objc2-metal` | 0.2.2 | registry |
| `objc2-quartz-core` | 0.2.2 | registry |
| `objc2-symbols` | 0.2.2 | registry |
| `objc2-ui-kit` | 0.2.2 | registry |
| `objc2-uniform-type-identifiers` | 0.2.2 | registry |
| `objc2-user-notifications` | 0.2.2 | registry |
| `orbclient` | 0.3.55 | registry |
| `pem` | 3.0.6 | registry |
| `quick-xml` | 0.41.0 | registry |
| `redox_syscall` | 0.4.1 | registry |
| `redox_syscall` | 0.5.18 | registry |
| `redox_syscall` | 0.9.5 | registry |
| `redox_users` | 0.5.3 | registry |
| `schannel` | 0.1.30 | registry |
| `serde_json_canonicalizer` | 0.3.2 | registry |
| `simd-adler32` | 0.3.10 | registry |
| `slab` | 0.4.12 | registry |
| `smithay-client-toolkit` | 0.19.2 | registry |
| `smithay-client-toolkit` | 0.20.0 | registry |
| `smithay-clipboard` | 0.7.3 | registry |
| `strum` | 0.28.0 | registry |
| `strum_macros` | 0.28.0 | registry |
| `synstructure` | 0.14.0 | registry |
| `tiff` | 0.11.3 | registry |
| `tokio` | 1.54.0 | registry |
| `tokio-util` | 0.7.20 | registry |
| `tower` | 0.5.3 | registry |
| `tower-http` | 0.6.11 | registry |
| `tower-layer` | 0.3.3 | registry |
| `tower-service` | 0.3.3 | registry |
| `tracing` | 0.1.44 | registry |
| `tracing-attributes` | 0.1.31 | registry |
| `tracing-core` | 0.1.36 | registry |
| `try-lock` | 0.2.5 | registry |
| `want` | 0.3.2 | registry |
| `wayland-backend` | 0.3.17 | registry |
| `wayland-client` | 0.31.15 | registry |
| `wayland-csd-frame` | 0.3.0 | registry |
| `wayland-cursor` | 0.31.14 | registry |
| `wayland-protocols` | 0.32.13 | registry |
| `wayland-protocols-experimental` | 20250721.0.1 | registry |
| `wayland-protocols-misc` | 0.3.12 | registry |
| `wayland-protocols-plasma` | 0.3.12 | registry |
| `wayland-protocols-wlr` | 0.3.12 | registry |
| `wayland-scanner` | 0.31.11 | registry |
| `wayland-sys` | 0.31.11 | registry |
| `winnow` | 1.0.4 | registry |
| `winsafe` | 0.0.29 | registry |
| `x11-dl` | 2.21.0 | registry |
| `xcursor` | 0.3.11 | registry |
| `xkbcommon-dl` | 0.4.2 | registry |
| `xml` | 1.4.0 | registry |
| `xml-rs` | 0.8.29 | registry |
| `zmij` | 1.0.23 | registry |

### MIT / Apache-2.0

| Package | Version | Source |
| --- | --- | --- |
| `cgl` | 0.3.2 | registry |

### MIT OR Apache-2.0

| Package | Version | Source |
| --- | --- | --- |
| `accesskit` | 0.24.1 | registry |
| `ahash` | 0.8.12 | registry |
| `android-activity` | 0.6.1 | registry |
| `android_system_properties` | 0.1.6 | registry |
| `arboard` | 3.7.0 | registry |
| `arrayvec` | 0.7.8 | registry |
| `as-raw-xcb-connection` | 1.0.1 | registry |
| `base64` | 0.22.1 | registry |
| `base64` | 0.23.1 | registry |
| `bitflags` | 2.13.2 | registry |
| `block-buffer` | 0.10.4 | registry |
| `bumpalo` | 3.20.3 | registry |
| `cc` | 1.9.0 | registry |
| `cfg-if` | 1.0.5 | registry |
| `chacha20` | 0.10.2 | registry |
| `chrono` | 0.4.45 | registry |
| `cmake` | 0.1.58 | registry |
| `core-foundation` | 0.10.1 | registry |
| `core-foundation` | 0.9.4 | registry |
| `core-foundation-sys` | 0.8.7 | registry |
| `core-graphics` | 0.23.2 | registry |
| `core-graphics-types` | 0.1.3 | registry |
| `cpufeatures` | 0.2.17 | registry |
| `cpufeatures` | 0.3.1 | registry |
| `craft-atomic` | 0.1.0 | git |
| `craft-print` | 0.1.0 | git |
| `craft-settings` | 0.1.0 | git |
| `craft-ui` | 0.1.0 | git |
| `craft-winprint-gdi` | 0.1.0 | git |
| `crc32fast` | 1.5.3 | registry |
| `crossbeam-utils` | 0.8.23 | registry |
| `crypto-common` | 0.1.7 | registry |
| `digest` | 0.10.7 | registry |
| `dirs` | 7.0.0 | registry |
| `dirs-sys` | 0.5.0 | registry |
| `displaydoc` | 0.2.7 | registry |
| `document-features` | 0.2.12 | registry |
| `ecolor` | 0.36.2 | registry |
| `eframe` | 0.36.2 | registry |
| `egui` | 0.36.2 | registry |
| `egui-wgpu` | 0.36.2 | registry |
| `egui-winit` | 0.36.2 | registry |
| `egui_glow` | 0.36.2 | registry |
| `either` | 1.19.0 | registry |
| `emath` | 0.36.2 | registry |
| `epaint` | 0.36.2 | registry |
| `errno` | 0.3.14 | registry |
| `euclid` | 0.22.14 | registry |
| `fdeflate` | 0.3.7 | registry |
| `find-msvc-tools` | 0.1.15 | registry |
| `flate2` | 1.1.10 | registry |
| `fmt-derive` | 0.1.2 | registry |
| `fmt-derive-proc` | 0.1.2 | registry |
| `font-types` | 0.12.7 | registry |
| `form_urlencoded` | 1.2.2 | registry |
| `futures-channel` | 0.3.34 | registry |
| `futures-core` | 0.3.34 | registry |
| `futures-io` | 0.3.34 | registry |
| `futures-sink` | 0.3.34 | registry |
| `futures-task` | 0.3.34 | registry |
| `futures-util` | 0.3.34 | registry |
| `getrandom` | 0.2.17 | registry |
| `getrandom` | 0.3.4 | registry |
| `getrandom` | 0.4.3 | registry |
| `half` | 2.7.1 | registry |
| `hashbrown` | 0.17.1 | registry |
| `heck` | 0.5.0 | registry |
| `hermit-abi` | 0.5.3 | registry |
| `hex` | 0.4.3 | registry |
| `http` | 1.5.0 | registry |
| `httparse` | 1.10.1 | registry |
| `iana-time-zone` | 0.1.65 | registry |
| `iana-time-zone-haiku` | 0.1.2 | registry |
| `idna` | 1.1.0 | registry |
| `image` | 0.25.10 | registry |
| `ipnet` | 2.12.2 | registry |
| `itertools` | 0.15.0 | registry |
| `itoa` | 1.0.18 | registry |
| `jni` | 0.22.4 | registry |
| `jni-macros` | 0.22.4 | registry |
| `jni-sys` | 0.3.1 | registry |
| `jni-sys` | 0.4.1 | registry |
| `jni-sys-macros` | 0.4.1 | registry |
| `jobserver` | 0.1.35 | registry |
| `js-sys` | 0.3.106 | registry |
| `libc` | 0.2.191 | registry |
| `litrs` | 1.0.1 | registry |
| `lock_api` | 0.4.14 | registry |
| `log` | 0.4.34 | registry |
| `memmap2` | 0.9.11 | registry |
| `mime` | 0.3.17 | registry |
| `naga` | 30.0.1 | registry |
| `naga-types` | 30.0.1 | registry |
| `ndk` | 0.9.0 | registry |
| `ndk-context` | 0.1.1 | registry |
| `ndk-sys` | 0.6.0+11769913 | registry |
| `num-traits` | 0.2.19 | registry |
| `once_cell` | 1.21.4 | registry |
| `openssl-probe` | 0.2.1 | registry |
| `parking_lot` | 0.12.5 | registry |
| `parking_lot_core` | 0.9.12 | registry |
| `percent-encoding` | 2.3.2 | registry |
| `pkg-config` | 0.3.34 | registry |
| `png` | 0.18.1 | registry |
| `polycool` | 0.4.0 | registry |
| `proc-macro-crate` | 3.5.0 | registry |
| `proc-macro-error` | 1.0.4 | registry |
| `proc-macro-error-attr` | 1.0.4 | registry |
| `proc-macro2` | 1.0.107 | registry |
| `profiling` | 1.0.18 | registry |
| `quinn` | 0.11.12 | registry |
| `quinn-proto` | 0.11.19 | registry |
| `quinn-udp` | 0.5.16 | registry |
| `quote` | 1.0.47 | registry |
| `rand` | 0.10.3 | registry |
| `rand_core` | 0.10.1 | registry |
| `rand_core` | 0.6.4 | registry |
| `rand_pcg` | 0.10.2 | registry |
| `read-fonts` | 0.41.0 | registry |
| `renderdoc-sys` | 1.1.0 | registry |
| `reqwest` | 0.13.5 | registry |
| `roxmltree` | 0.20.0 | registry |
| `rustc_version` | 0.4.1 | registry |
| `rustls-pki-types` | 1.15.1 | registry |
| `rustls-platform-verifier` | 0.7.1 | registry |
| `rustls-platform-verifier-android` | 0.2.0 | registry |
| `rustversion` | 1.0.23 | registry |
| `scopeguard` | 1.2.0 | registry |
| `security-framework` | 3.7.0 | registry |
| `security-framework-sys` | 2.17.0 | registry |
| `semver` | 1.0.28 | registry |
| `serde` | 1.0.229 | registry |
| `serde_core` | 1.0.229 | registry |
| `serde_derive` | 1.0.229 | registry |
| `serde_json` | 1.0.154 | registry |
| `serde_spanned` | 1.1.2 | registry |
| `sha1` | 0.10.7 | registry |
| `sha2` | 0.10.9 | registry |
| `shlex` | 2.0.1 | registry |
| `simdutf8` | 0.1.5 | registry |
| `skrifa` | 0.44.0 | registry |
| `smallvec` | 1.16.3 | registry |
| `smol_str` | 0.2.2 | registry |
| `socket2` | 0.6.5 | registry |
| `stable_deref_trait` | 1.2.1 | registry |
| `static_assertions` | 1.1.0 | registry |
| `syn` | 1.0.109 | registry |
| `syn` | 2.0.119 | registry |
| `syn` | 3.0.9 | registry |
| `system-configuration` | 0.7.0 | registry |
| `system-configuration-sys` | 0.6.0 | registry |
| `tar` | 0.4.46 | registry |
| `tempfile` | 3.27.0 | registry |
| `thiserror` | 1.0.69 | registry |
| `thiserror` | 2.0.21 | registry |
| `thiserror-impl` | 1.0.69 | registry |
| `thiserror-impl` | 2.0.21 | registry |
| `tokio-rustls` | 0.26.6 | registry |
| `toml_datetime` | 1.1.2+spec-1.1.0 | registry |
| `toml_edit` | 0.25.17+spec-1.1.0 | registry |
| `toml_parser` | 1.1.5+spec-1.1.0 | registry |
| `toml_writer` | 1.1.3+spec-1.1.0 | registry |
| `typenum` | 1.20.1 | registry |
| `unicode-segmentation` | 1.13.3 | registry |
| `unicode-width` | 0.2.2 | registry |
| `url` | 2.5.8 | registry |
| `wasm-bindgen` | 0.2.129 | registry |
| `wasm-bindgen-futures` | 0.4.79 | registry |
| `wasm-bindgen-macro` | 0.2.129 | registry |
| `wasm-bindgen-macro-support` | 0.2.129 | registry |
| `wasm-bindgen-shared` | 0.2.129 | registry |
| `web-sys` | 0.3.106 | registry |
| `web-time` | 1.1.0 | registry |
| `weezl` | 0.1.12 | registry |
| `wgpu` | 30.0.1 | registry |
| `wgpu-core` | 30.0.1 | registry |
| `wgpu-core-deps-windows-linux-android` | 30.0.1 | registry |
| `wgpu-hal` | 30.0.1 | registry |
| `wgpu-naga-bridge` | 30.0.1 | registry |
| `wgpu-types` | 30.0.1 | registry |
| `windows` | 0.62.2 | registry |
| `windows-collections` | 0.3.2 | registry |
| `windows-core` | 0.62.2 | registry |
| `windows-future` | 0.3.2 | registry |
| `windows-implement` | 0.60.2 | registry |
| `windows-interface` | 0.59.3 | registry |
| `windows-link` | 0.2.1 | registry |
| `windows-numerics` | 0.3.1 | registry |
| `windows-registry` | 0.6.1 | registry |
| `windows-result` | 0.4.1 | registry |
| `windows-strings` | 0.5.1 | registry |
| `windows-sys` | 0.52.0 | registry |
| `windows-sys` | 0.59.0 | registry |
| `windows-sys` | 0.61.2 | registry |
| `windows-targets` | 0.52.6 | registry |
| `windows-threading` | 0.2.1 | registry |
| `windows_aarch64_gnullvm` | 0.52.6 | registry |
| `windows_aarch64_msvc` | 0.52.6 | registry |
| `windows_i686_gnu` | 0.52.6 | registry |
| `windows_i686_gnullvm` | 0.52.6 | registry |
| `windows_i686_msvc` | 0.52.6 | registry |
| `windows_x86_64_gnu` | 0.52.6 | registry |
| `windows_x86_64_gnullvm` | 0.52.6 | registry |
| `windows_x86_64_msvc` | 0.52.6 | registry |
| `x11rb` | 0.13.2 | registry |
| `x11rb-protocol` | 0.13.2 | registry |
| `xattr` | 1.6.1 | registry |

### MIT OR Apache-2.0 OR LGPL-2.1-or-later

| Package | Version | Source |
| --- | --- | --- |
| `r-efi` | 5.3.0 | registry |
| `r-efi` | 6.0.0 | registry |

### MIT OR Apache-2.0 OR Zlib

| Package | Version | Source |
| --- | --- | --- |
| `cursor-icon` | 1.2.0 | registry |
| `glow` | 0.17.0 | registry |
| `lru-slab` | 0.1.3 | registry |
| `raw-window-handle` | 0.6.2 | registry |
| `xkeysym` | 0.2.1 | registry |
| `zune-core` | 0.5.3 | registry |
| `zune-jpeg` | 0.5.15 | registry |

### MIT OR Zlib OR Apache-2.0

| Package | Version | Source |
| --- | --- | --- |
| `miniz_oxide` | 0.8.9 | registry |
| `miniz_oxide` | 0.9.1 | registry |

### MIT/Apache-2.0

| Package | Version | Source |
| --- | --- | --- |
| `bitflags` | 1.3.2 | registry |
| `core_detect` | 1.0.0 | registry |
| `downcast-rs` | 1.2.1 | registry |
| `filetime` | 0.2.29 | registry |
| `foreign-types` | 0.5.0 | registry |
| `foreign-types-macros` | 0.2.4 | registry |
| `foreign-types-shared` | 0.3.1 | registry |
| `guillotiere` | 0.7.0 | registry |
| `plain` | 0.2.3 | registry |
| `quick-error` | 2.0.1 | registry |
| `scoped-tls` | 1.0.1 | registry |
| `serde_urlencoded` | 0.7.1 | registry |
| `type-map` | 0.5.1 | registry |
| `version_check` | 0.9.5 | registry |

### MPL-2.0

| Package | Version | Source |
| --- | --- | --- |
| `option-ext` | 0.2.0 | registry |

### Unicode-3.0

| Package | Version | Source |
| --- | --- | --- |
| `icu_collections` | 2.3.0 | registry |
| `icu_locale_core` | 2.3.0 | registry |
| `icu_normalizer` | 2.3.0 | registry |
| `icu_normalizer_data` | 2.3.0 | registry |
| `icu_properties` | 2.3.0 | registry |
| `icu_properties_data` | 2.3.0 | registry |
| `icu_provider` | 2.3.1 | registry |
| `litemap` | 0.8.3 | registry |
| `potential_utf` | 0.1.6 | registry |
| `tinystr` | 0.8.4 | registry |
| `writeable` | 0.6.4 | registry |
| `yoke` | 0.8.3 | registry |
| `yoke-derive` | 0.8.4 | registry |
| `zerofrom` | 0.1.8 | registry |
| `zerofrom-derive` | 0.1.8 | registry |
| `zerotrie` | 0.2.5 | registry |
| `zerovec` | 0.11.8 | registry |
| `zerovec-derive` | 0.11.6 | registry |

### Unlicense OR MIT

| Package | Version | Source |
| --- | --- | --- |
| `byteorder-lite` | 0.1.0 | registry |
| `memchr` | 2.8.3 | registry |
| `winapi-util` | 0.1.11 | registry |

### Unlicense/MIT

| Package | Version | Source |
| --- | --- | --- |
| `same-file` | 1.0.6 | registry |
| `walkdir` | 2.5.0 | registry |

### Zlib

| Package | Version | Source |
| --- | --- | --- |
| `foldhash` | 0.2.0 | registry |
| `slotmap` | 1.1.1 | registry |
| `zlib-rs` | 0.6.8 | registry |

### Zlib OR Apache-2.0 OR MIT

| Package | Version | Source |
| --- | --- | --- |
| `bytemuck` | 1.25.2 | registry |
| `bytemuck_derive` | 1.12.1 | registry |
| `dispatch2` | 0.3.1 | registry |
| `objc2-app-kit` | 0.3.2 | registry |
| `objc2-core-foundation` | 0.3.2 | registry |
| `objc2-core-graphics` | 0.3.2 | registry |
| `objc2-io-surface` | 0.3.2 | registry |
| `objc2-ui-kit` | 0.3.2 | registry |
| `tinyvec` | 1.13.3 | registry |

## Build-only dependencies

The Rust toolchain, `cmake` and a C toolchain are required to build: `aws-lc-sys`
arrives transitively through `craft-print` → `winprint` → `sigstore`. See
`AGENTS.md`.

## Notes

- `craft-*` crates come from `storytold/craft-libs`, pinned by git `rev` in the
  root `Cargo.toml`; they are consumed as Apache-2.0 licensed libraries.
- Workspace-local crates (`freerusch-core`, `freerusch-report`,
  `freerusch-project`, `freerusch`) are excluded: they are this project.

