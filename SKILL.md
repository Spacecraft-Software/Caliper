<!--
SPDX-License-Identifier: GPL-3.0-or-later
Copyright (c) 2026 Mohamed Hammad
-->

# SKILL.md

**Discoverable skill manifest for agentic clients.**

Caliper exposes itself to LLM-driven tooling along three surfaces. This
file is the index to all three — start here, then jump to the canonical
descriptor for each.

| Surface              | What it is                                           | Canonical descriptor          |
|----------------------|------------------------------------------------------|-------------------------------|
| `caliper` CLI        | POSIX-compliant binary; `--json` on every data verb  | `caliper describe`, `caliper schema` |
| `caliper mcp` server | Model Context Protocol server (stdio / sse / http)  | `caliper mcp --transport stdio` then `tools/list` |
| `libcaliper` C-ABI   | Shared library for embedding into other processes    | `target/include/caliper.h` (cbindgen-generated) |

---

## Quick-Reference Capability Matrix

| Capability                                | CLI verb                                  | MCP tool                | Tags                           |
|-------------------------------------------|-------------------------------------------|-------------------------|--------------------------------|
| Trace a single image                      | `caliper trace`                           | `caliper.trace`         | `write`, `idempotent`          |
| Batch-trace many images                   | `caliper batch`                           | `caliper.batch`         | `write`, `idempotent`          |
| Inspect an image (no output)              | `caliper inspect`                         | `caliper.inspect`       | `read`, `idempotent`           |
| Quick low-res preview                     | `caliper preview`                         | `caliper.preview`       | `read`, `idempotent`           |
| Performance profile                       | `caliper benchmark`                       | `caliper.benchmark`     | `read`, `idempotent`           |
| Extract palette                           | `caliper palette`                         | `caliper.palette`       | `read`, `idempotent`           |
| View persistent config                    | `caliper config get`                      | `caliper.config.get`    | `read`, `idempotent`           |
| Modify persistent config                  | `caliper config set`                      | `caliper.config.set`    | `write`, `idempotent`          |
| List ML models                            | `caliper models list`                     | `caliper.models.list`   | `read`, `idempotent`           |
| Download an ML model                      | `caliper models pull`                     | `caliper.models.pull`   | `write`, `destructive` (network)|
| Verify model integrity                    | `caliper models verify`                   | `caliper.models.verify` | `read`, `idempotent`           |
| Emit per-command JSON Schema              | `caliper schema [<command>]`              | `caliper.schema`        | `read`, `idempotent`           |
| Emit tool manifest                        | `caliper describe`                        | `caliper.describe`      | `read`, `idempotent`           |

Capability tags follow the [Steelbore Agentic CLI Standard §6](https://Steelbore.com/standard) —
the same scheme MCP `tools/list` uses for lazy schema loading.

---

## CLI Discovery

The fastest path to "what can this thing do" is `caliper describe`. It returns
a single JSON document containing every sub-command, every flag, and a
capability tag for each.

```json
{
  "metadata": { "tool": "caliper", "version": "1.0.0" },
  "data": {
    "commands": [
      {
        "name": "trace",
        "summary": "Trace a single image to vector output.",
        "tags": ["write", "idempotent"],
        "schema": "caliper schema trace"
      },
      "..."
    ]
  }
}
```

For full per-command schemas: `caliper schema trace`, `caliper schema batch`,
etc. Each emits a JSON Schema Draft 2020-12 document describing both the
arguments and the output payload.

---

## MCP Server

Start the server:

```sh
caliper mcp --transport stdio
caliper mcp --transport sse --port 7421
caliper mcp --transport streamable-http --port 7421
```

The server uses **lazy schema loading** (PRD §9.9): `tools/list` returns one-line
descriptions and capability tags only — full schemas are loaded on `tools/get`.
This keeps token budgets sane for clients with many MCP servers attached.

---

## Library Embedding

Rust:

```rust
use caliper_trace::{TracingConfig, trace};

let config = TracingConfig::builder()
    .colors(16)
    .accelerator(caliper_trace::Backend::Auto)
    .build();
let doc = trace(&image::open("logo.png")?, &config)?;
doc.write_svg("logo.svg")?;
```

C (via `libcaliper.so` / `.dylib` / `.dll`):

```c
caliper_config_t* cfg = caliper_config_new();
caliper_config_set_colors(cfg, 16);
caliper_status_t st = caliper_trace_file("logo.png", "logo.svg", cfg);
caliper_config_free(cfg);
```

All FFI is panic-safe (`std::panic::catch_unwind` at the boundary) and returns
status codes rather than panicking across the boundary.

---

## Maintainer

**Mohamed Hammad** &lt;[Mohamed.Hammad@Steelbore.com](mailto:Mohamed.Hammad@Steelbore.com)&gt;
Copyright (c) 2026 Mohamed Hammad &nbsp;|&nbsp; License: GPL-3.0-or-later
[https://Caliper.Steelbore.com/](https://Caliper.Steelbore.com/)
