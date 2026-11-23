//! Opaque application bytes describing one recorded step; never parsed by the framework.
use std::sync::Arc;

/// Descriptive bytes an application attaches to a recorded history step.
/// The framework stores and returns them unchanged. They are process-local,
/// carry no authority, and are bounded by the installed
/// `ApplicationHistoryRetentionResources::maximum_step_metadata_bytes`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryHistoryStepMetadata(Option<Arc<[u8]>>);

impl WorthQueryHistoryStepMetadata {
    /// A step without application metadata.
    pub const fn none() -> Self {
        Self(None)
    }

    pub fn from_bytes(bytes: impl Into<Arc<[u8]>>) -> Self {
        Self(Some(bytes.into()))
    }

    /// The stored bytes; empty when the step carries none.
    pub fn bytes(&self) -> &[u8] {
        self.0.as_deref().unwrap_or(&[])
    }
}
