#[must_use = "a staged portal proposal must settle with existing publication"]
pub(crate) struct UiStagedPortalServiceProposal {
    transition: super::UiPreparedPortalServiceTransition,
    overlay_binding_stage: Option<super::UiPortalOverlayBindingStage>,
    proposal: crate::runtime::session::service_proposal::UiServiceProposalIdentity,
    scope: crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity,
    mounted_work: crate::runtime::session::service_proposal::UiServiceMountedWorkReference,
}

impl UiStagedPortalServiceProposal {
    pub(in crate::runtime) fn prepare(
        transition: super::UiPreparedPortalServiceTransition,
        proposal: crate::runtime::session::service_proposal::UiServiceProposalIdentity,
        scope: crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity,
        overlay_binding_stage: Option<super::UiPortalOverlayBindingStage>,
    ) -> Self {
        Self {
            transition,
            overlay_binding_stage,
            proposal,
            scope,
            mounted_work: crate::runtime::session::service_proposal::UiServiceMountedWorkReference::for_portal_proposal(
                proposal,
                scope,
            ),
        }
    }

    pub(in crate::runtime) fn family_proposal(
        transition: &super::UiPreparedPortalServiceTransition,
    ) -> crate::runtime::session::service_proposal::UiServiceFamilyProposal {
        let scope = crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity::for_mounted_owner(
            transition.request().portal().owner().mounted_instance_identity(),
        );
        crate::runtime::session::service_proposal::UiServiceFamilyProposal::portal(scope)
    }

    pub(in crate::runtime) const fn scope(
        &self,
    ) -> crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity {
        self.scope
    }

    pub(in crate::runtime) const fn transition(&self) -> &super::UiPreparedPortalServiceTransition {
        &self.transition
    }

    pub(in crate::runtime) const fn overlay_binding_stage(
        &self,
    ) -> Option<&super::UiPortalOverlayBindingStage> {
        self.overlay_binding_stage.as_ref()
    }

    pub(crate) fn overlay_binding_commit(&self) -> super::UiPortalOverlayBindingCommit {
        super::UiPortalOverlayBindingCommit::from_transition(
            &self.transition,
            self.overlay_binding_stage.clone(),
        )
    }

    pub(in crate::runtime) fn stage_receipt(
        &self,
    ) -> crate::runtime::session::service_proposal::UiServiceProposalStageReceipt {
        crate::runtime::session::service_proposal::UiServiceProposalStageReceipt::from_family_owner(
            self.proposal,
            crate::capability::UiRuntimeServiceFamily::Portal,
            self.scope,
            Vec::new(),
            vec![self.mounted_work],
        )
    }

    pub(in crate::runtime) fn into_transition(self) -> super::UiPreparedPortalServiceTransition {
        self.transition
    }
}
