# tauri-apple-intelligence

> [!WARNING]
> **Deprecated.** This crate is no longer maintained. It has been merged into
> [`tauri-plugin-apple-intelligence`](https://crates.io/crates/tauri-plugin-apple-intelligence)
> ([repo](https://github.com/entro314-labs/tauri-plugin-apple-intelligence)), a proper Tauri v2
> plugin with namespaced, permission-gated commands.

## Migrating

```diff
  # src-tauri/Cargo.toml
- tauri-apple-intelligence = "0.6"
+ tauri-plugin-apple-intelligence = "0.12"
```

```diff
  tauri::Builder::default()
-     .invoke_handler(tauri::generate_handler![
-         tauri_apple_intelligence::apple_ai_check_availability,
-         // ...the other apple_ai_* commands
-     ])
+     .plugin(tauri_plugin_apple_intelligence::init())
```

Add `"apple-intelligence:default"` to your capability's `permissions`, and switch the frontend from
`@entro314labs/apple-intelligence-sdk` to `@entro314labs/plugin-apple-intelligence`. Full steps
(including bundling `libappleai.dylib`) are in the plugin's
[Setup](https://github.com/entro314-labs/tauri-plugin-apple-intelligence#setup).

The original documentation follows for users pinned to 0.6.x.

---

Native Tauri commands for Apple Intelligence (Foundation Models) with streaming, tool calling,
**Private Cloud Compute**, **reasoning**, **multimodal image input**, and **live token/context usage**.

Companion JS transport: https://github.com/entro314-labs/apple-intelligence-sdk

## Install

```toml
[dependencies]
tauri-apple-intelligence = "0.6"
```

## Capabilities

| Capability | Command(s) | Notes |
|---|---|---|
| Availability (on-device) | `apple_ai_check_availability` | Device eligible + Apple Intelligence enabled + model ready |
| Availability (Private Cloud Compute) | `apple_ai_pcc_check_availability` | macOS 27+; larger, reasoning-capable, still private (no API key/bill) |
| Generate / stream | `apple_ai_generate`, `apple_ai_stream`, `apple_ai_cancel_stream` | Basic, tools, and structured modes; `model: "on-device" \| "private-cloud"`, `reasoningLevel`, sampling (`temperature` incl. `0`, `topP`, `topK`, `seed`), `toolChoice` (macOS 27), and per-message `images` |
| Context window | `apple_ai_context_info` | Real `contextSize` per model (4k on-device, ~32k PCC) — stop hardcoding |
| Token counting | `apple_ai_token_count` | Real tokenizer counts (`tokenCount(for:)`, macOS 26.4+) for budgeting prompts against `contextSize` |
| Supported languages | `apple_ai_supported_languages` | Live BCP-47 tags from `SystemLanguageModel.supportedLanguages` |
| Prewarm | `apple_ai_prewarm` | Lower first-token latency; optional `promptPrefix` eagerly processes a known prompt prefix |

Generation results and streams carry a `usage` object (`inputTokens`, `cachedInputTokens`,
`outputTokens`, `reasoningTokens`) on macOS 27+.

### Typed errors

Generation failures carry a stable machine-readable `code` mirroring the FoundationModels error
cases — `context-window-exceeded`, `guardrail-violation`, `refusal`, `rate-limited`,
`concurrent-requests`, and more. Non-streaming commands reject with
`{ type: "generation", code, message, contextSize?, tokenCount? }`; streams emit an `error` event
with the same fields. `context-window-exceeded` includes the model's `contextSize` and the
offending `tokenCount` (macOS 27+), so hosts can condense the conversation and retry — Apple's
documented recovery strategy for the context window.

## Usage

Then register the commands in your `tauri::Builder`:

```rust
tauri::Builder::default()
  .invoke_handler(tauri::generate_handler![
    tauri_apple_intelligence::apple_ai_check_availability,
    tauri_apple_intelligence::apple_ai_pcc_check_availability,
    tauri_apple_intelligence::apple_ai_generate,
    tauri_apple_intelligence::apple_ai_stream,
    tauri_apple_intelligence::apple_ai_cancel_stream,
    tauri_apple_intelligence::apple_ai_context_info,
    tauri_apple_intelligence::apple_ai_token_count,
    tauri_apple_intelligence::apple_ai_supported_languages,
    tauri_apple_intelligence::apple_ai_prewarm,
  ])
```

For the JavaScript side, use the companion npm package:

```ts
import {
  createAppleIntelligenceProvider,
  createTauriAppleIntelligenceTransport,
} from "@entro314labs/apple-intelligence-sdk";
```

## Prebuilt native library

This crate ships a prebuilt `libappleai.dylib` in the `prebuilt/` directory. You must bundle it into your Tauri app resources so the linker and runtime can find it.

Recommended flow in your Tauri app:

1. Copy the prebuilt dylib into your app's `src-tauri/resources/` folder during the build.
2. Reference it in `tauri.conf.json` under `bundle.resources`.

There are two common ways to automate the copy step:

- **App build script**: use your app's `src-tauri/build.rs` (not this crate) to copy `prebuilt/libappleai.dylib` into `src-tauri/resources/`.
- **Before-build hook**: add a small script (Node/Bash) and call it from `tauri.conf.json`'s `build.beforeBuildCommand` or your package manager's build pipeline.

If you want, the copy script can locate this crate via `cargo metadata` and copy the dylib from the Cargo registry to your app's resources folder.

## Linking the native library

This crate expects a `libappleai.dylib` (Foundation Models bridge) to be linked into your Tauri app.
Make sure the dylib is bundled as a resource and that the linker can resolve it at runtime.

## Command prefix

The JS transport expects `apple_ai_*` command names by default. If you rename the commands or use a different prefix, update the JS transport's `commandPrefix` option.

## Supported platforms

- ✅ macOS 26+ on Apple Silicon (on-device model, streaming, tools, structured output, sampling
  modes, typed error codes)
- ✅ macOS 26.4+ adds `apple_ai_token_count` (`tokenCount(for:)`)
- ✅ macOS 27+ adds Private Cloud Compute, reasoning levels, multimodal image input, per-call
  token usage, native `toolChoice` enforcement, and context-size details on
  `context-window-exceeded` errors — all gated behind `@available`, so the crate still runs on
  macOS 26 with those features simply unavailable
- ❌ Other platforms (returns `UnsupportedPlatform`)

## License

MIT