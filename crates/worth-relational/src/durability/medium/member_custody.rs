//! Provider-owned placement retention, independent of payload admission.
use super::RelationalDurableColdSource;
use std::any::Any;
use std::sync::{Arc, Weak};

mod retirement;
pub use retirement::{
    RelationalDurableMemberRetirement, RelationalDurableMemberRetirementObservation,
    RelationalDurableMemberRetirementRefusal, RetiredRelationalDurableMemberCustody,
};

struct OriginalMemberCustody {
    owner: Box<dyn Any + Send + Sync>,
    source: Option<Weak<dyn RelationalDurableColdSource>>,
}

/// Keeps the provider's selected member placement alive until the last clone
/// drops. This does not verify bytes or authorize record reads or publication.
#[derive(Clone)]
pub struct RelationalDurableMemberCustody {
    owner: Arc<OriginalMemberCustody>,
}

impl RelationalDurableMemberCustody {
    /// The provider's owner releases its placement obligation on Drop. An
    /// immutable source may use `()` because its source handle already retains
    /// every addressed byte for the handle's entire lifetime.
    pub fn new(owner: impl Send + Sync + 'static) -> Self {
        Self {
            owner: Arc::new(OriginalMemberCustody {
                owner: Box::new(owner),
                source: None,
            }),
        }
    }

    /// Bind the original provider payload to its actual issuing source. The
    /// weak identity keeps its allocation distinct without keeping it serving.
    /// Boxed payload custody adds one allocation and one
    /// indirection to the former unsized Arc representation.
    pub fn new_from_source(
        owner: impl Send + Sync + 'static,
        source: Weak<dyn RelationalDurableColdSource>,
    ) -> Self {
        Self {
            owner: Arc::new(OriginalMemberCustody {
                owner: Box::new(owner),
                source: Some(source),
            }),
        }
    }
}

impl std::fmt::Debug for RelationalDurableMemberCustody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelationalDurableMemberCustody")
            .finish_non_exhaustive()
    }
}
