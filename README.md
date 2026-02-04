# tauri-apple-intelligence

Native Tauri commands for Apple Intelligence (Foundation Models) with streaming + tool calling support.

## Install

```toml
[dependencies]
tauri-apple-intelligence = "0.1"
```

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
} from "@equidock/apple-intelligence-sdk";
```

## Linking the native library

This crate expects a `libappleai.dylib` (Foundation Models bridge) to be linked into your Tauri app.
Configure your `build.rs` to add the correct `rustc-link-search` path and bundle the dylib as a resource.

## Command prefix

The JS transport expects `apple_ai_*` command names by default. If you rename the commands or use a different prefix, update the JS transport's `commandPrefix` option.

## Supported platforms

- ✅ macOS 26+ on Apple Silicon
- ❌ Other platforms (returns `UnsupportedPlatform`)

## License

MIT