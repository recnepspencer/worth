use super::{
    WorthUiActiveApplicationSession, WorthUiMountedReplacementPreparationOutcome,
    WorthUiPreparedApplicationReplacement,
};

pub(crate) struct WorthUiPreparedEvidenceOnlyApplicationRebind<'session> {
    session: &'session mut WorthUiActiveApplicationSession,
    successor_authority:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    appearance_succession: Option<super::UiPreparedAppearanceGenerationSuccession>,
    _admitted_candidate: crate::runtime::WorthUiAdmittedReplacementCandidate,
    _comparison: crate::runtime::WorthUiRuntimeArtifactComparison,
}

impl<'session> WorthUiPreparedEvidenceOnlyApplicationRebind<'session> {
    fn new(
        session: &'session mut WorthUiActiveApplicationSession,
        succession: crate::runtime::observation::UiAuthoredSourceSuccession,
    ) -> Result<Self, crate::runtime::rebind::UiRebindPreparationDenial> {
        let crate::runtime::observation::UiAuthoredSourceSuccession::EvidenceOnly {
            successor_authority,
            admitted_candidate,
            comparison,
        } = succession
        else {
            return Err(crate::runtime::rebind::UiRebindPreparationDenial::InvalidSemanticProof);
        };
        let predecessor = session.active_generation_identity();
        let successor = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
            session.session_identity(),
            successor_authority.generation_identity(),
        );
        let generation_succession = crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationSuccession::new(
                predecessor.prepared_generation().clone(),
                successor.prepared_generation().clone(),
            );
        let appearance_succession = session
            .prepare_appearance_generation_succession(&generation_succession)
            .map(Some)
            .map_err(|denial| match denial {
                super::UiAppearanceGenerationSuccessionDenial::Theme(denial) => {
                    crate::runtime::rebind::UiRebindPreparationDenial::AppearanceThemeSuccession(
                        denial,
                    )
                }
                super::UiAppearanceGenerationSuccessionDenial::Inspection(denial) => {
                    crate::runtime::rebind::UiRebindPreparationDenial::AppearanceInspectionSuccession(
                        denial,
                    )
                }
            })?;
        Ok(Self {
            session,
            successor_authority,
            appearance_succession,
            _admitted_candidate: admitted_candidate,
            _comparison: comparison,
        })
    }

    pub(crate) fn commit(
        self,
    ) -> (
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    ) {
        let Self {
            session,
            successor_authority,
            appearance_succession,
            _admitted_candidate: _,
            _comparison: _,
        } = self;
        let generations = session
            .application
            .commit_evidence_only_rebind(successor_authority);
        if let Some(appearance_succession) = appearance_succession {
            session.commit_appearance_generation_succession(appearance_succession);
        }
        generations
    }

    pub(crate) fn generation_identity(
        &self,
    ) -> &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity
    {
        self.successor_authority.generation_identity()
    }
}

impl WorthUiActiveApplicationSession {
    pub fn prepare_rebind(
        &mut self,
        mut plan: crate::runtime::rebind::UiRebindPlan,
        request: crate::runtime::rebind::UiRebindExecutionRequest,
    ) -> Result<
        crate::runtime::rebind::UiPreparedRebind<'_>,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        let reservation = crate::runtime::rebind::admit_plan(
            &self.rebind,
            crate::runtime::rebind::UiRebindFinalAdmissionBasis::new(
                self.identity,
                self.capabilities().digest().as_u64(),
                self.generation_identity(),
            ),
            &plan,
            request,
        )?;
        match plan.take_semantic_proof() {
            crate::runtime::rebind::UiRebindSemanticProof::Changed(changed) => {
                self.prepare_changed_rebind(plan, reservation, changed)
            }
            crate::runtime::rebind::UiRebindSemanticProof::AuthoredContent(content) => {
                self.prepare_authored_content_rebind(plan, reservation, *content)
            }
            crate::runtime::rebind::UiRebindSemanticProof::EvidenceOnly(succession) => {
                let prepared =
                    WorthUiPreparedEvidenceOnlyApplicationRebind::new(self, *succession)?;
                crate::runtime::rebind::UiPreparedRebind::evidence_only(plan, reservation, prepared)
            }
            crate::runtime::rebind::UiRebindSemanticProof::NonSource => {
                self.prepare_content_rebind(plan, reservation)
            }
            crate::runtime::rebind::UiRebindSemanticProof::Transferred => {
                Err(crate::runtime::rebind::UiRebindPreparationDenial::InvalidSemanticProof)
            }
        }
    }

    fn prepare_content_rebind(
        &mut self,
        plan: crate::runtime::rebind::UiRebindPlan,
        reservation: crate::runtime::rebind::UiRebindReservation,
    ) -> Result<
        crate::runtime::rebind::UiPreparedRebind<'_>,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        let mut semantic_content = plan.content().clone();
        let presentation = self.presentation.project().map_err(|denial| {
            crate::runtime::rebind::UiRebindPreparationDenial::ContentMountedPreparation(Box::new(
                denial,
            ))
        })?;
        semantic_content
            .merge_application_presentation(presentation.content())
            .map_err(|denial| {
                crate::runtime::rebind::UiRebindPreparationDenial::ContentMountedPreparation(
                    Box::new(crate::mounting::UiMountedFramePreparationDenial::Projection(denial)),
                )
            })?;
        let frame_request = self.current_portal_rebind_frame_request();
        let mut frame = {
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
                })?
        };
        frame.set_application_text_publication(presentation.text_publication(), &self.mounted);
        let content =
            Box::new(crate::facade::entry::WorthUiPreparedMountedContentRebind::new(self, frame));
        Ok(crate::runtime::rebind::UiPreparedRebind::content(
            plan,
            reservation,
            content,
        ))
    }

    fn prepare_authored_content_rebind(
        &mut self,
        plan: crate::runtime::rebind::UiRebindPlan,
        reservation: crate::runtime::rebind::UiRebindReservation,
        content: crate::runtime::rebind::UiAuthoredContentRebindSemanticProof,
    ) -> Result<
        crate::runtime::rebind::UiPreparedRebind<'_>,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        let semantic_content = plan.content().clone();
        let frame_request = self.current_portal_rebind_frame_request();
        let frame = {
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
                })?
        };
        let prepared = crate::facade::entry::WorthUiPreparedMountedContentRebind::authored(
            self,
            frame,
            content.successor_authority,
        );
        Ok(crate::runtime::rebind::UiPreparedRebind::content(
            plan,
            reservation,
            Box::new(prepared),
        ))
    }

    fn prepare_changed_rebind(
        &mut self,
        plan: crate::runtime::rebind::UiRebindPlan,
        reservation: crate::runtime::rebind::UiRebindReservation,
        changed: Box<crate::runtime::rebind::UiChangedRebindSemanticProof>,
    ) -> Result<
        crate::runtime::rebind::UiPreparedRebind<'_>,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        let semantic_content = plan.content().clone();
        let mut prepared = WorthUiPreparedApplicationReplacement::from_changed_rebind_plan(
            self.identity,
            self.application.host_session_plan().clone(),
            std::sync::Arc::clone(self.application.font_collection()),
            *changed,
        )
        .ok_or(crate::runtime::rebind::UiRebindPreparationDenial::CandidateBindingMismatch)?;
        let catalog = self
            .admit_native_replacement_allocation_catalog(&mut prepared)
            .map_err(|_| crate::runtime::rebind::UiRebindPreparationDenial::CandidateAllocation)?;
        let lowered = self
            .lower_prepared_replacement(prepared)
            .map_err(|_| crate::runtime::rebind::UiRebindPreparationDenial::CandidateLowering)?;
        let pending = self
            .stage_prepared_replacement(lowered)
            .map_err(|_| crate::runtime::rebind::UiRebindPreparationDenial::CandidateStaging)?;
        let boundary = self
            .execute_framework_turn(|_| {})
            .map_err(|_| {
                crate::runtime::rebind::UiRebindPreparationDenial::FrameBoundaryUnavailable
            })?
            .into_completion()
            .into_execution()
            .map_err(|_| {
                crate::runtime::rebind::UiRebindPreparationDenial::FrameBoundaryUnavailable
            })?
            .into_activation_boundary();
        let replacement = self
            .prepare_mounted_replacement_with_content(
                pending,
                catalog,
                boundary,
                None,
                semantic_content,
                self.current_portal_rebind_frame_request(),
            )
            .map_err(|denial| match denial {
                crate::facade::WorthUiApplicationCutoverDenial::MountedFrame(denial) => {
                    crate::runtime::rebind::UiRebindPreparationDenial::CandidateMountedPreparation(
                        Box::new(denial),
                    )
                }
                _ => crate::runtime::rebind::UiRebindPreparationDenial::CandidateCutoverPreparation,
            })?;
        let replacement = match replacement {
            WorthUiMountedReplacementPreparationOutcome::Prepared(replacement) => replacement,
            WorthUiMountedReplacementPreparationOutcome::SemanticNoOp(_) => return Err(
                crate::runtime::rebind::UiRebindPreparationDenial::PlannedChangeBecameSemanticNoOp,
            ),
        };
        Ok(crate::runtime::rebind::UiPreparedRebind::changed(
            plan,
            reservation,
            replacement,
        ))
    }

    fn current_portal_rebind_frame_request(&self) -> crate::mounting::UiMountedFrameRequest {
        self.mounted_frame_request()
    }
}
