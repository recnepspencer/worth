use super::super::WorthUiApplicationSessionState;
use super::{
    UiPortalProposalPreparation, UiPortalProposalPreparationDenial,
    UiStagedPortalProposalTransaction,
};

impl WorthUiApplicationSessionState {
    pub(crate) fn bind_portal_service_proposal_frame(
        &mut self,
        mut preparation: UiPortalProposalPreparation,
        frame: &crate::mounting::UiPreparedMountedFrame,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        focus: &mut crate::runtime::focus::UiFocusRuntimeState,
        scroll_state: Option<&crate::runtime::scroll::UiScrollRuntimeState>,
        surface_incarnation: crate::runtime::scroll::UiScrollOwnerIncarnation,
        motion_state: &mut crate::runtime::motion::UiMotionRuntimeState,
    ) -> Result<UiStagedPortalProposalTransaction, UiPortalProposalPreparationDenial> {
        if let Err(denial) = focus.stage_portal_proposal(
            &preparation.focus,
            mounted.candidate_focus_participation_snapshot(frame),
        ) {
            self.cancel_portal_staging(
                preparation.staging,
                &preparation.portal,
                &preparation.focus,
                &preparation.scroll,
                preparation.selection.as_ref(),
                motion_state,
                preparation.motion,
            );
            return Err(UiPortalProposalPreparationDenial::Focus(denial));
        }
        let reveal_requirement =
            match focus.staged_portal_reveal_requirement(preparation.focus.proposal()) {
                Ok(requirement) => requirement,
                Err(denial) => {
                    focus
                        .discard_portal_proposal(preparation.focus.proposal())
                        .expect("Focus owner discards the exact malformed reveal candidate");
                    self.cancel_portal_staging(
                        preparation.staging,
                        &preparation.portal,
                        &preparation.focus,
                        &preparation.scroll,
                        preparation.selection.as_ref(),
                        motion_state,
                        preparation.motion,
                    );
                    return Err(UiPortalProposalPreparationDenial::Focus(denial));
                }
            };
        let staged_reveal = match reveal_requirement {
            Some(mut requirement) => {
                // The Scroll participant is compiled into every portal proposal,
                // so a reveal requirement always has an owner to carry it. Only a
                // missing runtime owner is still a typed denial.
                if let Some(anchor) = preparation
                    .selection
                    .as_ref()
                    .and_then(|selection| selection.scroll_anchor_key_for(requirement.target()))
                {
                    requirement = requirement.with_application_item_anchor(anchor);
                }
                preparation.scroll = preparation.scroll.with_requirement(requirement);
                let Some(scroll_state) = scroll_state else {
                    focus
                        .discard_portal_proposal(preparation.focus.proposal())
                        .expect("Focus owner discards the proposal missing its Scroll owner");
                    self.cancel_portal_staging(
                        preparation.staging,
                        &preparation.portal,
                        &preparation.focus,
                        &preparation.scroll,
                        preparation.selection.as_ref(),
                        motion_state,
                        preparation.motion,
                    );
                    return Err(UiPortalProposalPreparationDenial::MissingScrollOwner);
                };
                match self.stage_focus_reveal(
                    preparation.scroll.requirement(),
                    frame,
                    mounted,
                    scroll_state,
                    surface_incarnation,
                ) {
                    Ok(reveal) => reveal,
                    Err(denial) => {
                        focus
                            .discard_portal_proposal(preparation.focus.proposal())
                            .expect("Focus owner discards the exact failed reveal candidate");
                        self.cancel_portal_staging(
                            preparation.staging,
                            &preparation.portal,
                            &preparation.focus,
                            &preparation.scroll,
                            preparation.selection.as_ref(),
                            motion_state,
                            preparation.motion,
                        );
                        return Err(UiPortalProposalPreparationDenial::Scroll(denial));
                    }
                }
            }
            None => None,
        };
        let receipt = crate::runtime::session::service_proposal::UiServiceProposalStageReceipt::existing_preparation(
            preparation.staging.identity(),
        );
        if let Err(denial) = self
            .runtime
            .service_proposals
            .advance_staging(&mut preparation.staging, receipt)
        {
            focus
                .discard_portal_proposal(preparation.focus.proposal())
                .expect("Focus owner discards the exact proposal staged above");
            self.cancel_portal_staging(
                preparation.staging,
                &preparation.portal,
                &preparation.focus,
                &preparation.scroll,
                preparation.selection.as_ref(),
                motion_state,
                preparation.motion,
            );
            return Err(UiPortalProposalPreparationDenial::Staging(denial));
        }
        let revealed_scroll_scope = staged_reveal.as_ref().map(|_| preparation.scroll.scope());
        if let Err(denial) = self.runtime.service_proposals.advance_staging(
            &mut preparation.staging,
            preparation.focus.resolution_receipt(revealed_scroll_scope),
        ) {
            focus
                .discard_portal_proposal(preparation.focus.proposal())
                .expect("Focus owner discards the exact resolved proposal");
            self.cancel_portal_staging(
                preparation.staging,
                &preparation.portal,
                &preparation.focus,
                &preparation.scroll,
                preparation.selection.as_ref(),
                motion_state,
                preparation.motion,
            );
            return Err(UiPortalProposalPreparationDenial::Staging(denial));
        }
        let motion = preparation
            .motion
            .map(|motion| motion_state.derive(motion, frame.canonical_core().frame()));
        if let Some(receipt) = motion
            .as_ref()
            .map(crate::runtime::motion::UiDerivedMotionServiceProposal::derivation_receipt)
        {
            if let Err(denial) = self
                .runtime
                .service_proposals
                .advance_staging(&mut preparation.staging, receipt)
            {
                focus
                    .discard_portal_proposal(preparation.focus.proposal())
                    .expect("Focus owner discards the exact Motion derivation candidate");
                if let Some(motion) = motion {
                    motion_state.discard_derived(motion);
                }
                self.cancel_portal_staging(
                    preparation.staging,
                    &preparation.portal,
                    &preparation.focus,
                    &preparation.scroll,
                    preparation.selection.as_ref(),
                    motion_state,
                    None,
                );
                return Err(UiPortalProposalPreparationDenial::Staging(denial));
            }
        }
        match self
            .runtime
            .service_proposals
            .finish_staging(preparation.staging)
        {
            Ok(batch) => Ok(UiStagedPortalProposalTransaction {
                batch,
                portal: preparation.portal,
                focus: preparation.focus,
                scroll: preparation.scroll,
                staged_reveal,
                selection: preparation.selection,
                motion,
                prepared_frame: frame.canonical_core().frame(),
            }),
            Err((staging, denial)) => {
                focus
                    .discard_portal_proposal(preparation.focus.proposal())
                    .expect("Focus owner discards the exact complete staging candidate");
                if let Some(motion) = motion {
                    motion_state.discard_derived(motion);
                }
                self.cancel_portal_staging(
                    staging,
                    &preparation.portal,
                    &preparation.focus,
                    &preparation.scroll,
                    preparation.selection.as_ref(),
                    motion_state,
                    None,
                );
                Err(UiPortalProposalPreparationDenial::Staging(denial))
            }
        }
    }
}
