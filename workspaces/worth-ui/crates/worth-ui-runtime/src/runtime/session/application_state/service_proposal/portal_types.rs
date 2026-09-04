#[must_use = "portal proposal preparation retains compiler occupancy until staged or cancelled"]
pub(crate) struct UiPortalProposalPreparation {
    pub(super) staging: crate::runtime::session::service_proposal::UiServiceProposalStaging,
    pub(super) portal: crate::runtime::portal::UiStagedPortalServiceProposal,
    pub(super) focus: crate::runtime::focus::UiStagedFocusServiceProposal,
    pub(super) scroll: crate::runtime::scroll::UiStagedScrollServiceProposal,
    pub(super) selection: Option<crate::runtime::selection::UiStagedDeclaredSelectionTransition>,
    pub(super) motion: Option<crate::runtime::motion::UiStagedMotionServiceProposal>,
}

#[must_use = "a staged portal proposal must settle with existing publication"]
pub(crate) struct UiStagedPortalProposalTransaction {
    pub(super) batch: crate::runtime::session::service_proposal::UiServiceProposalStagedBatch,
    pub(super) portal: crate::runtime::portal::UiStagedPortalServiceProposal,
    pub(super) focus: crate::runtime::focus::UiStagedFocusServiceProposal,
    pub(super) scroll: crate::runtime::scroll::UiStagedScrollServiceProposal,
    pub(super) staged_reveal: Option<super::UiStagedFocusReveal>,
    pub(super) selection: Option<crate::runtime::selection::UiStagedDeclaredSelectionTransition>,
    pub(super) motion: Option<crate::runtime::motion::UiDerivedMotionServiceProposal>,
    pub(super) prepared_frame: worth_ui_host_contract::UiMountedFrameIdentity,
}

pub(super) struct UiPortalProposalSettlement {
    pub(super) settlement: crate::runtime::session::service_proposal::UiServiceProposalSettlement,
    pub(super) transition: crate::runtime::portal::UiPreparedPortalServiceTransition,
    pub(super) focus: crate::runtime::focus::UiStagedFocusServiceProposal,
    pub(super) scroll: crate::runtime::scroll::UiStagedScrollServiceProposal,
    pub(super) staged_reveal: Option<super::UiStagedFocusReveal>,
    pub(super) selection: Option<crate::runtime::selection::UiStagedDeclaredSelectionTransition>,
    pub(super) motion: Option<crate::runtime::motion::UiDerivedMotionServiceProposal>,
    pub(super) prepared_frame: worth_ui_host_contract::UiMountedFrameIdentity,
    pub(super) publication:
        crate::runtime::session::service_proposal::UiServiceProposalPublicationReceipt,
    pub(super) scope:
        crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity,
}

#[must_use = "indeterminate portal and Focus successors must settle from presentation truth or shutdown"]
pub(crate) struct UiIndeterminatePortalProposalTransaction {
    pub(super) transaction: UiStagedPortalProposalTransaction,
}

#[derive(Debug)]
pub(crate) enum UiPortalProposalPreparationDenial {
    RequestBasis(crate::runtime::session::service_proposal::UiServiceRequestBasisDenial),
    Demand(crate::runtime::session::service_proposal::UiServiceProposalDemandConstructionDenial),
    Preflight(crate::runtime::session::service_proposal::UiServiceProposalPreflightDenial),
    Reservation(crate::runtime::session::service_proposal::UiServiceProposalReservationDenial),
    Staging(crate::runtime::session::service_proposal::UiServiceProposalStagingDenial),
    Publication(crate::runtime::session::service_proposal::UiServiceProposalPublicationDenial),
    Focus(crate::runtime::focus::UiPortalFocusTransitionDenial),
    Scroll(super::UiFocusRevealStagingDenial),
    MissingScrollOwner,
    SelectionMapping(super::super::UiDeclaredSelectionMappingDenial),
    Selection(crate::runtime::selection::UiDeclaredSelectionStagingDenial),
    MotionRequest(crate::runtime::motion::UiMotionTransitionRequestDenial),
    Motion(crate::runtime::motion::UiMotionStagingDenial),
    MountedFrameMismatch,
    /// The compiled batch and the Focus owner's staged reveal disagree about
    /// whether a Scroll owner replanned, or about which owner did.
    RevealRefinementMismatch,
    Coalesced(crate::runtime::session::service_proposal::UiServiceProposalIdentity),
}

impl UiStagedPortalProposalTransaction {
    pub(crate) fn overlay_binding_commit(
        &self,
    ) -> crate::runtime::portal::UiPortalOverlayBindingCommit {
        self.portal.overlay_binding_commit()
    }

    /// The compiled reveal witness must name exactly the Scroll owner whose
    /// staged replan this transaction is about to commit.
    pub(super) fn reveal_refinement_agrees(&self) -> bool {
        let staged_scope = self.staged_reveal.as_ref().map(|_| self.scroll.scope());
        self.batch.reveal_refinement() == staged_scope
    }
}
