use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::AppHandle;

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppleAIMessage {
    pub role: String,
    pub content: Option<String>,
    pub name: Option<String>,
    pub tool_call_id: Option<String>,
    pub tool_calls: Option<Vec<AppleAIToolCall>>,
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
    pub temperature: Option<f64>,
    pub max_tokens: Option<i32>,
    pub stop_after_tool_calls: Option<bool>,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppleAIGenerateResult {
    pub text: String,
    pub tool_calls: Option<Vec<AppleAIToolCall>>,
    pub object: Option<serde_json::Value>,
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
    #[serde(rename = "tool-call")]
    ToolCall {
        tool_call_id: String,
        tool_name: String,
        args: serde_json::Value,
    },
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

#[tauri::command]
pub fn apple_ai_check_availability() -> Result<AppleAIAvailability, AppleAIError> {
    native::check_availability()
}

#[tauri::command]
pub fn apple_ai_generate(
    request: AppleAIGenerateRequest,
) -> Result<AppleAIGenerateResult, AppleAIError> {
    native::generate(request)
}

#[tauri::command]
pub fn apple_ai_stream(
    app: AppHandle,
    request: AppleAIGenerateRequest,
) -> Result<AppleAIStreamStart, AppleAIError> {
    native::stream(app, request)
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod native {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, OnceLock,
    };
    use tauri::Emitter;

    #[link(name = "appleai")]
    unsafe extern "C" {
        fn apple_ai_init() -> bool;
        fn apple_ai_check_availability() -> i32;
        fn apple_ai_get_availability_reason() -> *mut std::os::raw::c_char;
        fn apple_ai_free_string(ptr: *mut std::os::raw::c_char);

        fn apple_ai_register_tool_callback(cb: Option<extern "C" fn(u64, *const std::os::raw::c_char)>);
        fn apple_ai_tool_result_callback(tool_id: u64, result_json: *const std::os::raw::c_char);

        fn apple_ai_generate_unified(
            messages_json: *const std::os::raw::c_char,
            tools_json: *const std::os::raw::c_char,
            schema_json: *const std::os::raw::c_char,
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
    static TOOL_NAME_MAP: OnceLock<Mutex<std::collections::HashMap<u64, String>>> =
        OnceLock::new();
    static STREAM_STATE: OnceLock<Mutex<Option<StreamState>>> = OnceLock::new();

    struct StreamState {
        app_handle: AppHandle,
        event_name: String,
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

    pub fn generate(request: AppleAIGenerateRequest) -> Result<AppleAIGenerateResult, AppleAIError> {
        ensure_initialized()?;

        let messages_json = serde_json::to_string(&request.messages)
            .map_err(|e| AppleAIError::InvalidPayload(e.to_string()))?;
        let tools_json = serialize_tools(&request.tools)?;
        let schema_json = request
            .schema
            .as_ref()
            .map(|value| serde_json::to_string(value))
            .transpose()
            .map_err(|e| AppleAIError::InvalidPayload(e.to_string()))?;

        let c_messages = CString::new(messages_json)
            .map_err(|_| AppleAIError::InvalidPayload("Messages contained null byte".into()))?;
        let c_tools = tools_json
            .map(|tools| CString::new(tools))
            .transpose()
            .map_err(|_| AppleAIError::InvalidPayload("Tools contained null byte".into()))?;
        let c_schema = schema_json
            .map(|schema| CString::new(schema))
            .transpose()
            .map_err(|_| AppleAIError::InvalidPayload("Schema contained null byte".into()))?;

        if request.tools.as_ref().map_or(false, |t| !t.is_empty()) {
            register_tool_callback();
        }

        let result_ptr = unsafe {
            apple_ai_generate_unified(
                c_messages.as_ptr(),
                c_tools.as_ref().map_or(std::ptr::null(), |value| value.as_ptr()),
                c_schema.as_ref().map_or(std::ptr::null(), |value| value.as_ptr()),
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

        Ok(AppleAIGenerateResult {
            text,
            tool_calls,
            object,
        })
    }

    pub fn stream(
        app: AppHandle,
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

        let state = StreamState {
            app_handle: app.clone(),
            event_name: event_name.clone(),
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
            .map(|value| serde_json::to_string(value))
            .transpose()
            .map_err(|e| AppleAIError::InvalidPayload(e.to_string()))?;

        let c_messages = CString::new(messages_json)
            .map_err(|_| AppleAIError::InvalidPayload("Messages contained null byte".into()))?;
        let c_tools = tools_json
            .map(|tools| CString::new(tools))
            .transpose()
            .map_err(|_| AppleAIError::InvalidPayload("Tools contained null byte".into()))?;
        let c_schema = schema_json
            .map(|schema| CString::new(schema))
            .transpose()
            .map_err(|_| AppleAIError::InvalidPayload("Schema contained null byte".into()))?;

        if request.tools.as_ref().map_or(false, |t| !t.is_empty()) {
            register_tool_callback();
        }

        std::thread::spawn(move || unsafe {
            apple_ai_generate_unified(
                c_messages.as_ptr(),
                c_tools.as_ref().map_or(std::ptr::null(), |value| value.as_ptr()),
                c_schema.as_ref().map_or(std::ptr::null(), |value| value.as_ptr()),
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
            .and_then(|map| map.lock().ok().and_then(|guard| guard.get(&tool_id).cloned()))
            .unwrap_or_else(|| format!("tool-{tool_id}"));

        if let Some(store) = TOOL_CALLS.get() {
            store.lock().unwrap().push((call_id, tool_name, args.clone()));
        }

        let result = CString::new("{}").unwrap();
        unsafe { apple_ai_tool_result_callback(tool_id, result.as_ptr()) };
    }

    const ERROR_SENTINEL: u8 = 0x02;

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
        if bytes.first() == Some(&ERROR_SENTINEL) {
            let message = String::from_utf8_lossy(&bytes[1..]).into_owned();
            emit_event(state, AppleAIStreamEvent::Error { message });
            STREAM_ACTIVE.store(false, Ordering::SeqCst);
            *guard = None;
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
        let _ = state.app_handle.emit(&state.event_name, event);
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

    pub fn generate(_request: AppleAIGenerateRequest) -> Result<AppleAIGenerateResult, AppleAIError> {
        Err(AppleAIError::UnsupportedPlatform(
            "Apple Intelligence is only available on Apple Silicon macOS".into(),
        ))
    }

    pub fn stream(
        _app: AppHandle,
        _request: AppleAIGenerateRequest,
    ) -> Result<AppleAIStreamStart, AppleAIError> {
        Err(AppleAIError::UnsupportedPlatform(
            "Apple Intelligence is only available on Apple Silicon macOS".into(),
        ))
    }
}