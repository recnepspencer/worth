//! Opaque identity issued only with a genuine invariant observation.
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) struct InvariantObservationIdentity(Arc<()>);

impl InvariantObservationIdentity {
    pub(super) fn issue() -> Self {
        Self(Arc::new(()))
    }
}

impl PartialEq for InvariantObservationIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for InvariantObservationIdentity {}

impl Hash for InvariantObservationIdentity {
    #[expect(
        clippy::disallowed_methods,
        reason = "Addresses serve live allocation lookup and custody; canonical order uses declared identity"
    )]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}
