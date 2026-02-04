# tauri-apple-intelligence

Native Tauri commands for Apple Intelligence (Foundation Models) with streaming + tool calling support.

Companion JS transport: https://github.com/entro314-labs/apple-intelligence-sdk

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

- ✅ macOS 26+ on Apple Silicon
- ❌ Other platforms (returns `UnsupportedPlatform`)

## License

MIT