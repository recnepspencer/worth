//! Immutable marks occupy one allocation shared by current and retained roots.
use super::MarkState;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

// Prove the persistent marks are shareable at their owning boundary, before
// embedding this exact allocation in a product observation or receipt.
trait MarkBacking: Send + Sync {
    fn marks(&self) -> &MarkState;
}

impl MarkBacking for MarkState {
    fn marks(&self) -> &MarkState {
        self
    }
}

#[derive(Clone)]
pub struct RetainedMarkState(Arc<dyn MarkBacking>);

impl RetainedMarkState {
    pub(in super::super) fn new(state: MarkState) -> Self {
        Self(Arc::new(state))
    }
}

impl Deref for RetainedMarkState {
    type Target = MarkState;

    fn deref(&self) -> &MarkState {
        self.0.marks()
    }
}

impl AsRef<MarkState> for RetainedMarkState {
    fn as_ref(&self) -> &MarkState {
        self
    }
}

impl fmt::Debug for RetainedMarkState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_ref(), formatter)
    }
}
