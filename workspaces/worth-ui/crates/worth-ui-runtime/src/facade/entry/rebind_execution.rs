use super::{
    WorthUiActiveApplicationSession, WorthUiMountedReplacementPreparationOutcome,
    WorthUiPreparedApplicationReplacement,
};

mod evidence_only;
pub(crate) use evidence_only::WorthUiPreparedEvidenceOnlyApplicationRebind;

impl WorthUiActiveApplicationSession {
    pub fn prepare_rebind(
        &mut self,
        plan: crate::runtime::rebind::UiRebindPlan,
        request: crate::runtime::rebind::UiRebindExecutionRequest,
    ) -> Result<
        crate::runtime::rebind::UiPreparedRebind<'_>,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        self.prepare_rebind_with_inputs(plan, request, &[], None, None, None)
    }

    pub(in crate::facade::entry) fn prepare_native_rebind(
        &mut self,
        plan: crate::runtime::rebind::UiRebindPlan,
        request: crate::runtime::rebind::UiRebindExecutionRequest,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        viewport: Option<worth_ui_host_contract::UiMountedCanonicalBox>,
        layout: Option<&mut super::application_replacement::UiNativeReplacementLayoutSupplier<'_>>,
    ) -> Result<
        crate::runtime::rebind::UiPreparedRebind<'_>,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        self.prepare_rebind_with_inputs(plan, request, &[], Some(surface), viewport, layout)
    }

    pub(in crate::facade::entry) fn prepare_rebind_with_reconciliation(
        &mut self,
        plan: crate::runtime::rebind::UiRebindPlan,
        request: crate::runtime::rebind::UiRebindExecutionRequest,
        reconciliation: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
    ) -> Result<
        crate::runtime::rebind::UiPreparedRebind<'_>,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        self.prepare_rebind_with_inputs(plan, request, reconciliation, None, None, None)
    }

    fn prepare_rebind_with_inputs(
        &mut self,
        mut plan: crate::runtime::rebind::UiRebindPlan,
        request: crate::runtime::rebind::UiRebindExecutionRequest,
        reconciliation: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
        native_surface: Option<worth_ui_host_contract::UiSemanticSurfaceIdentity>,
        native_viewport: Option<worth_ui_host_contract::UiMountedCanonicalBox>,
        native_layout: Option<
            &mut super::application_replacement::UiNativeReplacementLayoutSupplier<'_>,
        >,
    ) -> Result<
        crate::runtime::rebind::UiPreparedRebind<'_>,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        let pointer_succession = plan
            .retained_successor_authority()
            .map(|authority| self.prepare_pointer_generation_succession(authority))
            .transpose()?;
        let pointer_publication = pointer_succession.as_ref().is_some_and(|pointer| {
            !pointer.changed_surfaces().is_empty() || pointer.requires_timed_publication()
        });
        if pointer_publication {
            let pointer = pointer_succession
                .as_ref()
                .expect("publication has prepared pointer owner");
            self.include_pointer_publication(&mut plan, pointer)?;
        }
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
        let semantic_proof = plan.take_semantic_proof();
        if !reconciliation.is_empty()
            && !matches!(
                &semantic_proof,
                crate::runtime::rebind::UiRebindSemanticProof::ThemeSwitch(_)
            )
        {
            return Err(crate::runtime::rebind::UiRebindPreparationDenial::InvalidSemanticProof);
        }
        match semantic_proof {
            crate::runtime::rebind::UiRebindSemanticProof::Changed(changed) => self
                .prepare_changed_rebind(
                    plan,
                    reservation,
                    changed,
                    native_surface,
                    native_viewport,
                    native_layout,
                ),
            crate::runtime::rebind::UiRebindSemanticProof::AuthoredContent(content) => self
                .prepare_authored_content_rebind(
                    plan,
                    reservation,
                    *content,
                    pointer_succession
                        .expect("authored content requires prepared pointer succession"),
                ),
            crate::runtime::rebind::UiRebindSemanticProof::EvidenceOnly(succession) => {
                let pointer = pointer_succession
                    .expect("evidence publication requires prepared pointer succession");
                if pointer_publication {
                    let crate::runtime::observation::UiAuthoredSourceSuccession::EvidenceOnly {
                        successor_authority,
                        admitted_candidate,
                        comparison: _,
                    } = *succession
                    else {
                        unreachable!("evidence plan retains evidence-only succession")
                    };
                    let content = crate::runtime::rebind::UiAuthoredContentRebindSemanticProof {
                        successor_authority,
                        source_candidate_artifact_digest: admitted_candidate
                            .candidate()
                            .basis()
                            .artifact_digest()
                            .raw(),
                    };
                    return self.prepare_authored_content_rebind(
                        plan,
                        reservation,
                        content,
                        pointer,
                    );
                }
                let prepared =
                    WorthUiPreparedEvidenceOnlyApplicationRebind::new(self, *succession, pointer)?;
                crate::runtime::rebind::UiPreparedRebind::evidence_only(plan, reservation, prepared)
            }
            crate::runtime::rebind::UiRebindSemanticProof::ThemeSwitch(theme) => {
                let content = Box::new(super::WorthUiPreparedMountedContentRebind::prepare_theme(
                    self,
                    plan.content().clone(),
                    *theme,
                    reconciliation,
                )?);
                Ok(crate::runtime::rebind::UiPreparedRebind::content(
                    plan,
                    reservation,
                    content,
                ))
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
        let content = Box::new(
            crate::facade::entry::WorthUiPreparedMountedContentRebind::prepare(
                self,
                plan.content().clone(),
            )?,
        );
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
        pointer_succession: crate::runtime::pointer_affordance::UiPreparedPointerAffordanceGenerationSuccession,
    ) -> Result<
        crate::runtime::rebind::UiPreparedRebind<'_>,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        let semantic_content = plan.content().clone();
        let overlay_bindings = self
            .authored_overlay_bindings
            .prepare_application_replacement(
                self.application.prepared_authority(),
                &content.successor_authority,
                &[],
            )
            .map_err(|_| {
                crate::runtime::rebind::UiRebindPreparationDenial::CandidateCutoverPreparation
            })?;
        let occurrence_geometry = self
            .application
            .prepare_retained_layout_succession(
                &self.mounted,
                &content.successor_authority,
                &overlay_bindings,
            )
            .map_err(
                crate::runtime::rebind::UiRebindPreparationDenial::CandidateOccurrenceGeometry,
            )?;
        let predecessor = self.active_generation_identity();
        let successor = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
            self.session_identity(),
            content.successor_authority.generation_identity(),
        );
        let generation_succession = crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationSuccession::new(
                predecessor.prepared_generation().clone(),
                successor.prepared_generation().clone(),
            );
        let appearance_succession = self
            .prepare_appearance_generation_succession(&generation_succession)
            .map_err(|denial| {
                match denial {
                super::UiAppearanceGenerationSuccessionDenial::Theme(denial) => {
                    crate::runtime::rebind::UiRebindPreparationDenial::AppearanceThemeSuccession(
                        denial,
                    )
                }
                super::UiAppearanceGenerationSuccessionDenial::Inspection(denial) => {
                    crate::runtime::rebind::UiRebindPreparationDenial::
                        AppearanceInspectionSuccession(denial)
                }
            }
            })?;
        let owners = self.prepare_retained_appearance_owners(
            &content.successor_authority,
            &appearance_succession,
        )?;
        let prepared = crate::facade::entry::WorthUiPreparedMountedContentRebind::prepare_authored(
            self,
            semantic_content,
            content.successor_authority,
            appearance_succession,
            overlay_bindings,
            occurrence_geometry,
            pointer_succession,
            owners,
        )?;
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
        native_surface: Option<worth_ui_host_contract::UiSemanticSurfaceIdentity>,
        native_viewport: Option<worth_ui_host_contract::UiMountedCanonicalBox>,
        native_layout: Option<
            &mut super::application_replacement::UiNativeReplacementLayoutSupplier<'_>,
        >,
    ) -> Result<
        crate::runtime::rebind::UiPreparedRebind<'_>,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        let native_component_candidates = native_surface
            .map(|_| {
                plan.identity_decisions()
                    .iter()
                    .filter(|entry| {
                        entry.key().kind() == crate::graph::UiGraphFactConsumerKind::GraphNode
                            && entry.key().authored_identity().starts_with("component:")
                    })
                    .filter_map(|entry| match entry.candidate() {
                        Some(crate::graph::UiGraphFactConsumerIdentity::GraphNode(node)) => {
                            Some(node)
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
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
                native_surface,
                &native_component_candidates,
                native_viewport,
                native_layout,
            )
            .map_err(|denial| match denial {
                crate::facade::WorthUiApplicationCutoverDenial::MountedFrame(denial) => {
                    crate::runtime::rebind::UiRebindPreparationDenial::CandidateMountedPreparation(
                        Box::new(denial),
                    )
                }
                crate::facade::WorthUiApplicationCutoverDenial::OccurrenceGeometry(denial) => {
                    crate::runtime::rebind::UiRebindPreparationDenial::CandidateOccurrenceGeometry(
                        denial,
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
