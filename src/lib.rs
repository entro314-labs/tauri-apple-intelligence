use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::AppHandle;

/// An image attached to a user turn (multimodal input, macOS 27+). Provide either a `fileURL` (a
/// path or `file://` URL — preferred, zero-copy) or inline `base64` bytes. `mediaType` is advisory.
/// Field names are chosen to match the Swift bridge's `ImageInput` decoder exactly.
#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppleAIImageInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(rename = "fileURL", default, skip_serializing_if = "Option::is_none")]
    pub file_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base64: Option<String>,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppleAIMessage {
    pub role: String,
    pub content: Option<String>,
    pub name: Option<String>,
    pub tool_call_id: Option<String>,
    pub tool_calls: Option<Vec<AppleAIToolCall>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<AppleAIImageInput>>,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppleAIToolDefinition {
    pub name: String,
    pub description: Option<String>,
    pub parameters: serde_json::Value,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppleAIToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: AppleAIToolCallFunction,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppleAIToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppleAIGenerateRequest {
    pub messages: Vec<AppleAIMessage>,
    pub tools: Option<Vec<AppleAIToolDefinition>>,
    pub schema: Option<serde_json::Value>,
    /// `"on-device"` (default) or `"private-cloud"` (macOS 27 Private Cloud Compute).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Reasoning level for reasoning-capable models: `"light" | "moderate" | "deep"` (or a custom
    /// string). `None` disables reasoning. Only honored on macOS 27+.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_level: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<i32>,
    pub stop_after_tool_calls: Option<bool>,
}

/// Token usage for one generation. All counts are `0` on macOS 26 (which does not report per-call
/// token usage); real counts arrive on macOS 27+.
#[derive(Serialize, Deserialize, specta::Type, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppleAIUsage {
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppleAIGenerateResult {
    pub text: String,
    pub tool_calls: Option<Vec<AppleAIToolCall>>,
    pub object: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<AppleAIUsage>,
}

/// Context-window info for a model. `context_size` is the max token count; `-1` when it cannot be
/// determined (e.g. Private Cloud Compute unavailable, or queried on macOS &lt; 27).
#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppleAIContextInfo {
    pub model: String,
    pub context_size: i64,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppleAIAvailability {
    pub available: bool,
    pub reason: String,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppleAIStreamStart {
    pub stream_id: String,
    pub event_name: String,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AppleAIStreamEvent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "reasoning")]
    Reasoning { text: String },
    #[serde(rename = "tool-call")]
    ToolCall {
        tool_call_id: String,
        tool_name: String,
        args: serde_json::Value,
    },
    #[serde(rename = "usage")]
    Usage { usage: AppleAIUsage },
    #[serde(rename = "done")]
    Done,
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Serialize, specta::Type, Debug)]
#[serde(tag = "type", content = "message", rename_all = "camelCase")]
pub enum AppleAIError {
    UnsupportedPlatform(String),
    NativeError(String),
    StreamBusy(String),
    InvalidPayload(String),
}

impl std::fmt::Display for AppleAIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppleAIError::UnsupportedPlatform(message)
            | AppleAIError::NativeError(message)
            | AppleAIError::StreamBusy(message)
            | AppleAIError::InvalidPayload(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for AppleAIError {}

pub fn apple_ai_check_availability() -> Result<AppleAIAvailability, AppleAIError> {
    native::check_availability()
}

pub fn apple_ai_generate(
    request: AppleAIGenerateRequest,
) -> Result<AppleAIGenerateResult, AppleAIError> {
    native::generate(request)
}

pub fn apple_ai_stream<R: tauri::Runtime>(
    app: AppHandle<R>,
    request: AppleAIGenerateRequest,
) -> Result<AppleAIStreamStart, AppleAIError> {
    native::stream(app, request)
}

/// Cancel the in-flight stream identified by `stream_id` (from [`AppleAIStreamStart`]).
///
/// Returns `Ok(true)` when the stream was active and cancellation was requested, `Ok(false)` when
/// no matching stream is active (it already finished — a stale abort is a harmless no-op). The
/// cancelled stream still terminates through its normal end-of-stream event (`done`), emitted by
/// the native task's cancellation handler, so consumers need no special casing.
pub fn apple_ai_cancel_stream(stream_id: &str) -> Result<bool, AppleAIError> {
    native::cancel_stream(stream_id)
}

/// Availability of the Private Cloud Compute model (macOS 27+ server-side model, private-by-design,
/// no API key). Mirrors [`apple_ai_check_availability`] for the on-device model.
pub fn apple_ai_pcc_check_availability() -> Result<AppleAIAvailability, AppleAIError> {
    native::pcc_check_availability()
}

/// Context window (max token count) for a model (`"on-device"` default | `"private-cloud"`). Lets
/// hosts budget prompt content against the real window instead of hardcoding it.
pub fn apple_ai_context_info(model: Option<String>) -> Result<AppleAIContextInfo, AppleAIError> {
    native::context_info(model)
}

/// BCP-47 language tags the on-device model supports (e.g. `["en", "fr", "zh-Hans", …]`), read from
/// the framework at runtime rather than a hardcoded list.
pub fn apple_ai_supported_languages() -> Result<Vec<String>, AppleAIError> {
    native::supported_languages()
}

/// Prewarm a model so the next request pays less first-token latency. Best-effort; returns `Ok(())`
/// even when the model can't be prewarmed on this OS.
pub fn apple_ai_prewarm(model: Option<String>) -> Result<(), AppleAIError> {
    native::prewarm(model)
}

#[doc(hidden)]
#[macro_export]
macro_rules! __cmd__apple_ai_check_availability {
    ($path:path, $invoke:ident) => {{
        move || {
            #[allow(unused_imports)]
            use ::tauri::ipc::private::*;
            #[allow(unused_variables)]
            let ::tauri::ipc::Invoke {
                message: __tauri_message__,
                resolver: __tauri_resolver__,
                acl: __tauri_acl__,
            } = $invoke;

            let result = $path();
            let kind = (&result).blocking_kind();
            kind.block(result, __tauri_resolver__);
            return true;
        }()
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __cmd__apple_ai_generate {
    ($path:path, $invoke:ident) => {{
        move || {
            #[allow(unused_imports)]
            use ::tauri::ipc::private::*;
            #[allow(unused_variables)]
            let ::tauri::ipc::Invoke {
                message: __tauri_message__,
                resolver: __tauri_resolver__,
                acl: __tauri_acl__,
            } = $invoke;

            let request = match ::tauri::ipc::CommandArg::from_command(::tauri::ipc::CommandItem {
                plugin: None,
                name: "apple_ai_generate",
                key: "request",
                message: &__tauri_message__,
                acl: &__tauri_acl__,
            }) {
                Ok(arg) => arg,
                Err(err) => {
                    __tauri_resolver__.invoke_error(err);
                    return true;
                }
            };

            let result = $path(request);
            let kind = (&result).blocking_kind();
            kind.block(result, __tauri_resolver__);
            return true;
        }()
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __cmd__apple_ai_stream {
    ($path:path, $invoke:ident) => {{
        move || {
            #[allow(unused_imports)]
            use ::tauri::ipc::private::*;
            #[allow(unused_variables)]
            let ::tauri::ipc::Invoke {
                message: __tauri_message__,
                resolver: __tauri_resolver__,
                acl: __tauri_acl__,
            } = $invoke;

            let app = match ::tauri::ipc::CommandArg::from_command(::tauri::ipc::CommandItem {
                plugin: None,
                name: "apple_ai_stream",
                key: "app",
                message: &__tauri_message__,
                acl: &__tauri_acl__,
            }) {
                Ok(arg) => arg,
                Err(err) => {
                    __tauri_resolver__.invoke_error(err);
                    return true;
                }
            };

            let request = match ::tauri::ipc::CommandArg::from_command(::tauri::ipc::CommandItem {
                plugin: None,
                name: "apple_ai_stream",
                key: "request",
                message: &__tauri_message__,
                acl: &__tauri_acl__,
            }) {
                Ok(arg) => arg,
                Err(err) => {
                    __tauri_resolver__.invoke_error(err);
                    return true;
                }
            };

            let result = $path(app, request);
            let kind = (&result).blocking_kind();
            kind.block(result, __tauri_resolver__);
            return true;
        }()
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __cmd__apple_ai_pcc_check_availability {
    ($path:path, $invoke:ident) => {{
        move || {
            #[allow(unused_imports)]
            use ::tauri::ipc::private::*;
            #[allow(unused_variables)]
            let ::tauri::ipc::Invoke {
                message: __tauri_message__,
                resolver: __tauri_resolver__,
                acl: __tauri_acl__,
            } = $invoke;

            let result = $path();
            let kind = (&result).blocking_kind();
            kind.block(result, __tauri_resolver__);
            return true;
        }()
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __cmd__apple_ai_supported_languages {
    ($path:path, $invoke:ident) => {{
        move || {
            #[allow(unused_imports)]
            use ::tauri::ipc::private::*;
            #[allow(unused_variables)]
            let ::tauri::ipc::Invoke {
                message: __tauri_message__,
                resolver: __tauri_resolver__,
                acl: __tauri_acl__,
            } = $invoke;

            let result = $path();
            let kind = (&result).blocking_kind();
            kind.block(result, __tauri_resolver__);
            return true;
        }()
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __cmd__apple_ai_context_info {
    ($path:path, $invoke:ident) => {{
        move || {
            #[allow(unused_imports)]
            use ::tauri::ipc::private::*;
            #[allow(unused_variables)]
            let ::tauri::ipc::Invoke {
                message: __tauri_message__,
                resolver: __tauri_resolver__,
                acl: __tauri_acl__,
            } = $invoke;

            let model = match ::tauri::ipc::CommandArg::from_command(::tauri::ipc::CommandItem {
                plugin: None,
                name: "apple_ai_context_info",
                key: "model",
                message: &__tauri_message__,
                acl: &__tauri_acl__,
            }) {
                Ok(arg) => arg,
                Err(err) => {
                    __tauri_resolver__.invoke_error(err);
                    return true;
                }
            };

            let result = $path(model);
            let kind = (&result).blocking_kind();
            kind.block(result, __tauri_resolver__);
            return true;
        }()
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __cmd__apple_ai_prewarm {
    ($path:path, $invoke:ident) => {{
        move || {
            #[allow(unused_imports)]
            use ::tauri::ipc::private::*;
            #[allow(unused_variables)]
            let ::tauri::ipc::Invoke {
                message: __tauri_message__,
                resolver: __tauri_resolver__,
                acl: __tauri_acl__,
            } = $invoke;

            let model = match ::tauri::ipc::CommandArg::from_command(::tauri::ipc::CommandItem {
                plugin: None,
                name: "apple_ai_prewarm",
                key: "model",
                message: &__tauri_message__,
                acl: &__tauri_acl__,
            }) {
                Ok(arg) => arg,
                Err(err) => {
                    __tauri_resolver__.invoke_error(err);
                    return true;
                }
            };

            let result = $path(model);
            let kind = (&result).blocking_kind();
            kind.block(result, __tauri_resolver__);
            return true;
        }()
    }};
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod native {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    };
    use tauri::Emitter;

    #[link(name = "appleai")]
    unsafe extern "C" {
        fn apple_ai_init() -> bool;
        fn apple_ai_check_availability() -> i32;
        fn apple_ai_get_availability_reason() -> *mut std::os::raw::c_char;
        fn apple_ai_free_string(ptr: *mut std::os::raw::c_char);

        fn apple_ai_pcc_check_availability() -> i32;
        fn apple_ai_pcc_get_availability_reason() -> *mut std::os::raw::c_char;
        fn apple_ai_context_size(model: *const std::os::raw::c_char) -> i32;
        fn apple_ai_prewarm(model: *const std::os::raw::c_char);
        fn apple_ai_get_supported_languages_count() -> i32;
        fn apple_ai_get_supported_language(index: i32) -> *mut std::os::raw::c_char;

        fn apple_ai_register_tool_callback(
            cb: Option<extern "C" fn(u64, *const std::os::raw::c_char)>,
        );
        fn apple_ai_tool_result_callback(tool_id: u64, result_json: *const std::os::raw::c_char);

        fn apple_ai_cancel_stream() -> bool;

        fn apple_ai_generate_unified(
            messages_json: *const std::os::raw::c_char,
            tools_json: *const std::os::raw::c_char,
            schema_json: *const std::os::raw::c_char,
            model: *const std::os::raw::c_char,
            reasoning_level: *const std::os::raw::c_char,
            temperature: f64,
            max_tokens: i32,
            stream: bool,
            stop_after_tool_calls: bool,
            on_chunk: Option<extern "C" fn(*const std::os::raw::c_char)>,
        ) -> *mut std::os::raw::c_char;
    }

    static INIT: OnceLock<()> = OnceLock::new();
    static STREAM_ACTIVE: AtomicBool = AtomicBool::new(false);
    static TOOL_CALLS: OnceLock<Mutex<Vec<(String, String, serde_json::Value)>>> = OnceLock::new();
    static TOOL_NAME_MAP: OnceLock<Mutex<std::collections::HashMap<u64, String>>> = OnceLock::new();
    static STREAM_STATE: OnceLock<Mutex<Option<StreamState>>> = OnceLock::new();

    struct StreamState {
        /// Type-erased event emitter capturing the host's `AppHandle<R>` — the state lives in a
        /// static, which cannot be generic over the Tauri runtime, so the runtime is erased here.
        /// This is what lets `apple_ai_stream` accept any `Runtime` (incl. tauri::test::MockRuntime).
        emit: Box<dyn Fn(AppleAIStreamEvent) + Send + Sync>,
        /// Id from [`AppleAIStreamStart`] — `cancel_stream` only acts on a matching id, so a
        /// stale abort for an already-finished stream can never touch a newer one.
        stream_id: String,
        /// Set by `cancel_stream`. The chunk callback re-issues the native cancel on the next
        /// chunk (closing the startup race where the Swift task wasn't registered yet) and stops
        /// emitting text the consumer already abandoned.
        cancel_requested: bool,
    }

    fn ensure_initialized() -> Result<(), AppleAIError> {
        INIT.get_or_init(|| unsafe {
            if !apple_ai_init() {
                panic!("Failed to initialize Apple Intelligence native library");
            }
        });
        Ok(())
    }

    fn take_c_string(ptr: *mut std::os::raw::c_char) -> String {
        if ptr.is_null() {
            return String::new();
        }
        unsafe {
            let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
            apple_ai_free_string(ptr);
            s
        }
    }

    pub fn check_availability() -> Result<AppleAIAvailability, AppleAIError> {
        ensure_initialized()?;
        unsafe {
            let status = apple_ai_check_availability();
            if status == 1 {
                Ok(AppleAIAvailability {
                    available: true,
                    reason: "Available".to_string(),
                })
            } else {
                let reason_ptr = apple_ai_get_availability_reason();
                let reason = take_c_string(reason_ptr);
                Ok(AppleAIAvailability {
                    available: false,
                    reason,
                })
            }
        }
    }

    pub fn generate(
        request: AppleAIGenerateRequest,
    ) -> Result<AppleAIGenerateResult, AppleAIError> {
        ensure_initialized()?;

        let messages_json = serde_json::to_string(&request.messages)
            .map_err(|e| AppleAIError::InvalidPayload(e.to_string()))?;
        let tools_json = serialize_tools(&request.tools)?;
        let schema_json = request
            .schema
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| AppleAIError::InvalidPayload(e.to_string()))?;

        let c_messages = CString::new(messages_json)
            .map_err(|_| AppleAIError::InvalidPayload("Messages contained null byte".into()))?;
        let c_tools = tools_json
            .map(CString::new)
            .transpose()
            .map_err(|_| AppleAIError::InvalidPayload("Tools contained null byte".into()))?;
        let c_schema = schema_json
            .map(CString::new)
            .transpose()
            .map_err(|_| AppleAIError::InvalidPayload("Schema contained null byte".into()))?;
        let c_model = optional_cstring(request.model.as_deref())?;
        let c_reasoning = optional_cstring(request.reasoning_level.as_deref())?;

        if request.tools.as_ref().is_some_and(|t| !t.is_empty()) {
            register_tool_callback();
        }

        let result_ptr = unsafe {
            apple_ai_generate_unified(
                c_messages.as_ptr(),
                c_tools
                    .as_ref()
                    .map_or(std::ptr::null(), |value| value.as_ptr()),
                c_schema
                    .as_ref()
                    .map_or(std::ptr::null(), |value| value.as_ptr()),
                c_model
                    .as_ref()
                    .map_or(std::ptr::null(), |value| value.as_ptr()),
                c_reasoning
                    .as_ref()
                    .map_or(std::ptr::null(), |value| value.as_ptr()),
                request.temperature.unwrap_or(0.0),
                request.max_tokens.unwrap_or(0),
                false,
                request.stop_after_tool_calls.unwrap_or(true),
                None,
            )
        };

        if result_ptr.is_null() {
            return Err(AppleAIError::NativeError("Generation returned null".into()));
        }

        let raw = take_c_string(result_ptr);
        if raw.starts_with("Error: ") {
            return Err(AppleAIError::NativeError(
                raw.trim_start_matches("Error: ").to_string(),
            ));
        }

        let parsed: serde_json::Value =
            serde_json::from_str(&raw).map_err(|e| AppleAIError::InvalidPayload(e.to_string()))?;

        let text = parsed
            .get("text")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string();
        let tool_calls = parsed
            .get("toolCalls")
            .cloned()
            .and_then(|value| serde_json::from_value(value).ok());
        let object = parsed.get("object").cloned();
        let usage = parsed.get("usage").and_then(parse_usage);

        Ok(AppleAIGenerateResult {
            text,
            tool_calls,
            object,
            usage,
        })
    }

    pub fn stream<R: tauri::Runtime>(
        app: AppHandle<R>,
        request: AppleAIGenerateRequest,
    ) -> Result<AppleAIStreamStart, AppleAIError> {
        ensure_initialized()?;

        if STREAM_ACTIVE.swap(true, Ordering::SeqCst) {
            return Err(AppleAIError::StreamBusy(
                "Another Apple Intelligence stream is already active".into(),
            ));
        }

        let stream_id = uuid::Uuid::new_v4().to_string();
        let event_name = format!("apple-ai://stream/{stream_id}");

        let emit_event_name = event_name.clone();
        let state = StreamState {
            emit: Box::new(move |event| {
                let _ = app.emit(&emit_event_name, event);
            }),
            stream_id: stream_id.clone(),
            cancel_requested: false,
        };

        let state_mutex = STREAM_STATE.get_or_init(|| Mutex::new(None));
        *state_mutex.lock().unwrap() = Some(state);

        TOOL_CALLS
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .unwrap()
            .clear();
        TOOL_NAME_MAP
            .get_or_init(|| Mutex::new(std::collections::HashMap::new()))
            .lock()
            .unwrap()
            .clear();

        let messages_json = serde_json::to_string(&request.messages)
            .map_err(|e| AppleAIError::InvalidPayload(e.to_string()))?;
        let tools_json = serialize_tools(&request.tools)?;
        let schema_json = request
            .schema
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| AppleAIError::InvalidPayload(e.to_string()))?;

        let c_messages = CString::new(messages_json)
            .map_err(|_| AppleAIError::InvalidPayload("Messages contained null byte".into()))?;
        let c_tools = tools_json
            .map(CString::new)
            .transpose()
            .map_err(|_| AppleAIError::InvalidPayload("Tools contained null byte".into()))?;
        let c_schema = schema_json
            .map(CString::new)
            .transpose()
            .map_err(|_| AppleAIError::InvalidPayload("Schema contained null byte".into()))?;
        let c_model = optional_cstring(request.model.as_deref())?;
        let c_reasoning = optional_cstring(request.reasoning_level.as_deref())?;

        if request.tools.as_ref().is_some_and(|t| !t.is_empty()) {
            register_tool_callback();
        }

        std::thread::spawn(move || unsafe {
            apple_ai_generate_unified(
                c_messages.as_ptr(),
                c_tools
                    .as_ref()
                    .map_or(std::ptr::null(), |value| value.as_ptr()),
                c_schema
                    .as_ref()
                    .map_or(std::ptr::null(), |value| value.as_ptr()),
                c_model
                    .as_ref()
                    .map_or(std::ptr::null(), |value| value.as_ptr()),
                c_reasoning
                    .as_ref()
                    .map_or(std::ptr::null(), |value| value.as_ptr()),
                request.temperature.unwrap_or(0.0),
                request.max_tokens.unwrap_or(0),
                true,
                request.stop_after_tool_calls.unwrap_or(true),
                Some(stream_chunk_callback),
            );
        });

        Ok(AppleAIStreamStart {
            stream_id,
            event_name,
        })
    }

    pub fn cancel_stream(stream_id: &str) -> Result<bool, AppleAIError> {
        let state_mutex = STREAM_STATE.get_or_init(|| Mutex::new(None));
        let mut guard = state_mutex.lock().unwrap();
        let Some(state) = guard.as_mut() else {
            return Ok(false);
        };
        if state.stream_id != stream_id {
            return Ok(false);
        }

        state.cancel_requested = true;
        // The Swift side cancels its in-flight task; the task's cancellation handler emits the
        // terminal nil chunk, which flows through `stream_chunk_callback` to emit `done`, reset
        // STREAM_ACTIVE and clear this state. If the task wasn't registered yet (startup race),
        // the chunk callback above re-issues the cancel on the first chunk.
        unsafe {
            apple_ai_cancel_stream();
        }
        Ok(true)
    }

    pub fn pcc_check_availability() -> Result<AppleAIAvailability, AppleAIError> {
        ensure_initialized()?;
        unsafe {
            let status = apple_ai_pcc_check_availability();
            if status == 1 {
                Ok(AppleAIAvailability {
                    available: true,
                    reason: "Available".to_string(),
                })
            } else {
                let reason = take_c_string(apple_ai_pcc_get_availability_reason());
                Ok(AppleAIAvailability {
                    available: false,
                    reason,
                })
            }
        }
    }

    pub fn context_info(model: Option<String>) -> Result<AppleAIContextInfo, AppleAIError> {
        ensure_initialized()?;
        let model = model.unwrap_or_else(|| "on-device".to_string());
        let c_model = CString::new(model.clone())
            .map_err(|_| AppleAIError::InvalidPayload("Model contained null byte".into()))?;
        let size = unsafe { apple_ai_context_size(c_model.as_ptr()) };
        Ok(AppleAIContextInfo {
            model,
            context_size: size as i64,
        })
    }

    pub fn supported_languages() -> Result<Vec<String>, AppleAIError> {
        ensure_initialized()?;
        unsafe {
            let count = apple_ai_get_supported_languages_count();
            if count <= 0 {
                return Ok(Vec::new());
            }
            let mut languages = Vec::with_capacity(count as usize);
            for index in 0..count {
                let ptr = apple_ai_get_supported_language(index);
                if ptr.is_null() {
                    continue;
                }
                let tag = take_c_string(ptr);
                if !tag.is_empty() {
                    languages.push(tag);
                }
            }
            Ok(languages)
        }
    }

    pub fn prewarm(model: Option<String>) -> Result<(), AppleAIError> {
        ensure_initialized()?;
        let model = model.unwrap_or_else(|| "on-device".to_string());
        let c_model = CString::new(model)
            .map_err(|_| AppleAIError::InvalidPayload("Model contained null byte".into()))?;
        unsafe { apple_ai_prewarm(c_model.as_ptr()) };
        Ok(())
    }

    /// Build an optional C string for a `generate_unified` argument, treating empty as absent so the
    /// Swift side sees a null pointer (its default) rather than an empty string.
    fn optional_cstring(value: Option<&str>) -> Result<Option<CString>, AppleAIError> {
        value
            .filter(|s| !s.is_empty())
            .map(CString::new)
            .transpose()
            .map_err(|_| {
                AppleAIError::InvalidPayload("string contained an interior null byte".into())
            })
    }

    /// Parse the `usage` object the Swift bridge attaches to a generation result / stream.
    fn parse_usage(value: &serde_json::Value) -> Option<AppleAIUsage> {
        serde_json::from_value(value.clone()).ok()
    }

    fn serialize_tools(
        tools: &Option<Vec<AppleAIToolDefinition>>,
    ) -> Result<Option<String>, AppleAIError> {
        let Some(tools) = tools else {
            return Ok(None);
        };
        if tools.is_empty() {
            return Ok(None);
        }

        let map = TOOL_NAME_MAP.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
        let mut guard = map.lock().unwrap();
        guard.clear();
        for (index, tool) in tools.iter().enumerate() {
            guard.insert((index + 1) as u64, tool.name.clone());
        }

        let payload: Vec<serde_json::Value> = tools
            .iter()
            .enumerate()
            .map(|(index, tool)| {
                json!({
                    "id": index + 1,
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.parameters,
                })
            })
            .collect();

        serde_json::to_string(&payload)
            .map(Some)
            .map_err(|e| AppleAIError::InvalidPayload(e.to_string()))
    }

    fn register_tool_callback() {
        unsafe { apple_ai_register_tool_callback(Some(tool_callback)) }
    }

    extern "C" fn tool_callback(tool_id: u64, args_json: *const std::os::raw::c_char) {
        let args = unsafe {
            if args_json.is_null() {
                serde_json::Value::Object(serde_json::Map::new())
            } else {
                let raw = CStr::from_ptr(args_json).to_string_lossy().into_owned();
                serde_json::from_str(&raw).unwrap_or_else(|_| json!({}))
            }
        };

        let call_id = format!("tool-call-{}", uuid::Uuid::new_v4());
        let tool_name = TOOL_NAME_MAP
            .get()
            .and_then(|map| {
                map.lock()
                    .ok()
                    .and_then(|guard| guard.get(&tool_id).cloned())
            })
            .unwrap_or_else(|| format!("tool-{tool_id}"));

        if let Some(store) = TOOL_CALLS.get() {
            store
                .lock()
                .unwrap()
                .push((call_id, tool_name, args.clone()));
        }

        let result = CString::new("{}").unwrap();
        unsafe { apple_ai_tool_result_callback(tool_id, result.as_ptr()) };
    }

    // Streaming chunk channel tags — must match the Swift bridge's sentinel table. Untagged chunks
    // are plain answer-text deltas.
    const ERROR_SENTINEL: u8 = 0x02;
    const REASONING_SENTINEL: u8 = 0x03;
    const USAGE_SENTINEL: u8 = 0x04;

    extern "C" fn stream_chunk_callback(ptr: *const std::os::raw::c_char) {
        let state_mutex = STREAM_STATE.get_or_init(|| Mutex::new(None));
        let mut guard = state_mutex.lock().unwrap();
        let Some(state) = guard.as_ref() else {
            return;
        };

        if ptr.is_null() {
            emit_tool_calls(state);
            emit_event(state, AppleAIStreamEvent::Done);
            STREAM_ACTIVE.store(false, Ordering::SeqCst);
            *guard = None;
            return;
        }

        let slice = take_stream_string(ptr);
        if slice.is_empty() {
            return;
        }

        let bytes = slice.as_bytes();
        match bytes.first() {
            Some(&ERROR_SENTINEL) => {
                let message = String::from_utf8_lossy(&bytes[1..]).into_owned();
                emit_event(state, AppleAIStreamEvent::Error { message });
                STREAM_ACTIVE.store(false, Ordering::SeqCst);
                *guard = None;
                return;
            }
            Some(&USAGE_SENTINEL) => {
                if let Ok(usage) = serde_json::from_slice::<AppleAIUsage>(&bytes[1..]) {
                    emit_event(state, AppleAIStreamEvent::Usage { usage });
                }
                return;
            }
            Some(&REASONING_SENTINEL) => {
                let text = String::from_utf8_lossy(&bytes[1..]).into_owned();
                emit_event(state, AppleAIStreamEvent::Reasoning { text });
                return;
            }
            _ => {}
        }

        if state.cancel_requested {
            // The consumer already aborted: drop the text and re-issue the native cancel — this
            // closes the race where `cancel_stream` ran before the Swift task registered itself.
            unsafe {
                apple_ai_cancel_stream();
            }
            return;
        }

        emit_event(state, AppleAIStreamEvent::Text { text: slice });
    }

    fn take_stream_string(ptr: *const std::os::raw::c_char) -> String {
        if ptr.is_null() {
            return String::new();
        }

        unsafe {
            let owned = CString::from_raw(ptr as *mut std::os::raw::c_char);
            owned.to_string_lossy().into_owned()
        }
    }

    fn emit_tool_calls(state: &StreamState) {
        if let Some(store) = TOOL_CALLS.get() {
            let mut calls = store.lock().unwrap();
            let drained: Vec<_> = calls.drain(..).collect();
            for (id, name, args) in drained {
                emit_event(
                    state,
                    AppleAIStreamEvent::ToolCall {
                        tool_call_id: id,
                        tool_name: name,
                        args,
                    },
                );
            }
        }
    }

    fn emit_event(state: &StreamState, event: AppleAIStreamEvent) {
        (state.emit)(event);
    }
}

#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
mod native {
    use super::*;

    pub fn check_availability() -> Result<AppleAIAvailability, AppleAIError> {
        Err(AppleAIError::UnsupportedPlatform(
            "Apple Intelligence is only available on Apple Silicon macOS".into(),
        ))
    }

    pub fn generate(
        _request: AppleAIGenerateRequest,
    ) -> Result<AppleAIGenerateResult, AppleAIError> {
        Err(AppleAIError::UnsupportedPlatform(
            "Apple Intelligence is only available on Apple Silicon macOS".into(),
        ))
    }

    pub fn stream<R: tauri::Runtime>(
        _app: AppHandle<R>,
        _request: AppleAIGenerateRequest,
    ) -> Result<AppleAIStreamStart, AppleAIError> {
        Err(AppleAIError::UnsupportedPlatform(
            "Apple Intelligence is only available on Apple Silicon macOS".into(),
        ))
    }

    pub fn cancel_stream(_stream_id: &str) -> Result<bool, AppleAIError> {
        Err(AppleAIError::UnsupportedPlatform(
            "Apple Intelligence is only available on Apple Silicon macOS".into(),
        ))
    }

    pub fn pcc_check_availability() -> Result<AppleAIAvailability, AppleAIError> {
        Err(AppleAIError::UnsupportedPlatform(
            "Apple Intelligence is only available on Apple Silicon macOS".into(),
        ))
    }

    pub fn context_info(_model: Option<String>) -> Result<AppleAIContextInfo, AppleAIError> {
        Err(AppleAIError::UnsupportedPlatform(
            "Apple Intelligence is only available on Apple Silicon macOS".into(),
        ))
    }

    pub fn supported_languages() -> Result<Vec<String>, AppleAIError> {
        Err(AppleAIError::UnsupportedPlatform(
            "Apple Intelligence is only available on Apple Silicon macOS".into(),
        ))
    }

    pub fn prewarm(_model: Option<String>) -> Result<(), AppleAIError> {
        Err(AppleAIError::UnsupportedPlatform(
            "Apple Intelligence is only available on Apple Silicon macOS".into(),
        ))
    }
}
