use super::super::WorthUiApplicationSessionState;
use super::{UiPortalProposalPreparation, UiPortalProposalPreparationDenial};

impl WorthUiApplicationSessionState {
    pub(crate) fn begin_portal_service_proposal(
        &mut self,
        handoff: &crate::runtime::intent_execution::UiIntentConsequenceHandoff,
        transition: crate::runtime::portal::UiPreparedPortalServiceTransition,
        application: crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
        overlay_binding_stage: Option<crate::runtime::portal::UiPortalOverlayBindingStage>,
        declared_selection: Option<crate::runtime::selection::UiDeclaredSelectionBinding>,
        selection_state: Option<&crate::runtime::selection::UiSelectionRuntimeState>,
        motion_state: &mut crate::runtime::motion::UiMotionRuntimeState,
        motion_request: Option<crate::runtime::motion::UiMotionTransitionRequest>,
    ) -> Result<UiPortalProposalPreparation, UiPortalProposalPreparationDenial> {
        let request = crate::runtime::session::service_proposal::UiServiceRequestBasis::<
            crate::runtime::session::service_proposal::UiAdmittedIntentServiceRequestAuthority,
        >::from_intent_consequence(handoff, application)
        .map_err(UiPortalProposalPreparationDenial::RequestBasis)?;
        self.begin_portal_service_proposal_from_request(
            request,
            transition,
            overlay_binding_stage,
            declared_selection.zip(selection_state),
            motion_state,
            motion_request,
        )
    }

    pub(crate) fn begin_portal_dismissal_service_proposal(
        &mut self,
        transition: crate::runtime::portal::UiPreparedPortalServiceTransition,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        application: crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
        motion_state: &mut crate::runtime::motion::UiMotionRuntimeState,
        motion_request: Option<crate::runtime::motion::UiMotionTransitionRequest>,
    ) -> Result<UiPortalProposalPreparation, UiPortalProposalPreparationDenial> {
        let request = crate::runtime::session::service_proposal::UiServiceRequestBasis::<
            crate::runtime::session::service_proposal::UiPortalDismissalServiceRequestAuthority,
        >::from_portal_dismissal(&transition, presentation, application)
        .map_err(UiPortalProposalPreparationDenial::RequestBasis)?;
        self.begin_portal_service_proposal_from_request(
            request,
            transition,
            None,
            None,
            motion_state,
            motion_request,
        )
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn begin_portal_service_proposal_for_certification(
        &mut self,
        transition: crate::runtime::portal::UiPreparedPortalServiceTransition,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        application: crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
        overlay_binding_stage: Option<crate::runtime::portal::UiPortalOverlayBindingStage>,
        motion_state: &mut crate::runtime::motion::UiMotionRuntimeState,
        motion_request: Option<crate::runtime::motion::UiMotionTransitionRequest>,
    ) -> Result<UiPortalProposalPreparation, UiPortalProposalPreparationDenial> {
        let request = crate::runtime::session::service_proposal::UiServiceRequestBasis::<
            crate::runtime::session::service_proposal::UiPortalCertificationServiceRequestAuthority,
        >::from_portal_certification(&transition, presentation, application)
        .map_err(UiPortalProposalPreparationDenial::RequestBasis)?;
        self.begin_portal_service_proposal_from_request(
            request,
            transition,
            overlay_binding_stage,
            None,
            motion_state,
            motion_request,
        )
    }

    pub(crate) fn begin_portal_exit_terminal_service_proposal(
        &mut self,
        transition: crate::runtime::portal::UiPreparedPortalServiceTransition,
        retention: crate::runtime::portal::UiPortalExitRetentionReceipt,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        application: crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
        motion_state: &mut crate::runtime::motion::UiMotionRuntimeState,
    ) -> Result<UiPortalProposalPreparation, UiPortalProposalPreparationDenial> {
        let request = crate::runtime::session::service_proposal::UiServiceRequestBasis::<
            crate::runtime::session::service_proposal::UiPortalExitTerminalServiceRequestAuthority,
        >::from_portal_exit_terminal(
            &transition, retention, presentation, application
        )
        .map_err(UiPortalProposalPreparationDenial::RequestBasis)?;
        self.begin_portal_service_proposal_from_request(
            request,
            transition,
            None,
            None,
            motion_state,
            None,
        )
    }

    fn begin_portal_service_proposal_from_request<Authority>(
        &mut self,
        request: crate::runtime::session::service_proposal::UiServiceRequestBasis<Authority>,
        transition: crate::runtime::portal::UiPreparedPortalServiceTransition,
        overlay_binding_stage: Option<crate::runtime::portal::UiPortalOverlayBindingStage>,
        declared_selection: Option<(
            crate::runtime::selection::UiDeclaredSelectionBinding,
            &crate::runtime::selection::UiSelectionRuntimeState,
        )>,
        motion_state: &mut crate::runtime::motion::UiMotionRuntimeState,
        motion_request: Option<crate::runtime::motion::UiMotionTransitionRequest>,
    ) -> Result<UiPortalProposalPreparation, UiPortalProposalPreparationDenial>
    where
        Authority: crate::runtime::session::service_proposal::UiServiceRequestOriginAuthority,
    {
        let coherence = request.coherence();
        let portal_family =
            crate::runtime::portal::UiStagedPortalServiceProposal::family_proposal(&transition);
        let focus_requirement = crate::runtime::focus::UiPortalFocusRequirement::new(
            portal_family.scope(),
            transition.portal().owner().mounted_instance_identity(),
            transition.opens_portal(),
            transition
                .closed_descendants()
                .iter()
                .map(|portal| {
                    crate::runtime::focus::UiPortalFocusBoundaryIdentity::from_scope(
                        crate::runtime::session::service_proposal::
                            UiServiceProposalOccupancyScopeIdentity::for_mounted_owner(
                                portal.owner().mounted_instance_identity(),
                            ),
                    )
                })
                .collect(),
        );
        let focus_family = crate::runtime::focus::UiStagedFocusServiceProposal::family_proposal(
            &focus_requirement,
        );
        let focus_owner = focus_requirement.owner();
        // The Focus owner may emit exactly one reveal requirement against this
        // portal's successor, and the Scroll owner owns that decision. It is
        // therefore a compiled participant of every portal proposal, not a
        // conditional one: a reveal that no owner staged would be a claim with
        // no occupancy, no stage receipt, and no settlement acknowledgement.
        // Preflight denies with `UnsupportedFamily` if a world reaches here
        // without Scroll support rather than silently dropping the participant.
        let scroll_family = crate::runtime::scroll::UiStagedScrollServiceProposal::family_proposal(
            portal_family.scope(),
        );
        let mut family_proposals = vec![portal_family, focus_family, scroll_family];
        if let Some((selection, _)) = declared_selection.as_ref() {
            family_proposals.push(
                crate::runtime::selection::UiStagedSelectionServiceProposal::family_proposal(
                    selection.action(),
                ),
            );
        }
        if let Some(request) = motion_request.as_ref() {
            family_proposals.push(
                crate::runtime::motion::UiStagedMotionServiceProposal::family_proposal(request),
            );
        }
        let candidate =
            crate::runtime::session::service_proposal::UiServiceProposalCandidate::from_request(
                &request,
                family_proposals,
            )
            .map_err(UiPortalProposalPreparationDenial::Demand)?;
        let proposal = candidate.identity();
        let support = self.app.runtime_service_support();
        let preflighted = self
            .runtime
            .service_proposals
            .preflight(candidate, &coherence, support)
            .map_err(UiPortalProposalPreparationDenial::Preflight)?;
        let reservation = match self
            .runtime
            .service_proposals
            .reserve(preflighted)
            .map_err(UiPortalProposalPreparationDenial::Reservation)?
        {
            crate::runtime::session::service_proposal::UiServiceProposalReservationOutcome::Reserved(
                reservation,
            ) => reservation,
            crate::runtime::session::service_proposal::UiServiceProposalReservationOutcome::Coalesced { incumbent } => {
                return Err(UiPortalProposalPreparationDenial::Coalesced(incumbent))
            }
        };
        let portal = crate::runtime::portal::UiStagedPortalServiceProposal::prepare(
            transition,
            proposal,
            portal_family.scope(),
            overlay_binding_stage,
        );
        let focus = crate::runtime::focus::UiStagedFocusServiceProposal::prepare(
            proposal,
            focus_requirement,
        );
        let scroll = crate::runtime::scroll::UiStagedScrollServiceProposal::prepare(
            proposal,
            portal_family.scope(),
            crate::runtime::session::service_proposal::UiFocusRevealRequirement::new(focus_owner),
        );
        let selection = match declared_selection {
            Some((binding, selection_state)) => {
                let (action, registration) = binding.into_parts();
                match crate::runtime::selection::UiStagedDeclaredSelectionTransition::prepare(
                    proposal,
                    action,
                    registration,
                    selection_state,
                ) {
                    Ok(selection) => Some(selection),
                    Err(denial) => {
                        self.runtime
                            .service_proposals
                            .cancel_before_effect(reservation)
                            .expect("reserved proposal remains cancellable before owner staging");
                        return Err(UiPortalProposalPreparationDenial::Selection(denial));
                    }
                }
            }
            None => None,
        };
        let motion = match motion_request {
            Some(request) => match motion_state.stage(proposal, request) {
                Ok(motion) => Some(motion),
                Err(denial) => {
                    self.runtime
                        .service_proposals
                        .cancel_before_effect(reservation)
                        .expect("reserved proposal remains cancellable before owner staging");
                    return Err(UiPortalProposalPreparationDenial::Motion(denial));
                }
            },
            None => None,
        };
        let mut staging = match self.runtime.service_proposals.begin_staging(reservation) {
            Ok(staging) => staging,
            Err((reservation, denial)) => {
                if let Some(motion) = motion {
                    motion_state.discard_staged(motion);
                }
                // The reservation never entered staging, so its occupancy and
                // cancellation records are released here rather than leaked.
                let _ = self
                    .runtime
                    .service_proposals
                    .shutdown_reservation(reservation);
                return Err(UiPortalProposalPreparationDenial::Staging(denial));
            }
        };
        if let Err(denial) = self
            .runtime
            .service_proposals
            .advance_staging(&mut staging, portal.stage_receipt())
        {
            self.cancel_portal_staging(
                staging,
                &portal,
                &focus,
                &scroll,
                selection.as_ref(),
                motion_state,
                motion,
            );
            return Err(UiPortalProposalPreparationDenial::Staging(denial));
        }
        if let Err(denial) = self
            .runtime
            .service_proposals
            .advance_staging(&mut staging, scroll.family_stage_receipt())
        {
            self.cancel_portal_staging(
                staging,
                &portal,
                &focus,
                &scroll,
                selection.as_ref(),
                motion_state,
                motion,
            );
            return Err(UiPortalProposalPreparationDenial::Staging(denial));
        }
        if let Err(denial) = self
            .runtime
            .service_proposals
            .advance_staging(&mut staging, focus.family_stage_receipt())
        {
            self.cancel_portal_staging(
                staging,
                &portal,
                &focus,
                &scroll,
                selection.as_ref(),
                motion_state,
                motion,
            );
            return Err(UiPortalProposalPreparationDenial::Staging(denial));
        }
        if let Some(receipt) = selection.as_ref().map(
            crate::runtime::selection::UiStagedDeclaredSelectionTransition::family_stage_receipt,
        ) {
            if let Err(denial) = self
                .runtime
                .service_proposals
                .advance_staging(&mut staging, receipt)
            {
                self.cancel_portal_staging(
                    staging,
                    &portal,
                    &focus,
                    &scroll,
                    selection.as_ref(),
                    motion_state,
                    motion,
                );
                return Err(UiPortalProposalPreparationDenial::Staging(denial));
            }
        }
        if let Some(receipt) = motion
            .as_ref()
            .map(crate::runtime::motion::UiStagedMotionServiceProposal::family_stage_receipt)
        {
            if let Err(denial) = self
                .runtime
                .service_proposals
                .advance_staging(&mut staging, receipt)
            {
                self.cancel_portal_staging(
                    staging,
                    &portal,
                    &focus,
                    &scroll,
                    selection.as_ref(),
                    motion_state,
                    motion,
                );
                return Err(UiPortalProposalPreparationDenial::Staging(denial));
            }
        }
        Ok(UiPortalProposalPreparation {
            staging,
            portal,
            focus,
            scroll,
            selection,
            motion,
        })
    }
}
