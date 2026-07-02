# tauri-apple-intelligence

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
| Generate / stream | `apple_ai_generate`, `apple_ai_stream`, `apple_ai_cancel_stream` | Basic, tools, and structured modes; `model: "on-device" \| "private-cloud"`, `reasoningLevel`, and per-message `images` |
| Context window | `apple_ai_context_info` | Real `contextSize` per model (4k on-device, ~32k PCC) — stop hardcoding |
| Supported languages | `apple_ai_supported_languages` | Live BCP-47 tags from `SystemLanguageModel.supportedLanguages` |
| Prewarm | `apple_ai_prewarm` | Lower first-token latency |

Generation results and streams carry a `usage` object (`inputTokens`, `cachedInputTokens`,
`outputTokens`, `reasoningTokens`) on macOS 27+.

## Usage

Then register the commands in your `tauri::Builder`:

```rust
tauri::Builder::default()
  .invoke_handler(tauri::generate_handler![
    tauri_apple_intelligence::apple_ai_check_availability,
    tauri_apple_intelligence::apple_ai_generate,
    tauri_apple_intelligence::apple_ai_stream,
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

- ✅ macOS 26+ on Apple Silicon (on-device model, streaming, tools, structured output)
- ✅ macOS 27+ adds Private Cloud Compute, reasoning levels, multimodal image input, and per-call
  token usage — all gated behind `@available(macOS 27, *)`, so the crate still runs on macOS 26 with
  those features simply unavailable
- ❌ Other platforms (returns `UnsupportedPlatform`)

## License

MIT