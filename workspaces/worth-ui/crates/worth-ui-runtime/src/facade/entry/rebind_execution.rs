use super::{
    WorthUiActiveApplicationSession, WorthUiMountedReplacementPreparationOutcome,
    WorthUiPreparedApplicationReplacement,
};
use crate::facade::WorthUiApplicationCutoverDenial;
use crate::runtime::rebind::UiRebindPreparationDenial;

mod evidence_only;
pub(crate) use evidence_only::WorthUiPreparedEvidenceOnlyApplicationRebind;

impl WorthUiActiveApplicationSession {
    pub fn prepare_rebind(
        &mut self,
        plan: crate::runtime::rebind::UiRebindPlan,
        request: crate::runtime::rebind::UiRebindExecutionRequest,
    ) -> Result<crate::runtime::rebind::UiPreparedRebind<'_>, UiRebindPreparationDenial> {
        self.prepare_rebind_with_inputs(plan, request, &[], None, None, None)
    }

    pub(in crate::facade::entry) fn prepare_native_rebind(
        &mut self,
        plan: crate::runtime::rebind::UiRebindPlan,
        request: crate::runtime::rebind::UiRebindExecutionRequest,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        viewport: Option<worth_ui_host_contract::UiMountedCanonicalBox>,
        layout: Option<&mut super::application_replacement::UiNativeReplacementLayoutSupplier<'_>>,
    ) -> Result<crate::runtime::rebind::UiPreparedRebind<'_>, UiRebindPreparationDenial> {
        self.prepare_rebind_with_inputs(plan, request, &[], Some(surface), viewport, layout)
    }

    pub(in crate::facade::entry) fn prepare_rebind_with_reconciliation(
        &mut self,
        plan: crate::runtime::rebind::UiRebindPlan,
        request: crate::runtime::rebind::UiRebindExecutionRequest,
        reconciliation: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
    ) -> Result<crate::runtime::rebind::UiPreparedRebind<'_>, UiRebindPreparationDenial> {
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
    ) -> Result<crate::runtime::rebind::UiPreparedRebind<'_>, UiRebindPreparationDenial> {
        let successions = plan
            .retained_successor_authority()
            .map(|authority| self.prepare_retained_successions(authority))
            .transpose()?;
        let pointer_publication = successions.as_ref().is_some_and(|(_, pointer)| {
            !pointer.changed_surfaces().is_empty() || pointer.requires_timed_publication()
        });
        if let Some((_, pointer)) = successions.as_ref().filter(|_| pointer_publication) {
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
            return Err(UiRebindPreparationDenial::InvalidSemanticProof);
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
            crate::runtime::rebind::UiRebindSemanticProof::AuthoredContent(content) => {
                let (expressions, pointer) =
                    successions.expect("authored content requires prepared successions");
                self.prepare_authored_content_rebind(
                    plan,
                    reservation,
                    *content,
                    expressions,
                    pointer,
                )
            }
            crate::runtime::rebind::UiRebindSemanticProof::EvidenceOnly(succession) => {
                let (expressions, pointer) =
                    successions.expect("evidence publication requires prepared successions");
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
                        expressions,
                        pointer,
                    );
                }
                let prepared = WorthUiPreparedEvidenceOnlyApplicationRebind::new(
                    self,
                    *succession,
                    expressions,
                    pointer,
                )?;
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
                Err(UiRebindPreparationDenial::InvalidSemanticProof)
            }
        }
    }

    fn prepare_content_rebind(
        &mut self,
        plan: crate::runtime::rebind::UiRebindPlan,
        reservation: crate::runtime::rebind::UiRebindReservation,
    ) -> Result<crate::runtime::rebind::UiPreparedRebind<'_>, UiRebindPreparationDenial> {
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

    /// The expression succession to `authority`'s generation, and the pointer
    /// succession observed through it, so a condition reads at the successor
    /// generation the outcome that generation will hold.
    pub(in crate::facade::entry) fn prepare_retained_successions(
        &mut self,
        authority: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    ) -> Result<
        (
            crate::runtime::expression::UiPreparedExpressionSuccession,
            crate::runtime::pointer_affordance::UiPreparedPointerAffordanceGenerationSuccession,
        ),
        UiRebindPreparationDenial,
    > {
        let successor = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
            self.session_identity(),
            authority.generation_identity(),
        );
        let expressions =
            self.prepare_expression_succession(&successor, authority.expression_catalog());
        let pointer = self.prepare_pointer_generation_succession(authority, &expressions)?;
        Ok((expressions, pointer))
    }

    fn prepare_authored_content_rebind(
        &mut self,
        plan: crate::runtime::rebind::UiRebindPlan,
        reservation: crate::runtime::rebind::UiRebindReservation,
        content: crate::runtime::rebind::UiAuthoredContentRebindSemanticProof,
        expressions: crate::runtime::expression::UiPreparedExpressionSuccession,
        pointer_succession: crate::runtime::pointer_affordance::UiPreparedPointerAffordanceGenerationSuccession,
    ) -> Result<crate::runtime::rebind::UiPreparedRebind<'_>, UiRebindPreparationDenial> {
        let semantic_content = plan.content().clone();
        let overlay_bindings = self
            .authored_overlay_bindings
            .prepare_application_replacement(
                self.application.prepared_authority(),
                &content.successor_authority,
                &[],
            )
            .map_err(|_| UiRebindPreparationDenial::CandidateCutoverPreparation)?;
        let occurrence_geometry = self
            .application
            .prepare_retained_layout_succession(
                &self.mounted,
                &content.successor_authority,
                &overlay_bindings,
            )
            .map_err(UiRebindPreparationDenial::CandidateOccurrenceGeometry)?;
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
            .map_err(|denial| match denial {
                super::UiAppearanceGenerationSuccessionDenial::Theme(denial) => {
                    UiRebindPreparationDenial::AppearanceThemeSuccession(denial)
                }
                super::UiAppearanceGenerationSuccessionDenial::Inspection(denial) => {
                    UiRebindPreparationDenial::AppearanceInspectionSuccession(denial)
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
            expressions,
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
    ) -> Result<crate::runtime::rebind::UiPreparedRebind<'_>, UiRebindPreparationDenial> {
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
        .ok_or(UiRebindPreparationDenial::CandidateBindingMismatch)?;
        let catalog = self
            .admit_native_replacement_allocation_catalog(&mut prepared)
            .map_err(|_| UiRebindPreparationDenial::CandidateAllocation)?;
        let lowered = self
            .lower_prepared_replacement(prepared)
            .map_err(|_| UiRebindPreparationDenial::CandidateLowering)?;
        let pending = self
            .stage_prepared_replacement(lowered)
            .map_err(|_| UiRebindPreparationDenial::CandidateStaging)?;
        let boundary = self
            .execute_framework_turn(|_| {})
            .map_err(|_| UiRebindPreparationDenial::FrameBoundaryUnavailable)?
            .into_completion()
            .into_execution()
            .map_err(|_| UiRebindPreparationDenial::FrameBoundaryUnavailable)?
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
                WorthUiApplicationCutoverDenial::MountedFrame(denial) => {
                    UiRebindPreparationDenial::CandidateMountedPreparation(Box::new(denial))
                }
                WorthUiApplicationCutoverDenial::OccurrenceGeometry(denial) => {
                    UiRebindPreparationDenial::CandidateOccurrenceGeometry(denial)
                }
                WorthUiApplicationCutoverDenial::MountedPresentationInFlight
                | WorthUiApplicationCutoverDenial::IncompleteMountedSurfaceScope
                | WorthUiApplicationCutoverDenial::ForeignActiveApplicationSession
                | WorthUiApplicationCutoverDenial::AppearanceOwnerUnavailable(_)
                | WorthUiApplicationCutoverDenial::AppearanceOwnerSuccessionUnavailable
                | WorthUiApplicationCutoverDenial::OverlayBindingSuccessionUnavailable
                | WorthUiApplicationCutoverDenial::AppearanceThemeAdmission(_)
                | WorthUiApplicationCutoverDenial::AppearanceThemeSuccession(_)
                | WorthUiApplicationCutoverDenial::AppearanceInspectionSuccession(_)
                | WorthUiApplicationCutoverDenial::PreparedApplicationGraphMismatch
                | WorthUiApplicationCutoverDenial::PreparedApplicationAuthorityMismatch
                | WorthUiApplicationCutoverDenial::FrameBoundaryUnavailable { .. }
                | WorthUiApplicationCutoverDenial::MountedIdentity(_)
                | WorthUiApplicationCutoverDenial::MountedPresentationRequired { .. }
                | WorthUiApplicationCutoverDenial::PortalExitRetentionPending { .. }
                | WorthUiApplicationCutoverDenial::MissingAllocationCatalogSuccessorReceipt
                | WorthUiApplicationCutoverDenial::Activation(_) => {
                    UiRebindPreparationDenial::CandidateCutoverPreparation
                }
            })?;
        let replacement = match replacement {
            WorthUiMountedReplacementPreparationOutcome::Prepared(replacement) => replacement,
            WorthUiMountedReplacementPreparationOutcome::SemanticNoOp(_) => {
                return Err(UiRebindPreparationDenial::PlannedChangeBecameSemanticNoOp)
            }
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
