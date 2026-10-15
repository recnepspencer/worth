//! Immutable typed input ownership, independent of the runtime which stores it.
use crate::native_input_capture::NativeImmutableInput;
use std::ops::Deref;
use std::sync::Arc;
enum Backing<Value> {
    Value(Arc<Value>),
    Owned(Arc<dyn NativeImmutableInput<Value> + Send + Sync>),
}
/// Shared immutable input; cloning preserves the SAME backend's final-owner custody.
/// There is no extraction of Value or a bare Arc that could discard that backend.
pub struct ApplicationSharedInput<Value> {
    backing: Backing<Value>,
}
impl<Value> ApplicationSharedInput<Value> {
    pub(super) fn from_value(value: Value) -> Self {
        Self {
            backing: Backing::Value(Arc::new(value)),
        }
    }
    pub(super) fn from_owned<Owner: NativeImmutableInput<Value> + Send + Sync + 'static>(
        owner: Owner,
    ) -> Self {
        Self {
            backing: Backing::Owned(Arc::new(owner)),
        }
    }
}
impl<Value> Clone for ApplicationSharedInput<Value> {
    fn clone(&self) -> Self {
        Self {
            backing: match &self.backing {
                Backing::Value(value) => Backing::Value(Arc::clone(value)),
                Backing::Owned(owner) => Backing::Owned(Arc::clone(owner)),
            },
        }
    }
}
impl<Value> AsRef<Value> for ApplicationSharedInput<Value> {
    fn as_ref(&self) -> &Value {
        match &self.backing {
            Backing::Value(value) => value.as_ref(),
            Backing::Owned(owner) => owner.value(),
        }
    }
}
impl<Value> Deref for ApplicationSharedInput<Value> {
    type Target = Value;
    fn deref(&self) -> &Value {
        self.as_ref()
    }
}
