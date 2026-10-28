//! Exact view posture shared by physical and logical structural sources.
use super::InvariantObservationIdentity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InvariantSourceAffinity {
    pub(crate) observation: InvariantObservationIdentity,
    pub(crate) committed: bool,
    pub(crate) version: crate::identity::data::VersionId,
    pub(crate) proposal: Option<crate::mvcc::RelationalMutationProposalIdentity>,
}
