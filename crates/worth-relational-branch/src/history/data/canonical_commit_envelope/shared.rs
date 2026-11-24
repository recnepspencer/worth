//! A public read share of the owner's existing canonical allocation.
use super::CanonicalCommitEnvelope;
use serde::Serialize;
use std::{fmt, ops::Deref, sync::Arc};

/// Immutable canonical data shared without exposing its allocation ownership.
/// Cloning retains the same original body and its funding; it allocates no
/// second body or control block and grants no performed-position authority.
#[cfg_attr(not(doctest), doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/facade_examples/shared_canonical_envelope.md")))]
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct SharedCanonicalCommitEnvelope {
    envelope: Arc<CanonicalCommitEnvelope>,
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

/// Operations used by the Relational owners.
pub trait SharedCanonicalCommitEnvelopeOwnerPort: Sized {
    fn from_arc(envelope: Arc<CanonicalCommitEnvelope>) -> Self;
    fn into_arc(self) -> Arc<CanonicalCommitEnvelope>;
}

impl SharedCanonicalCommitEnvelopeOwnerPort for SharedCanonicalCommitEnvelope {
    fn from_arc(envelope: Arc<CanonicalCommitEnvelope>) -> Self {
        Self { envelope }
    }

    fn into_arc(self) -> Arc<CanonicalCommitEnvelope> {
        self.envelope
    }
}
