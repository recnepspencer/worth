//! A current fact is not a fabricated historical envelope.
use crate::history::data::CanonicalCommitEnvelope;
use crate::publication::ownership::read::roster::CurrentPublicationRoster;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) enum PublicationEvidence {
    Canonical(Arc<CanonicalCommitEnvelope>),
    Current(Arc<CurrentPublicationRoster>),
}
