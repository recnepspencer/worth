//! A current fact is not a fabricated historical envelope.
use super::CurrentPublicationRoster;
use crate::history::data::CanonicalCommitEnvelope;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) enum PublicationEvidence {
    Canonical(Arc<CanonicalCommitEnvelope>),
    Current(Arc<CurrentPublicationRoster>),
}
