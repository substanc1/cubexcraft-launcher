# Third-party notices / Сторонние компоненты

CubeXCraft Launcher использует сторонние компоненты. Каждый распространяется по своей лицензии; лицензия
CubeXCraft (файл `LICENSE`) на них не распространяется и не ограничивает права, которые дают их лицензии.

CubeXCraft Launcher uses the third-party components listed below. Each one is distributed under its own
license; the CubeXCraft license (`LICENSE`) does not apply to them and does not restrict their licenses.

## Fonts / Шрифты

Bundled unmodified in `src/assets/fonts/`, licensed under the SIL Open Font License 1.1 (full text:
`src/assets/fonts/OFL.txt`, https://openfontlicense.org).

| Font | Copyright |
|---|---|
| JetBrains Mono | Copyright 2020 The JetBrains Mono Project Authors (https://github.com/JetBrains/JetBrainsMono) |
| Manrope | Copyright 2019 The Manrope Project Authors (https://github.com/sharanda/manrope) |
| Unbounded | Copyright 2022 The Unbounded Project Authors (https://github.com/googlefonts/unbounded) |

## JavaScript

Bundled into the interface at build time:

| Package | License |
|---|---|
| @tauri-apps/api | Apache-2.0 OR MIT |
| svelte (runtime) | MIT |

## Rust crates

The launcher core is built with the crates pinned in `src-tauri/Cargo.lock` (all target platforms
together). Their license texts and copyright notices ship with each crate's source on https://crates.io.
For crates offered under several licenses, CubeXCraft uses them under a permissive option (MIT or
Apache-2.0 where available). The MPL-2.0 crates (cssparser, cssparser-macros, dtoa-short, option-ext,
selectors) are used unmodified; their source code is available on https://crates.io.

### MIT OR Apache-2.0 (222)

aead, aes, android_system_properties, anyhow, arbitrary, async-broadcast, async-compression, async-recursion, async-trait, base64, bitflags, block-buffer, bs58, bumpalo, camino, cargo-platform, cc, cfg-expr, cfg-if, chrono, cipher, compression-codecs, compression-core, cookie, core-foundation, core-foundation-sys, core-graphics, core-graphics-types, cpufeatures, crc32fast, crossbeam-channel, crossbeam-utils, crypto-common, ctr, defmt, defmt-macros, defmt-parser, deranged, derive_arbitrary, digest, dirs, dirs-sys, displaydoc, dtoa, dyn-clone, embed_plist, enumflags2, enumflags2_derive, erased-serde, errno, fdeflate, field-offset, filetime, find-msvc-tools, flate2, foreign-types, foreign-types-macros, foreign-types-shared, form_urlencoded, fs4, futures-channel, futures-core, futures-executor, futures-io, futures-macro, futures-sink, futures-task, futures-util, getrandom, glob, hashbrown, heck, hermit-abi, hex, html5ever, http, httparse, hyper-tls, iana-time-zone, iana-time-zone-haiku, ident_case, idna, inout, ipnet, itoa, jni, jni-sys, jni-sys-macros, js-sys, json-patch, jsonptr, keyboard-types, libc, lock_api, log, markup5ever, mime, native-tls, ndk, ndk-context, ndk-sys, num-conv, num-traits, once_cell, opaque-debug, openssl-macros, openssl-probe, ordered-stream, parking_lot, parking_lot_core, percent-encoding, piper, pkg-config, png, powerfmt, ppv-lite86, proc-macro-crate, proc-macro-error, proc-macro-error-attr, proc-macro2, quote, rand, rand_chacha, rand_core, ref-cast, ref-cast-impl, regex, regex-automata, regex-syntax, reqwest, rustc_version, rustls-pki-types, rustversion, scopeguard, security-framework, security-framework-sys, semver, serde, serde-untagged, serde_core, serde_derive, serde_derive_internals, serde_json, serde_repr, serde_spanned, serde_urlencoded, serde_with, serde_with_macros, serialize-to-javascript, serialize-to-javascript-impl, servo_arc, sha1, sha2, shlex, signal-hook-registry, siphasher, smallvec, socket2, softbuffer, stable_deref_trait, string_cache, string_cache_codegen, swift-rs, syn, system-deps, tao-macros, tar, tempfile, tendril, thiserror, thiserror-impl, time, time-core, time-macros, toml, toml_datetime, toml_edit, toml_parser, toml_writer, tray-icon, typeid, typenum, unicode-segmentation, universal-hash, url, vcpkg, version_check, wasm-bindgen, wasm-bindgen-futures, wasm-bindgen-macro, wasm-bindgen-macro-support, wasm-bindgen-shared, wasm-streams, web-sys, web-time, web_atoms, winapi, winapi-i686-pc-windows-gnu, winapi-x86_64-pc-windows-gnu, windows, windows-collections, windows-core, windows-future, windows-implement, windows-interface, windows-link, windows-numerics, windows-result, windows-strings, windows-sys, windows-targets, windows-threading, windows-version, windows_aarch64_gnullvm, windows_aarch64_msvc, windows_i686_gnu, windows_i686_gnullvm, windows_i686_msvc, windows_x86_64_gnu, windows_x86_64_gnullvm, windows_x86_64_msvc, xattr

### MIT (114)

atk, atk-sys, block2, bytes, cairo-rs, cairo-sys-rs, cargo_metadata, cfb, combine, darling, darling_core, darling_macro, derive_more, derive_more-impl, dlopen2, dlopen2_derive, dom_query, embed-resource, endi, gdk, gdk-pixbuf, gdk-pixbuf-sys, gdk-sys, gdkwayland-sys, gdkx11, gdkx11-sys, generic-array, gio, gio-sys, glib, glib-macros, glib-sys, gobject-sys, gtk, gtk-sys, gtk3-macros, http-body, http-body-util, hyper, hyper-util, ico, infer, is-docker, is-wsl, javascriptcore-rs, javascriptcore-rs-sys, libredox, memoffset, mio, new_debug_unreachable, objc2, objc2-encode, objc2-foundation, open, openssl-sys, pango, pango-sys, phf, phf_codegen, phf_generator, phf_macros, phf_shared, plist, precomputed-hash, quick-xml, redox_syscall, redox_users, rfd, schannel, schemars, schemars_derive, simd-adler32, slab, soup3, soup3-sys, strsim, synstructure, tauri-winres, tokio, tokio-macros, tokio-native-tls, tokio-util, tower, tower-http, tower-layer, tower-service, tracing, tracing-attributes, tracing-core, try-lock, uds_windows, urlpattern, version-compare, vswhom, vswhom-sys, want, webkit2gtk, webkit2gtk-sys, webview2-com, webview2-com-macros, webview2-com-sys, winnow, winreg, x11, x11-dl, zbus, zbus_macros, zbus_names, zcheapstr, zip, zmij, zvariant, zvariant_derive, zvariant_utils

### Apache-2.0 OR MIT (54)

aes-gcm, async-channel, async-executor, async-io, async-lock, async-process, async-signal, async-task, atomic-waker, autocfg, bit-set, bit-vec, blocking, cargo_toml, cesu8, concurrent-queue, ctor, dbus, equivalent, event-listener, event-listener-strategy, fastrand, fnv, futures-lite, ghash, idna_adapter, indexmap, libappindicator, libappindicator-sys, libdbus-sys, muda, parking, pin-project-lite, polling, polyval, portable-atomic, portable-atomic-util, rustc-hash, tauri, tauri-build, tauri-codegen, tauri-macros, tauri-plugin, tauri-plugin-dialog, tauri-plugin-fs, tauri-plugin-single-instance, tauri-runtime, tauri-runtime-wry, tauri-utils, utf8_iter, uuid, window-vibrancy, wry, zeroize

### Unicode-3.0 (18)

icu_collections, icu_locale_core, icu_normalizer, icu_normalizer_data, icu_properties, icu_properties_data, icu_provider, litemap, potential_utf, tinystr, writeable, yoke, yoke-derive, zerofrom, zerofrom-derive, zerotrie, zerovec, zerovec-derive

### Zlib OR Apache-2.0 OR MIT (17)

bytemuck, dispatch2, objc2-app-kit, objc2-cloud-kit, objc2-core-data, objc2-core-foundation, objc2-core-graphics, objc2-core-image, objc2-core-location, objc2-core-text, objc2-exception-helper, objc2-io-surface, objc2-quartz-core, objc2-ui-kit, objc2-user-notifications, objc2-web-kit, tinyvec

### Unlicense OR MIT (11)

aho-corasick, byteorder, jiff, jiff-core, jiff-static, jiff-tzdb, jiff-tzdb-platform, memchr, same-file, walkdir, winapi-util

### MPL-2.0 (5)

cssparser, cssparser-macros, dtoa-short, option-ext, selectors

### Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT (5)

linux-raw-sys, rustix, wasi, wasip2, wit-bindgen

### Apache-2.0 (4)

openssl, sync_wrapper, tao, zopfli

### BSD-3-Clause (3)

alloc-no-stdlib, alloc-stdlib, subtle

### Zlib (2)

foldhash, zlib-rs

### BSD-3-Clause OR MIT OR Apache-2.0 (2)

num_enum, num_enum_derive

### BSD-2-Clause OR Apache-2.0 OR MIT (2)

zerocopy, zerocopy-derive

### 0BSD OR MIT OR Apache-2.0 (1)

adler2

### BSD-3-Clause AND MIT (1)

brotli

### BSD-3-Clause OR MIT (1)

brotli-decompressor

### Apache-2.0 AND MIT (1)

dpi

### CC0-1.0 OR MIT-0 OR Apache-2.0 (1)

dunce

### ISC (1)

libloading

### MIT OR Zlib OR Apache-2.0 (1)

miniz_oxide

### MIT OR Apache-2.0 OR LGPL-2.1-or-later (1)

r-efi

### MIT OR Apache-2.0 OR Zlib (1)

raw-window-handle

### Apache-2.0 OR BSL-1.0 (1)

ryu

### Apache-2.0 WITH LLVM-exception (1)

target-lexicon

### (MIT OR Apache-2.0) AND Unicode-3.0 (1)

unicode-ident

