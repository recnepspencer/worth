//! Exact view posture shared by physical and logical structural sources.
use super::InvariantObservationIdentity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvariantSourceAffinity {
    pub observation: InvariantObservationIdentity,
    pub committed: bool,
    pub version: worth_relational_contracts::internal_port::identity::data::primitives::VersionId,
    pub proposal:
        Option<crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity>,
}

#[doc(hidden)]
pub trait InvariantSourceAffinityFixturePort {
    fn proposal_for_test(
        &self,
    ) -> &Option<crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity>;
}

impl InvariantSourceAffinityFixturePort for InvariantSourceAffinity {
    fn proposal_for_test(
        &self,
    ) -> &Option<crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity>
    {
        &self.proposal
    }
}
