//! Owning evidence that an exact publication may be retried without a second effect.
use crate::AuthorityMarker;

/// The runtime seals `Original` and constructs it only from its medium's actual
/// proven-no-effect answer. Concrete retry doors demand their own specialization;
/// a caller's unrelated marker cannot authorize an owner's retry.
///
/// Unlike a zero-sized authority witness, this proof retains the original
/// runtime/subject binding supplied by its owning runtime.
#[derive(Debug)]
pub struct ProvenNoEffectPublication<Original: AuthorityMarker> {
    original: Original,
}
impl<Original: AuthorityMarker> ProvenNoEffectPublication<Original> {
    pub fn from_owner(original: Original) -> Self {
        Self { original }
    }
    pub fn original(&self) -> &Original {
        &self.original
    }
}
