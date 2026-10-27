//! Exact observation custody, independent of wrappers, payers and caches.
use crate::validation::engine::InvariantObservationIdentity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DerivedSourceAffinity {
    pub(crate) observation: InvariantObservationIdentity,
    pub(crate) committed: bool,
    pub(crate) version: crate::identity::data::VersionId,
    pub(crate) proposal: Option<crate::mvcc::RelationalMutationProposalIdentity>,
}
