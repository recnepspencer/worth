//! An already-owned provider refusal crosses Native without becoming unavailable.
use std::{error::Error, fmt, sync::Arc};

/// Diagnostic custody only. No source audience, root availability or publication
/// permission follows from this failure or from a concrete provider type.
#[derive(Clone, Debug)]
pub struct ManagedIndexSourceProviderFailure {
    cause: Arc<dyn Error + Send + Sync>,
}
impl ManagedIndexSourceProviderFailure {
    /// The producing resource owns allocation and the original typed cause.
    pub fn from_owned(cause: Arc<dyn Error + Send + Sync>) -> Self {
        Self { cause }
    }
    pub fn cause(&self) -> &(dyn Error + Send + Sync + 'static) {
        self.cause.as_ref()
    }
}
impl PartialEq for ManagedIndexSourceProviderFailure {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.cause, &other.cause)
    }
}
impl Eq for ManagedIndexSourceProviderFailure {}
impl fmt::Display for ManagedIndexSourceProviderFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.cause, formatter)
    }
}
impl Error for ManagedIndexSourceProviderFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.cause.as_ref())
    }
}
