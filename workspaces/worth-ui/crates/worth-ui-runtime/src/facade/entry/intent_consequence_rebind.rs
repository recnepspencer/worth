use super::WorthUiActiveApplicationSession;

pub(super) struct WorthUiIntentConsequenceRebindTransfer {
    pub(super) observation: crate::runtime::observation::UiPreparedObservationProgressCommit,
    pub(super) posture: Option<crate::mounting::UiIntentPostureCommit>,
    pub(super) consequence: crate::runtime::intent_execution::UiIntentConsequenceHandoff,
    pub(super) portal_transition: Option<crate::runtime::portal::UiPreparedPortalServiceTransition>,
    pub(super) portal_proposal: Option<crate::runtime::session::UiStagedPortalProposalTransaction>,
    pub(super) query_reference:
        Option<worth_ui_query_binding::WorthUiInstalledQueryBindingReference>,
}

impl WorthUiActiveApplicationSession {
    pub(super) fn prepare_intent_consequence_rebind(
        &mut self,
        plan: crate::runtime::rebind::UiRebindPlan,
        request: crate::runtime::rebind::UiRebindExecutionRequest,
        mut transfer: WorthUiIntentConsequenceRebindTransfer,
    ) -> Result<
        super::intent_consequence_publication::WorthUiPreparedIntentConsequenceRebind<'_>,
        crate::runtime::intent_execution::UiIntentConsequenceStop,
    > {
        if !plan.has_non_source_semantic_proof() {
            return Err(self.retain_intent_consequence_preparation_stop(
                crate::runtime::rebind::UiRebindPreparationDenial::InvalidSemanticProof,
                plan,
                transfer,
            ));
        }
        let reservation = match crate::runtime::rebind::admit_plan(
            &self.rebind,
            crate::runtime::rebind::UiRebindFinalAdmissionBasis::new(
                self.identity,
                self.capabilities().digest().as_u64(),
                self.generation_identity(),
            ),
            &plan,
            request,
        ) {
            Ok(reservation) => reservation,
            Err(denial) => {
                return Err(self.retain_intent_consequence_preparation_stop(denial, plan, transfer))
            }
        };
        let semantic_content = plan.content().clone();
        let generation = self.active_generation_identity();
        let (portal_overlay_revision, portal_overlays) = match transfer.portal_transition.as_ref() {
            Some(transition) => (
                transition.successor_revision(),
                self.portal
                    .as_ref()
                    .expect("a prepared Portal transition retains its installed owner")
                    .mounted_projection_inputs(transition, transition.closes_portal()),
            ),
            None => self.portal.as_ref().map_or_else(
                || (0, Vec::new()),
                |portal| {
                    (
                        portal.revision(),
                        portal.current_mounted_projection_inputs(),
                    )
                },
            ),
        };
        let portal_preparation = match transfer.portal_transition.take() {
            Some(transition) => {
                let declared_selection = match self.selection.as_ref() {
                    Some(selection) => match self.application.declared_selection_for_intent_target(
                        &transfer.consequence,
                        &self.mounted,
                        selection,
                    ) {
                        Ok(selection) => selection,
                        Err(denial) => {
                            drop(reservation);
                            return Err(self.retain_intent_consequence_service_proposal_stop(
                                crate::runtime::session::UiPortalProposalPreparationDenial::SelectionMapping(
                                    denial,
                                ),
                                plan,
                                transfer,
                            ));
                        }
                    },
                    None => None,
                };
                let motion_request = self.prepare_portal_motion_request(&transition).map_err(
                    crate::runtime::session::UiPortalProposalPreparationDenial::MotionRequest,
                );
                let motion_request = match motion_request {
                    Ok(request) => request,
                    Err(denial) => {
                        drop(reservation);
                        return Err(self.retain_intent_consequence_service_proposal_stop(
                            denial, plan, transfer,
                        ));
                    }
                };
                match self.application.begin_portal_service_proposal(
                    &transfer.consequence,
                    transition,
                    generation,
                    declared_selection,
                    self.selection.as_ref(),
                    self.motion
                        .as_mut()
                        .expect("a prepared Portal transition retains Motion installation"),
                    motion_request,
                ) {
                    Ok(preparation) => Some(preparation),
                    Err(denial) => {
                        drop(reservation);
                        return Err(self.retain_intent_consequence_service_proposal_stop(
                            denial, plan, transfer,
                        ));
                    }
                }
            }
            None => None,
        };
        let frame = match self.prepare_intent_consequence_frame(
            semantic_content,
            portal_overlay_revision,
            portal_overlays,
        ) {
            Ok(frame) => frame,
            Err(denial) => {
                if let Some(preparation) = portal_preparation {
                    self.application.cancel_portal_service_proposal_preparation(
                        preparation,
                        self.motion
                            .as_mut()
                            .expect("a staged Portal proposal retains Motion installation"),
                    );
                }
                drop(reservation);
                return Err(self.retain_intent_consequence_preparation_stop(denial, plan, transfer));
            }
        };
        let scroll_incarnation = self.scroll_owner_incarnation();
        transfer.portal_proposal = match portal_preparation {
            Some(preparation) => match self.application.bind_portal_service_proposal_frame(
                preparation,
                &frame,
                &self.mounted,
                self.focus
                    .as_mut()
                    .expect("a staged Portal proposal retains Focus installation"),
                self.scroll.as_ref(),
                scroll_incarnation,
                self.motion
                    .as_mut()
                    .expect("a staged Portal proposal retains Motion installation"),
            ) {
                Ok(proposal) => Some(proposal),
                Err(denial) => {
                    drop((reservation, frame));
                    return Err(self
                        .retain_intent_consequence_service_proposal_stop(denial, plan, transfer));
                }
            },
            None => None,
        };
        Ok(
            super::intent_consequence_publication::WorthUiPreparedIntentConsequenceRebind::new(
                self,
                plan,
                reservation,
                frame,
                transfer,
            ),
        )
    }

    pub(super) fn prepare_intent_consequence_frame(
        &mut self,
        semantic_content: crate::mounting::UiMountedSemanticContentInput,
        portal_overlay_revision: u64,
        portal_overlays: Vec<crate::mounting::UiMountedPortalOverlayProjectionInput>,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        let frame_request = self
            .mounted_frame_request()
            .with_portal_overlays(portal_overlay_revision, portal_overlays);
        let completion = self.execute_framework_turn(|_| {}).map_err(|_| {
            crate::runtime::rebind::UiRebindPreparationDenial::FrameBoundaryUnavailable
        })?;
        let mut execution = completion.into_execution().map_err(|_| {
            crate::runtime::rebind::UiRebindPreparationDenial::FrameBoundaryUnavailable
        })?;
        let theme_values = execution.presentation.theme_values_source();
        execution
            .prepare_mounted_frame_with_content_internal(
                frame_request,
                semantic_content,
                theme_values,
            )
            .map_err(|denial| {
                crate::runtime::rebind::UiRebindPreparationDenial::ContentMountedPreparation(
                    Box::new(denial),
                )
            })
    }

    fn retain_intent_consequence_preparation_stop(
        &mut self,
        denial: crate::runtime::rebind::UiRebindPreparationDenial,
        plan: crate::runtime::rebind::UiRebindPlan,
        mut transfer: WorthUiIntentConsequenceRebindTransfer,
    ) -> crate::runtime::intent_execution::UiIntentConsequenceStop {
        transfer
            .consequence
            .restore_query_from_facts(plan.into_retained_facts());
        self.intent_execution.retain_consequence_handoff(
            transfer.consequence,
            crate::runtime::intent_execution::UiIntentConsequenceStopReason::Preparation(Box::new(
                denial,
            )),
        )
    }

    fn retain_intent_consequence_service_proposal_stop(
        &mut self,
        denial: crate::runtime::session::UiPortalProposalPreparationDenial,
        plan: crate::runtime::rebind::UiRebindPlan,
        mut transfer: WorthUiIntentConsequenceRebindTransfer,
    ) -> crate::runtime::intent_execution::UiIntentConsequenceStop {
        transfer
            .consequence
            .restore_query_from_facts(plan.into_retained_facts());
        self.intent_execution.retain_consequence_handoff(
            transfer.consequence,
            crate::runtime::intent_execution::UiIntentConsequenceStopReason::RuntimeServiceProposal(
                denial.into(),
            ),
        )
    }
}
