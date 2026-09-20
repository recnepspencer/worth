//! The Scroll owner's staged participation in one settle proposal.
//!
//! A settle carries no reveal requirement: no Focus owner resolves against it
//! and no owner replans for it, so it is a different staged shape from
//! [`super::UiStagedScrollServiceProposal`], which exists to carry exactly that
//! requirement. Collapsing the two would make a settle claim a coordination it
//! never asked for.

#[must_use = "the Scroll owner must acknowledge or discard its staged settle proposal"]
pub(in crate::runtime) struct UiStagedScrollSettleProposal {
    proposal: crate::runtime::session::service_proposal::UiServiceProposalIdentity,
    scope: crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity,
    fact: crate::runtime::session::service_proposal::UiServiceProducedFactReference,
}

impl UiStagedScrollSettleProposal {
    pub(in crate::runtime) fn family_proposal(
        scope: crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity,
    ) -> crate::runtime::session::service_proposal::UiServiceFamilyProposal {
        crate::runtime::session::service_proposal::UiServiceFamilyProposal::scroll(scope)
    }

    pub(in crate::runtime) fn prepare(
        proposal: crate::runtime::session::service_proposal::UiServiceProposalIdentity,
        scope: crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity,
    ) -> Self {
        Self {
            proposal,
            scope,
            fact: crate::runtime::session::service_proposal::UiServiceProducedFactReference::for_scroll_proposal(
                proposal,
                scope,
            ),
        }
    }

    pub(in crate::runtime) const fn proposal(
        &self,
    ) -> crate::runtime::session::service_proposal::UiServiceProposalIdentity {
        self.proposal
    }

    pub(in crate::runtime) const fn scope(
        &self,
    ) -> crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity {
        self.scope
    }

    pub(in crate::runtime) fn family_stage_receipt(
        &self,
    ) -> crate::runtime::session::service_proposal::UiServiceProposalStageReceipt {
        crate::runtime::session::service_proposal::UiServiceProposalStageReceipt::from_family_owner(
            self.proposal,
            crate::capability::UiRuntimeServiceFamily::Scroll,
            self.scope,
            vec![self.fact],
            Vec::new(),
        )
    }
}
