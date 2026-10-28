//! Provider-owned placement retention, independent of payload admission.
use std::sync::Arc;

/// Keeps the provider's selected member placement alive until the last clone
/// drops. This does not verify bytes or authorize record reads or publication.
#[derive(Clone)]
pub struct RelationalDurableMemberCustody {
    _owner: Arc<dyn std::any::Any + Send + Sync>,
}

impl RelationalDurableMemberCustody {
    /// The provider's owner releases its placement obligation on Drop. An
    /// immutable source may use `()` because its source handle already retains
    /// every addressed byte for the handle's entire lifetime.
    pub fn new(owner: impl Send + Sync + 'static) -> Self {
        Self {
            _owner: Arc::new(owner),
        }
    }
}

impl std::fmt::Debug for RelationalDurableMemberCustody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelationalDurableMemberCustody")
            .finish_non_exhaustive()
    }
}
