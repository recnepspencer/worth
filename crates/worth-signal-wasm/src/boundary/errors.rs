use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use worth_signal::facade::SignalError;

use crate::runtime::compute_callbacks::ComputeCallbackFailure;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorthSignalJsError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl WorthSignalJsError {
    pub fn deferred(
        code: impl Into<String>,
        message: impl Into<String>,
        context: Option<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            context,
        }
    }

    pub fn callback_deferred(
        code: impl Into<String>,
        message: impl Into<String>,
        context: Option<String>,
    ) -> Self {
        Self::deferred(code, message, context)
    }

    pub fn callback_failure(
        code: impl Into<String>,
        message: impl Into<String>,
        context: Option<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            context,
        }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self {
            code: "invalidInput".to_owned(),
            message: message.into(),
            context: None,
        }
    }

    pub fn restore_token_not_pending(token: &str, maximum_pending_tokens: usize) -> Self {
        Self {
            code: "restoreTokenNotPending".to_owned(),
            message: format!(
                "restore token `{token}` is not pending: it was consumed, discarded, or evicted (a realm keeps its {maximum_pending_tokens} most recent pending exact restore artifacts)"
            ),
            context: None,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: "internal".to_owned(),
            message: message.into(),
            context: None,
        }
    }

    pub fn from_compute_callback_failure(value: ComputeCallbackFailure) -> Self {
        Self {
            code: value
                .code
                .unwrap_or_else(|| "computeCallbackFailure".to_owned()),
            message: value.message,
            context: None,
        }
    }
}

impl From<SignalError> for WorthSignalJsError {
    fn from(value: SignalError) -> Self {
        match value {
            SignalError::InvalidInput { message, context } => Self {
                code: "invalidInput".to_owned(),
                message,
                context,
            },
            SignalError::CycleDetected { .. } => Self {
                code: "cycleDetected".to_owned(),
                message: value.to_string(),
                context: None,
            },
            SignalError::IncompatibleSnapshot { reason } => Self {
                code: "incompatibleSnapshot".to_owned(),
                message: reason,
                context: None,
            },
            SignalError::UnknownBranch { .. } => Self {
                code: "unknownBranch".to_owned(),
                message: value.to_string(),
                context: None,
            },
            SignalError::BranchMergeFailed { message, .. } => Self {
                code: "branchMergeFailed".to_owned(),
                message,
                context: None,
            },
            SignalError::Internal { message, context } => Self {
                code: "internal".to_owned(),
                message,
                context,
            },
            other => Self {
                code: "signalError".to_owned(),
                message: other.to_string(),
                context: None,
            },
        }
    }
}

impl From<WorthSignalJsError> for JsValue {
    fn from(value: WorthSignalJsError) -> Self {
        serde_wasm_bindgen::to_value(&value)
            .unwrap_or_else(|_| JsValue::from_str("worth-signal wasm error"))
    }
}
