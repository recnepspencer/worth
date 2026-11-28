//! Intrinsic source admission follows this same private shared text owner.
use crate::admitted_shared::AdmittedShared;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SchemaChangeText(AdmittedShared<str>);

impl std::fmt::Debug for SchemaChangeText {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, formatter)
    }
}

impl From<Arc<str>> for SchemaChangeText {
    fn from(value: Arc<str>) -> Self {
        Self(AdmittedShared::new(value))
    }
}
impl std::ops::Deref for SchemaChangeText {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_ref()
    }
}
impl AsRef<str> for SchemaChangeText {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl From<&str> for SchemaChangeText {
    fn from(value: &str) -> Self {
        Self::from(Arc::<str>::from(value))
    }
}
impl From<String> for SchemaChangeText {
    fn from(value: String) -> Self {
        Self::from(Arc::<str>::from(value))
    }
}
impl std::fmt::Display for SchemaChangeText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&**self, f)
    }
}
impl std::borrow::Borrow<str> for SchemaChangeText {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

/// Operations used by the Relational owners.
pub trait SchemaChangeTextOwnerPort: Sized {
    fn control_bytes() -> Option<u64>;
    fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>>;
}

impl SchemaChangeTextOwnerPort for SchemaChangeText {
    fn control_bytes() -> Option<u64> {
        AdmittedShared::<str>::control_bytes()
    }

    fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>> {
        self.0.retain_decoded_memory(memory)
    }
}
