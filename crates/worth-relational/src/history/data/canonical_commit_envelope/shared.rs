//! A public read share of the owner's existing canonical allocation.
use super::CanonicalCommitEnvelope;
use serde::Serialize;
use std::{fmt, ops::Deref, sync::Arc};

/// Immutable canonical data shared without exposing its allocation ownership.
/// Cloning retains the same original body and its funding; it allocates no
/// second body or control block and grants no performed-position authority.
///
/// ```
/// use worth_relational::facade::history::{CanonicalCommitEnvelope, SharedCanonicalCommitEnvelope};
/// fn inspect(value: &SharedCanonicalCommitEnvelope) {
///     let shared = value.clone();
///     let body: &CanonicalCommitEnvelope = shared.as_ref();
///     assert_eq!(body, &**value);
/// }
/// ```
///
/// ```compile_fail
/// use worth_relational::facade::history::SharedCanonicalCommitEnvelope;
/// fn cannot_retain_a_weak_allocation(value: &SharedCanonicalCommitEnvelope) {
///     let _ = std::sync::Arc::downgrade(value);
/// }
/// ```
///
/// ```compile_fail
/// use worth_relational::facade::history::SharedCanonicalCommitEnvelope;
/// fn cannot_extract_the_owner(value: SharedCanonicalCommitEnvelope) {
///     let _ = value.into_arc();
/// }
/// ```
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct SharedCanonicalCommitEnvelope {
    envelope: Arc<CanonicalCommitEnvelope>,
}
impl SharedCanonicalCommitEnvelope {
    pub(crate) fn from_arc(envelope: Arc<CanonicalCommitEnvelope>) -> Self {
        Self { envelope }
    }
    pub(crate) fn into_arc(self) -> Arc<CanonicalCommitEnvelope> {
        self.envelope
    }
}
impl AsRef<CanonicalCommitEnvelope> for SharedCanonicalCommitEnvelope {
    fn as_ref(&self) -> &CanonicalCommitEnvelope {
        &self.envelope
    }
}
impl Deref for SharedCanonicalCommitEnvelope {
    type Target = CanonicalCommitEnvelope;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
impl fmt::Debug for SharedCanonicalCommitEnvelope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.envelope.fmt(formatter)
    }
}
