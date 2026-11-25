//! A current fact is not a fabricated historical envelope.
use crate::history::data::canonical_commit_envelope::shared::SharedCanonicalCommitEnvelopeOwnerPort as _;
use crate::history::data::{CanonicalCommitEnvelope, SharedCanonicalCommitEnvelope};
use crate::publication::ownership::read::roster::CurrentPublicationRoster;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum PublicationEvidence {
    Canonical(SharedCanonicalCommitEnvelope),
    Current(Arc<CurrentPublicationRoster>),
}

impl PublicationEvidence {
    pub fn canonical(envelope: Arc<CanonicalCommitEnvelope>) -> Self {
        Self::Canonical(SharedCanonicalCommitEnvelope::from_arc(envelope))
    }
}
