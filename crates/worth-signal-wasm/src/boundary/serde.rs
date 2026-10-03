use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;
use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::JsValue;

use super::errors::WorthSignalJsError;

pub fn from_js<T>(value: JsValue) -> Result<T, WorthSignalJsError>
where
    T: DeserializeOwned,
{
    serde_wasm_bindgen::from_value(value)
        .map_err(|err| WorthSignalJsError::invalid_input(format!("invalid wasm payload: {err}")))
}

pub fn to_js<T>(value: &T) -> Result<JsValue, WorthSignalJsError>
where
    T: Serialize,
{
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    value.serialize(&serializer).map_err(|err| {
        WorthSignalJsError::internal(format!("failed to serialize wasm value: {err}"))
    })
}

pub fn to_js_structured<T>(value: &T) -> Result<JsValue, WorthSignalJsError>
where
    T: Serialize,
{
    let serializer = serde_wasm_bindgen::Serializer::new();
    value.serialize(&serializer).map_err(|err| {
        WorthSignalJsError::internal(format!("failed to serialize wasm value: {err}"))
    })
}

pub fn to_portable_wire<T>(value: &T) -> Result<String, WorthSignalJsError>
where
    T: Serialize,
{
    // Checkpoint compatibility readers select meanings by their declared fields.
    // Named structs preserve those fields when private runtime layouts differ.
    let bytes = rmp_serde::to_vec_named(value).map_err(|err| {
        WorthSignalJsError::internal(format!("failed to serialize wasm value: {err}"))
    })?;
    Ok(BASE64_STANDARD.encode(bytes))
}

pub fn from_portable_wire<T>(value: &str) -> Result<T, WorthSignalJsError>
where
    T: DeserializeOwned,
{
    let bytes = BASE64_STANDARD
        .decode(value)
        .map_err(|err| WorthSignalJsError::invalid_input(format!("invalid wasm payload: {err}")))?;
    rmp_serde::from_slice(&bytes)
        .map_err(|err| WorthSignalJsError::invalid_input(format!("invalid wasm payload: {err}")))
}

pub fn to_json_wire<T>(value: &T) -> Result<String, WorthSignalJsError>
where
    T: Serialize,
{
    serde_json::to_string(value).map_err(|err| {
        WorthSignalJsError::internal(format!("failed to serialize wasm value: {err}"))
    })
}

pub fn from_json_wire<T>(value: &str) -> Result<T, WorthSignalJsError>
where
    T: DeserializeOwned,
{
    serde_json::from_str(value)
        .map_err(|err| WorthSignalJsError::invalid_input(format!("invalid wasm payload: {err}")))
}
