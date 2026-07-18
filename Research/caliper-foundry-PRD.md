<!--
SPDX-License-Identifier: GPL-3.0-or-later
Copyright (c) 2026 Mohamed Hammad
Steelbore Theme: Void Navy (#000027) bg, Molten Amber (#D98E32) body,
Steel Blue (#4B7EB0) H1, Radium Green (#50FA7B) H2, Liquid Coolant (#8BE9FD) H3
Fonts: Share Tech Mono (headings), Inconsolata (body)
-->

# CALIPER FOUNDRY

**Product Requirements Document — v1.0**
**Project Steelbore — User-Facing Layer over the Caliper Tracing Engine**

| Field            | Value                                                  |
|------------------|--------------------------------------------------------|
| Product          | Caliper Foundry                                        |
| Document Version | 1.0                                                    |
| Document Date    | 2026-05-04T00:00:00Z                                   |
| Author           | Mohamed Hammad                                         |
| License          | GPL-3.0-or-later                                       |
| Parent Project   | Caliper (`caliper-trace` v2.0 PRD)                     |
| Crates           | `caliper-foundry-{protocol,core,web,desktop,mobile,bot,cabinet,billet}` |
| Standard         | The Steelbore Standard v1.0 + the CLI Standard v1.0.0  |
| Status           | Draft for review                                       |

---

## 1. Executive Summary

Caliper Foundry is the user-facing layer over the Caliper raster-to-vector tracing engine. It brings Caliper's heterogeneous CPU + GPU + NPU pipeline to four front-end shells from a single Rust core: a Progressive Web App, a Tauri-based desktop application, a Flutter mobile application for iOS and Android, and an AI chat-bot adapter that exposes Caliper as a Poe / Discord / Slack / Telegram bot.

A foundry is where finished metalwork is cast — and Caliper Foundry is where Caliper's tracing pipeline reaches the end user. Where Caliper itself is a CLI / TUI / library for power users and AI agents, Foundry is the consumer surface: drop an image in, get an upscaled raster and a clean vector out. Two operations, one workflow, zero subscription, fully local-first per Steelbore Standard §6.

Foundry's core differentiator against the incumbents — Vectorizer.io, Vectorizer.AI, Topaz Labs (Gigapixel/Photo/Video AI), the Poe TopazLabs bot, and the various Logo Maker mobile apps — is the combination of (1) raster-to-vector tracing and (2) AI super-resolution upscaling unified in one workflow, on (3) all four mainstream platforms, with (4) processing performed locally on the user's hardware by default, under (5) GPL-3.0-or-later, with (6) no telemetry, no tracking, and no subscription required.

---

## 2. Project Identity

| Attribute       | Value                                                                  |
|-----------------|------------------------------------------------------------------------|
| Product Name    | Caliper Foundry                                                        |
| Domain          | Image vectorization + AI super-resolution upscaling, multi-platform    |
| Naming Origin   | Foundry — where finished cast forms emerge from the smelting pipeline (Standard §2) |
| Parent Project  | Caliper (raster-to-vector tracing engine, CLI + TUI + library)         |
| License         | GPL-3.0-or-later (SPDX-compliant)                                      |
| MSRV            | Rust 1.82.0 (Edition 2024)                                             |
| Web Stack       | Leptos 0.7+ (or Yew) + wasm-bindgen + wasm-bindgen-rayon, WebGPU       |
| Desktop Stack   | Tauri 2.x + native `caliper-trace` linked as a Rust dependency         |
| Mobile Stack    | Flutter 3.x UI + `caliper-trace` via `flutter_rust_bridge`             |
| Bot Stack       | `axum` HTTP server + `rmcp` MCP server + per-platform thin adapters    |
| Tier-1 Targets  | x86_64-linux-gnu, aarch64-linux-gnu, x86_64-darwin, aarch64-darwin, x86_64-windows-msvc, aarch64-windows, ios, android, wasm32 |

### 2.1 Module Naming (Metallurgical Convention §2)

| Crate                         | Role                                                              |
|-------------------------------|-------------------------------------------------------------------|
| `caliper-foundry-protocol`    | Shared types, JSON envelope, error codes — used by all shells     |
| `caliper-foundry-core`        | Job queue, scheduler, thread pool, cabinet, settings              |
| `caliper-foundry-cabinet`     | Local archive of past jobs, thumbnails, BLAKE3-keyed dedup        |
| `caliper-foundry-billet`      | Shared static assets — palette, fonts, icons, SKILL.md, AGENTS.md |
| `caliper-foundry-web`         | Browser PWA front end (Leptos / Yew)                              |
| `caliper-foundry-desktop`     | Tauri 2.x desktop shell                                           |
| `caliper-foundry-mobile`      | Flutter shell (iOS + Android) with Rust core via FFI              |
| `caliper-foundry-bot`         | HTTP / MCP server + Poe / Discord / Slack / Telegram adapters     |

The umbrella crate is `caliper-foundry`, exposing `pub use` re-exports of the public API. *Cabinet* — where finished pieces are stored and catalogued — and *Billet* — raw, semi-finished metal stock used as input to subsequent processing — round out the metallurgical naming for the platform-agnostic concerns.

---

## 3. Problem Statement

The current image-vectorize / image-upscale market is fragmented across single-platform, cloud-only, subscription-locked tools. A user who wants both operations on both desktop and mobile must subscribe to multiple services, upload private images to multiple clouds, and integrate the workflows manually. None of the incumbent tools is open source.

| Tool                              | Strengths                                                | Critical Deficiencies                                                                       |
|-----------------------------------|----------------------------------------------------------|---------------------------------------------------------------------------------------------|
| Vectorizer.io                     | Cheap entry; SVG/EPS/PDF outputs                         | Cloud SaaS; aged tracing tech; credit-based; no upscaling; no API for free tier             |
| Vectorizer.AI                     | Best-in-class proprietary tracing (15+ years), deep learning, mirror + rotational symmetry, sub-pixel precision, full API | Cloud SaaS; subscription required to download; closed source; no upscaling; web-only        |
| Topaz Labs (Gigapixel/Photo/Video) | Industry-leading upscaler; nine specialized AI models; up to 6× single pass; local GPU + multi-GPU (up to 10 cards in Pro) | Subscription-first since 2025-10 ($12–$42/mo); no vectorization; closed source; desktop-only |
| Poe TopazLabs Bot                 | Chat-native UX; @-mention multi-bot workflows; up to 16× upscale | Wraps proprietary backend; cloud only; no vectorization; per-message Poe credits             |
| Logo Maker (Android)              | 10 000+ templates; mobile UX; layered designs            | Adjacent product (template-based logo design, not raster→vector); ad-laden; mobile-only      |

Foundry closes this gap by unifying tracing + upscaling + multi-platform delivery in a single open-source product, all of which run **locally by default** on whatever compute the user already owns.

---

## 4. Design Philosophy

**Two operations, one canvas.** Every user flow has the same shape: drop an image, see a side-by-side preview, choose Vectorize or Upscale (or both, in sequence), tweak parameters, export. Tracing and upscaling share the same canvas, the same history, and the same settings.

**Local first, always.** Per Steelbore §6, no image leaves the device by default. Cloud rendering is opt-in *per job*, the cloud endpoint is user-controllable (self-host or use the operator-run Foundry endpoint), and the wire format is the same JSON envelope used internally — there is no separate "cloud-only" flow with extra surface area to attack.

**One Rust core, four shells.** The Caliper engine, Foundry's job management, settings, history, and cabinet all live in pure Rust. The four front ends — web, desktop, mobile, bot — are thin shells that bind to the same `caliper-foundry-core` crate.

**Zero-subscription floor.** The core product is free under GPL-3.0-or-later. Source-built users get every feature, forever. App-store builds may charge a one-time purchase to cover review and platform fees, but never a recurring subscription. Optional managed cloud is metered per render-second.

**Steelbore look and feel everywhere.** Void Navy (`#000027`) background, Molten Amber (`#D98E32`) body, Steel Blue (`#4B7EB0`) H1 / structural, Radium Green (`#50FA7B`) success / H2, Liquid Coolant (`#8BE9FD`) info / links, Red Oxide (`#FF5C5C`) warnings. Share Tech Mono for headings, Inconsolata for body. Material Design components themed with the §8 palette. WCAG 2.1 AA contrast verified on every surface.

**Two co-equal readers.** As with Caliper itself: every operation is exposed via a UI for humans and via JSON over a stable schema for AI agents and pipelines. The bot adapter is not a separate code path — it is the JSON surface with a chat envelope around it.

---

## 5. Architecture Overview

The Caliper engine (`caliper-trace`, with all its inner crates: Crucible, Assayer, Smelt, Etch, Burin, Temper, Weld, Cast, Bellows, Touchstone, Anvil) is unchanged. Foundry adds eight crates above it:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                              CALIPER FOUNDRY                                 │
│                                                                              │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  │
│  │  WEB (PWA)    │  │   DESKTOP     │  │   MOBILE      │  │     BOT       │  │
│  │  Leptos/Yew   │  │   Tauri 2.x   │  │   Flutter +   │  │   HTTP +      │  │
│  │  + WASM core  │  │   + WGPU/NPU  │  │   Rust FFI    │  │   MCP server  │  │
│  └───────┬───────┘  └───────┬───────┘  └───────┬───────┘  └───────┬───────┘  │
│          │                  │                  │                  │          │
│          └──────────┬───────┴──────────┬───────┴──────────┬───────┘          │
│                     │                  │                  │                  │
│              ┌──────▼──────────────────▼──────────────────▼──────┐           │
│              │  caliper-foundry-core (job mgmt + scheduler)      │           │
│              │   • Job queue (Rayon-aware, priority-aware)       │           │
│              │   • Local Cabinet (history, thumbnails, BLAKE3)   │           │
│              │   • Settings (XDG / OS-native paths)              │           │
│              │   • Cancellation tokens (each job is preemptible) │           │
│              └──────────────────────┬────────────────────────────┘           │
│                                     │                                        │
│              ┌──────────────────────▼────────────────────────────┐           │
│              │      caliper-foundry-protocol (shared types)      │           │
│              │   • Vectorize / Upscale / Combine / Describe      │           │
│              │   • JSON Schema 2020-12 emitted per command       │           │
│              └──────────────────────┬────────────────────────────┘           │
└─────────────────────────────────────┼────────────────────────────────────────┘
                                      │
                     ┌────────────────▼─────────────────┐
                     │   CALIPER ENGINE (caliper-trace) │
                     │   Crucible / Assayer / Smelt /   │
                     │   Etch / Burin / Temper / Weld / │
                     │   Cast / Bellows / Touchstone /  │
                     │   Anvil                          │
                     └──────────────────────────────────┘
```

The protocol crate is the contract between every shell and the core. Adding a new shell (Apple Vision Pro? E-ink reader? Steam Deck overlay?) is a matter of binding a new front end to `caliper-foundry-core` — the engine and protocol stay identical.

---

## 6. Platform Designs

### 6.1 Web — `caliper-foundry-web`

A Progressive Web App built with Leptos 0.7+ (Yew is the fallback if Leptos's `wasm-bindgen-rayon` story is not yet stable for the chosen browser baseline). The entire Caliper engine compiles to WebAssembly via `wasm-bindgen` + `wasm-bindgen-rayon` for client-side multi-threaded execution. WebGPU (via wgpu's `wasm32-unknown-unknown` target) accelerates Stages 1–3 of the pipeline when the browser supports it.

Constraints unique to the web target:

- No NPU acceleration — browser sandbox does not expose Touchstone-style execution providers.
- SIMD limited to WASM SIMD128.
- File access via the File System Access API (Chrome / Edge); download fallback elsewhere.
- Service worker caches the WASM bundle and any optional ML models for offline use.

UI components:

- Drop zone with paste-from-clipboard (`Ctrl+V`) and URL-paste support.
- Side-by-side raster ↔ vector preview with magnifier and pixel-level zoom.
- Parameter panel (palette size, edge style, simplification tolerance, primitives toggle, symmetry toggle).
- "Compute on" selector: CPU, WebGPU, or Cloud (opt-in).
- Export modal — SVG, PDF, EPS, DXF for vector; PNG, JPEG, WebP for upscaled raster.
- Cabinet strip across the bottom showing the last 20 jobs, with thumbnails.

Shipped as a static asset bundle deployable to any CDN. The reference deployment lives at `foundry.caliper.dev` and has no telemetry, no analytics, no third-party scripts, and a strict CSP (`script-src 'self' 'wasm-unsafe-eval'`).

### 6.2 Desktop — `caliper-foundry-desktop`

Tauri 2.x with the Caliper engine linked directly as a Rust dependency. Native GPU access via `wgpu`; native NPU access via `ort` (Touchstone) with full execution-provider support: TensorRT, CUDA, OpenVINO, DirectML, CoreML, QNN, Vitis AI.

Why Tauri (not Electron):

- Memory safety priority (§3.1) — Tauri's shell is Rust.
- Smaller bundles (~10 MB vs ~150 MB for Electron) with no Chromium runtime to ship.
- Native webview (WebView2 / WKWebView / WebKitGTK) per platform.
- Typed IPC channels rather than `postMessage` spaghetti.

UI components:

- Multi-document interface — multiple images open at once, tabbed.
- Full editor: layer panel, palette editor, path-edit tools (corner toggle, bezier handle drag).
- Batch mode: drag a folder; trace or upscale every file inside.
- Hardware monitor strip (CPU%, GPU%, NPU%, RAM, active SIMD level, current Assayer recommendation).
- Dual CUA + Vim keybindings per Standard §7. The same keys as Caliper's TUI (`--format explore`).
- Built-in MCP server toggle for agent integration — flip a switch and the desktop app exposes itself on `localhost:7421` as an MCP server, identical to `caliper mcp`.

### 6.3 Mobile — `caliper-foundry-mobile`

Flutter 3.x for the UI layer (cross-platform widget set with Material Design — required by §10). The Rust core is brought in via `flutter_rust_bridge` for typed FFI without manual marshalling.

Why Flutter (not native Swift / Kotlin separately):

- Single source per Standard §3 — one codebase, two app stores.
- Mawaqit precedent: the Steelbore project Mawaqit already ships Flutter UI + Rust CLI + `libmawaqit` together. The same architecture works for Foundry.
- Material Design out of the box for §10 compliance.
- Both stores from one repo.

Mobile-specific concerns:

- iOS: CoreML execution provider for the NPU; Metal for GPU.
- Android: NNAPI / vendor delegate (QNN for Snapdragon Hexagon, EdgeTPU for Tensor) for NPU; Vulkan for GPU.
- Image picker: photo library, camera, document picker, share extension.
- Local storage at the platform-default app sandbox; no external permissions requested unless the user picks a location outside the sandbox.
- Battery awareness: pause queue when battery <15% unless the device is charging.
- App Store + Play Store: PFA compliance — no IDFA, no tracking, no analytics, no third-party SDKs beyond the FFI bridge.

### 6.4 Bot — `caliper-foundry-bot`

HTTP server (`axum`) plus an MCP server (`rmcp`, the same crate Caliper uses for `caliper mcp`). Wraps three operations:

- `vectorize` — accepts an image (URL, base64, or HTTP upload); returns SVG / PDF / EPS / DXF plus metrics.
- `upscale` — accepts an image and a factor (2 / 4 / 8); returns the upscaled raster.
- `combine` — runs upscale then vectorize as a single chained job.

Adapters (each is a thin glue crate behind a feature flag):

- `--features poe` — Poe Server Bot Protocol over HTTP webhook.
- `--features discord` — slash commands + ephemeral file uploads via the Discord API.
- `--features slack` — slash command + `files.upload` reply.
- `--features telegram` — Telegram Bot API HTTP polling or webhook.
- `--features mcp` (default) — generic MCP transport: stdio, SSE, streamable-http.

The bot itself is platform-agnostic; the four chat adapters are thin glue. All adapters share a single inference backend that can run:

- Locally on the bot operator's hardware (default — preserves PFA §6 even when third parties run instances).
- Via a self-hosted Caliper backend at a configured URL.
- Via the optional managed Foundry cloud (metered, opt-in for the bot operator).

---

## 7. UX Flows

### 7.1 Vectorize Flow

1. User drops or selects a raster image.
2. Foundry calls `caliper inspect <path> --json`, receives the Assayer recommendation and an estimated cost.
3. UI renders: *"Recommended: 16-color logo trace, ~0.4 s on your GPU."*
4. User clicks Vectorize, optionally tweaking parameters first.
5. Foundry calls `caliper trace <path> -o <out> --json --accelerator auto`.
6. Streaming progress on stderr → progress bar.
7. Result: side-by-side preview, metrics panel, export menu.

### 7.2 Upscale Flow

1. User drops or selects a raster image.
2. UI offers 2× / 4× / 8× upscale factor + face-aware toggle + denoise slider.
3. Foundry runs the upscale operation (Touchstone NPU pathway via Real-ESRGAN-x4 or DRCT-L INT8 quantized model).
4. Progress bar; result preview; export.

### 7.3 Combined Flow (Foundry's main differentiator)

1. User drops a low-res raster (icon, logo screenshot, scanned sketch).
2. UI suggests *"Upscale to 4× then vectorize"* — a single click chains the two.
3. Foundry runs upscale → trace as one job, with intermediate raster cached and not re-decoded.
4. Result has both: the high-res raster *and* the clean SVG, both downloadable separately or as a `.zip`.

### 7.4 Batch Flow (desktop, mobile, web)

1. User selects multiple files or a folder.
2. Foundry generates a job manifest, submits via `caliper batch --stdin -o vectors/ --format jsonl`.
3. The JSONL stream feeds the progress UI: each line is a row in the batch table.
4. Cabinet shows the batch as a single rolled-up entry with per-file expand.

### 7.5 Bot Flow (chat)

```
User: @caliper-foundry vectorize https://example.com/logo.png

Bot:  📥 Received logo.png (1024×1024, 64 KB)
      🔬 Assayer: logo class, 16 colors, sharp edges
      ⚙️  Tracing on CPU (no GPU available in this region)…
      ✅ Done in 412 ms — 47 shapes, 18.4 KB SVG
      [logo.svg attached]
      ↩ Reply with "tweak colors=8" or "upscale 4x" to refine.
```

---

## 8. API Surface — The Foundry Protocol

The Foundry Protocol is a typed JSON-RPC-shaped surface, identical across all four front ends. Every operation has a `Request`, a `Stream<Progress>`, and a `Result`. JSON Schema (Draft 2020-12) is auto-emitted via `caliper-foundry schema`.

### 8.1 Request Envelope

```json
{
  "version": "1.0",
  "kind": "vectorize",
  "input": {
    "source": "file",
    "value": "/Users/mj/Pictures/logo.png"
  },
  "params": {
    "colors": 16,
    "mode": "spline",
    "primitives": true,
    "symmetry": "auto",
    "output_format": "svg"
  },
  "options": {
    "accelerator": "auto",
    "max_seconds": 30,
    "deterministic": true
  }
}
```

### 8.2 Result Envelope

Mirrors Caliper §9.5 exactly:

```json
{
  "metadata": {
    "tool": "caliper-foundry",
    "version": "1.0.0",
    "command": "foundry vectorize logo.png",
    "timestamp": "2026-05-04T14:30:00Z",
    "assayer": { "image_class": "logo", "stages": { "...": "..." } },
    "hardware": {
      "cpu": { "simd": "avx2" },
      "gpu": { "adapter": "Apple M3 Pro", "backend": "metal" },
      "npu": { "provider": "CoreMLExecutionProvider" }
    }
  },
  "data": {
    "input":  { "path": "logo.png", "width": 1024, "height": 1024, "format": "png" },
    "output": { "path": "logo.svg", "format": "svg", "bytes": 18432 },
    "metrics": {
      "shapes": 47,
      "max_deviation_px": 0.42,
      "elapsed_ms": 412,
      "stages_ms": { "decode": 12, "quantize": 38, "edge": 24, "trace": 41, "fit": 58, "stitch": 8, "cast": 6 }
    }
  }
}
```

### 8.3 Vectorize Params

Identical to the Caliper CLI flag set: `colors`, `mode`, `quantizer`, `corner_threshold`, `simplify_tolerance`, `path_precision`, `output_format`, `primitives`, `symmetry`, `bw`, `tile_size`, `filter_speckle`, `hierarchical`, `edge_detector`.

### 8.4 Upscale Params

```json
{
  "factor": 4,
  "model": "auto",
  "face_aware": true,
  "denoise_strength": 0.0,
  "output_format": "png"
}
```

`model` accepts `auto`, `real-esrgan-x2`, `real-esrgan-x4`, `drct-l-x4`, `swinir-real-x4`. Models are managed via `caliper models pull` and verified with BLAKE3 hashes per Standard §3.3.

### 8.5 Combine Params

```json
{
  "upscale_first": { "factor": 4, "model": "auto" },
  "then_vectorize": { "colors": 16, "mode": "spline", "primitives": true }
}
```

### 8.6 Streaming Progress

Each in-flight job emits a stream of progress events:

```jsonl
{"stage": "decode",   "progress": 1.0, "elapsed_ms": 12}
{"stage": "quantize", "progress": 0.5, "elapsed_ms": 38}
{"stage": "quantize", "progress": 1.0, "elapsed_ms": 76}
{"stage": "edge",     "progress": 1.0, "elapsed_ms": 100}
…
```

Front ends consume this stream over the in-process channel (desktop, mobile), the WebTransport / WebSocket (web), or chunked HTTP (bot).

---

## 9. Pricing and Distribution

### 9.1 Distribution Model

Caliper Foundry ships under GPL-3.0-or-later. Source builds are free forever and have every feature.

| Channel                                     | Pricing                  | Rationale                                          |
|---------------------------------------------|--------------------------|----------------------------------------------------|
| GitHub Releases (web ZIP, desktop binaries) | Free                     | Source is GPL — distribution must be free          |
| Self-hosted bot                             | Free                     | Same                                               |
| crates.io                                   | Free                     | Library users build from source                    |
| `foundry.caliper.dev` web app               | Free                     | Marketing surface; identical PWA to local build    |
| Apple App Store (iOS, iPadOS, macOS)        | $4.99 one-time           | Covers review fee; never a subscription            |
| Google Play Store (Android)                 | $4.99 one-time           | Same                                               |
| Microsoft Store (Windows)                   | $4.99 one-time           | Same                                               |
| Snap / Flatpak / AUR / Homebrew             | Free                     | Linux / macOS canonical FOSS channels              |
| Managed Foundry Cloud (opt-in)              | $0.01 per render-second  | For users without local GPU/NPU; data deleted within 60 s of result delivery |

### 9.2 Why No Subscription

Per Steelbore §6 (Privacy-Friendly Application Policy), an application owns the user's data and storage; subscription pricing creates incentive misalignment with that principle. Source remains GPL — paid channels are a *convenience tax* for store-distributed builds, not a license fee.

The store builds and the source build are bit-identical except for store-injected DRM stubs (which are no-ops on Foundry — the binary still runs without them). A Play Store user can download the APK from GitHub at any time and migrate sideways without losing anything.

---

## 10. Security

Inherits Caliper §12 wholesale: zero `unsafe` blocks outside `caliper-anvil`, position-independent code, CFI, stack canaries, hardened allocator, image-decoder fuzzing, max-dimension enforcement, no network in default mode, supply-chain audit via `cargo-vet` + `cargo-deny`. Additional Foundry-specific provisions:

- **Web** — strict CSP (`script-src 'self' 'wasm-unsafe-eval'`); subresource integrity on the WASM bundle; service worker only caches assets from the same origin; no third-party JavaScript.
- **Desktop** — Tauri's CSP and `tauri.conf.json` allowlist locked to the minimum filesystem, dialog, and shell scopes required. The shell scope is *not* opened — Foundry never spawns child processes.
- **Mobile** — every permission requested lazily at point of use per §6 (camera only when the user opens camera, photo library only when the user picks). No background services, no analytics SDK, no Crashlytics, no Firebase, no IDFA, no ATT prompt.
- **Bot** — server-side rate limiting (token bucket per source IP); per-user quota; image-content validation before any decode call; dimension caps as on the CLI (default 32 768²). All routes authenticated; no anonymous surface beyond `/healthz`.
- **Cloud (when opted in)** — TLS 1.3 + PQC hybrid (X25519 + ML-KEM-768); image envelopes BLAKE3-keyed; storage ephemeral, deleted within 60 s of result delivery; logs retained at debug grade for at most 24 h, never with image hashes attached.

---

## 11. Compliance Checklist (§13 Audit Gate)

Per Steelbore Standard §13 audit gate, this PRD satisfies:

- ✅ **§2** Metallurgical naming applied to all eight new module crates (Foundry, Cabinet, Billet, plus the four shells)
- ✅ **§3.1** Memory safety: Rust-first throughout; mobile UI in Flutter (Dart) with ASLR + CFI mitigations as Standard §3.1 prescribes for non-Rust pathways
- ✅ **§3.2** Concurrency designed-in; the core's job queue is Rayon-aware and inherits Caliper's three-level parallelism
- ✅ **§3.3** PQC readiness for cloud TLS; `cargo-audit` / `cargo-vet` / `cargo-deny` in CI; CFI; ASLR
- ✅ **§4** GPL-3.0-or-later; SPDX headers on all software source code files; this document declares its license at the head of the file
- ✅ **§5.1** POSIX-compliant CLI (the bot adapter when run as `caliper-foundry-bot`); platform extensions feature-flagged
- ✅ **§6** PFA: no telemetry, no auto-network, local storage default, opt-in cloud only
- ✅ **§7** CUA + Vim keybindings in desktop UI (matches Caliper TUI); web honors the same keymap; mobile uses platform-standard bindings with optional Vim keypad mode
- ✅ **§8** Steelbore palette across all surfaces; Void Navy (`#000027`) background mandatory
- ✅ **§9** Share Tech Mono / Inconsolata fonts shipped in the desktop and mobile app bundles; web uses `@font-face` with self-hosted OFL files
- ✅ **§10** Material Design components (Flutter Material, Tauri Material via Iced or fluent components); WCAG 2.1 AA contrast verified
- ✅ **§11** ISO 8601 dates, 24h time, UTC timestamps, metric units throughout the UI

Per the CLI Standard §9 + Agentic §9 (CLI-specific gates inherited from the bot adapter and shared protocol):

- ✅ ISO 8601 + UTC timestamps everywhere
- ✅ UTF-8 without BOM
- ✅ POSIX-first default output (bot CLI)
- ✅ `--json` on every data-returning command
- ✅ stdout = data only, stderr = everything else
- ✅ Structured errors on stderr in machine mode
- ✅ Path canonicalization + allow-list validation
- ✅ Non-TTY destructive ops require `--yes` / `--force`
- ✅ `caliper-foundry schema` emits JSON Schema Draft 2020-12
- ✅ `NO_COLOR` honored
- ✅ `--dry-run` on every write command
- ✅ `AI_AGENT=1` triggers JSON + no-color + non-interactive
- ✅ Every error response has a non-empty runnable `hint`
- ✅ MCP server uses lazy schema loading (inherited from Caliper)
- ✅ AGENTS.md, CLAUDE.md, SKILL.md, CONTRIBUTING.md at repo root

---

## 12. Roadmap

### 12.1 v0.1.0 — Foundation (Web)

- `caliper-foundry-protocol` and `caliper-foundry-core` crates scaffolded
- Web PWA built with Leptos
- Vectorize-only UI on top of the WASM build of Caliper v0.1
- Local Cabinet via browser IndexedDB
- Drop / paste / URL input, side-by-side preview, SVG export

### 12.2 v0.2.0 — Desktop and Upscale

- Tauri 2.x desktop shell
- Touchstone integration on desktop (CoreML on macOS, DirectML on Windows, OpenVINO on Linux)
- Upscale operation with Real-ESRGAN-x4 INT8 default model
- Combined upscale → trace flow with intermediate raster cache
- Multi-document tabs, full path editor, batch mode

### 12.3 v0.3.0 — Mobile

- Flutter shell over `caliper-foundry-core` via `flutter_rust_bridge`
- iOS CoreML, Android NNAPI / QNN
- Share-extension entry points (open from Photos, from Files, from any app's share sheet)
- Battery-aware queue throttling

### 12.4 v0.4.0 — Bot

- HTTP / MCP server
- Poe + Discord + Slack + Telegram adapter feature flags
- Self-host docs (Docker compose + systemd unit + nix flake module)
- Rate limiting + quota management

### 12.5 v1.0.0 — General Availability

- Full feature parity across web / desktop / mobile / bot
- App Store + Play Store + Mac App Store + Microsoft Store submissions
- Compliance audit complete
- AGENTS.md + CLAUDE.md + SKILL.md + CONTRIBUTING.md finalized
- Public launch at `foundry.caliper.dev`

### 12.6 v1.1.0 — Atelier (Templates)

- Logo / icon template library: 50–100 starter templates, FOSS-licensed (CC0 or OFL where typographic)
- Customization on top of vectorize results — recolor, retext, recombine
- Mirror the Logo Maker Android pattern but template-based, *not* AI-generated, so output remains user-owned and license-clean

### 12.7 v1.2.0 — Extended Outputs

- AI (Adobe Illustrator), GeoJSON, HPGL outputs (matches Caliper v1.2)
- Plugin system: import a custom output writer or pre/post-processing stage
- WebAssembly plugin sandbox for third-party extensions on desktop

### 12.8 v2.0.0 — Trained Assayer

- The Caliper v2.0 trained Assayer model is exposed in Foundry's auto-tune UI
- Differentiable parameter tuning: user adjusts a "more / less detail" slider, system gradient-tunes back-end parameters in real time
- Region-aware routing surface: tap a region in the preview to set per-region tracing parameters

---

## 13. References

### 13.1 Steelbore-Internal

- The Steelbore Standard v1.0 — `github.com/UnbreakableMJ/steelbore-standard`
- Steelbore CLI Standard v1.0.0 — Dual-Mode Self-Documenting CLI Framework
- Steelbore Rust Guidelines — `github.com/UnbreakableMJ/rust-guidelines`
- Caliper PRD v2.0 — parent document
- Mawaqit (Steelbore Flutter + Rust + libmawaqit precedent)
- Lattice (NixOS flake — used to package the desktop and bot Linux distributions)

### 13.2 Front-End Stacks

- Tauri 2.x — `tauri.app`
- Leptos — `leptos.dev`
- Yew — `yew.rs`
- Flutter — `flutter.dev`
- `flutter_rust_bridge` — `cjycode.com/flutter_rust_bridge`
- `wasm-bindgen-rayon` — `crates.io/crates/wasm-bindgen-rayon`
- `rmcp` (Rust Model Context Protocol) — `crates.io/crates/rmcp`
- `axum` — `crates.io/crates/axum`

### 13.3 Competitive Landscape

- Vectorizer.io — `vectorizer.io`
- Vectorizer.AI — `vectorizer.ai`
- Topaz Labs — `topazlabs.com`
- Topaz Gigapixel — `topazlabs.com/topaz-gigapixel`
- Poe TopazLabs Bot — `poe.com/TopazLabs`
- Poe Server Bot Protocol — `creator.poe.com`

### 13.4 Super-Resolution Models

- Real-ESRGAN — `github.com/xinntao/Real-ESRGAN`
- SwinIR — `github.com/JingyunLiang/SwinIR`
- DRCT — Dense Residual Connected Transformer (state-of-the-art real-world SR)
- ONNX Runtime Execution Providers — `onnxruntime.ai/docs/execution-providers`

---

*Forged in Steelbore.*
