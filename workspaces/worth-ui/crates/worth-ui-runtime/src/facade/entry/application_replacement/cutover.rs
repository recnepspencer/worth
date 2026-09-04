use super::publication_observation::WorthUiApplicationPublicationPreparation;
use super::*;
use crate::facade::WorthUiActiveApplicationSession;

mod appearance;
mod application_commit;
mod outcome;
use outcome::seal_prepared_activation;

pub(super) struct WorthUiCutoverGenerationBasis {
    pub(super) prior: WorthUiPreparedApplicationGenerationIdentity,
    pub(super) active: WorthUiPreparedApplicationGenerationIdentity,
}

struct WorthUiPreparedCutoverEvidence {
    generations: WorthUiCutoverGenerationBasis,
    visual_trace_source:
        crate::facade::prepared_application_authority::WorthUiPreparedVisualTraceSource,
    reload_cost_seed: crate::runtime::WorthUiReloadCostSeed,
    runtime_basis: crate::runtime::session::WorthUiRuntimePublicationBasis,
    host_session: crate::facade::WorthUiHostSessionIdentity,
    font_collection: std::sync::Arc<worth_ui_text::UiGlobalFontCollection>,
    candidate_graph: crate::graph::UiGraphSnapshot,
    candidate_application_authority:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationLoweringAuthority,
    candidate_service_policy_plan: crate::declaration::UiNormalizedServicePolicyPlan,
    appearance_succession: super::super::UiPreparedAppearanceGenerationSuccession,
}

struct WorthUiCutoverPreparationInput {
    pending: WorthUiPendingApplicationCutover,
    admitted_delta: crate::graph::UiAdmittedAllocationCatalogDelta,
    boundary: crate::runtime::WorthUiFrameBoundary,
    lane_parity_report: Option<crate::runtime::WorthUiLaneParityReport>,
}

struct WorthUiPreparedCatalogActivation {
    prepared: crate::runtime::WorthUiPreparedQueryAwarePlanOutcome,
    reload_cost_seed: crate::runtime::WorthUiReloadCostSeed,
    visual_trace_source:
        crate::facade::prepared_application_authority::WorthUiPreparedVisualTraceSource,
    font_collection: std::sync::Arc<worth_ui_text::UiGlobalFontCollection>,
    appearance_succession: super::super::UiPreparedAppearanceGenerationSuccession,
}

impl WorthUiActiveApplicationSession {
    pub fn activate_prepared_replacement(
        &mut self,
        pending: WorthUiPendingApplicationCutover,
        admitted_delta: crate::graph::UiAdmittedAllocationCatalogDelta,
        boundary: crate::runtime::WorthUiFrameBoundary,
        lane_parity_report: Option<crate::runtime::WorthUiLaneParityReport>,
    ) -> Result<WorthUiApplicationReplacementOutcome, WorthUiApplicationCutoverDenial> {
        if !self.mounted.view().surface_bindings().is_empty() {
            return Err(
                WorthUiApplicationCutoverDenial::MountedPresentationRequired {
                    retry: Box::new(WorthUiApplicationCutoverRetry {
                        pending,
                        admitted_delta,
                        lane_parity_report,
                    }),
                },
            );
        }
        let candidate_graph = pending.next_app.graph_snapshot().clone();
        let prepared = self.prepare_application_cutover(
            pending,
            admitted_delta,
            boundary,
            lane_parity_report,
        )?;
        match prepared {
            WorthUiPreparedApplicationCutoverOutcome::SemanticNoOp(receipt) => {
                Ok(WorthUiApplicationReplacementOutcome::SemanticNoOp(receipt))
            }
            WorthUiPreparedApplicationCutoverOutcome::Activation(activation) => {
                let next_mounted = self
                    .mounted
                    .prepare_graph_replacement_successor(crate::graph::UiGraphAuthority::new(
                        &candidate_graph,
                    ))
                    .map_err(WorthUiApplicationCutoverDenial::MountedIdentity)?;
                let lifecycle = self.prepare_application_lifecycle(
                    &next_mounted,
                    activation.candidate_service_policy_plan().portal(),
                );
                let scroll = self.prepare_scroll_replacement(&activation, &next_mounted, None);
                let selection =
                    self.prepare_selection_replacement(&activation, &next_mounted, false);
                let receipt = self.commit_application_activation(
                    activation,
                    next_mounted,
                    lifecycle,
                    scroll,
                    selection,
                );
                Ok(WorthUiApplicationReplacementOutcome::Activated(Box::new(
                    receipt,
                )))
            }
        }
    }

    pub(super) fn prepare_application_cutover(
        &mut self,
        pending: WorthUiPendingApplicationCutover,
        admitted_delta: crate::graph::UiAdmittedAllocationCatalogDelta,
        boundary: crate::runtime::WorthUiFrameBoundary,
        lane_parity_report: Option<crate::runtime::WorthUiLaneParityReport>,
    ) -> Result<WorthUiPreparedApplicationCutoverOutcome, WorthUiApplicationCutoverDenial> {
        let candidate_graph = pending.next_app.graph_snapshot().clone();
        let candidate_application_authority =
            pending.next_app.prepared_authority().lowering_authority();
        let candidate_service_policy_plan =
            pending.next_app.prepared_authority().service_policy_plan();
        appearance::validate_candidate_owner_installation(
            &pending,
            &candidate_service_policy_plan,
        )?;
        if let Some(kind) = self.portal_exit_retention.pending_replacement_kind() {
            return Err(
                WorthUiApplicationCutoverDenial::PortalExitRetentionPending {
                    kind: replacement_pending_kind(kind),
                    retry: Box::new(WorthUiApplicationCutoverRetry {
                        pending,
                        admitted_delta,
                        lane_parity_report,
                    }),
                },
            );
        }
        if self.mounted.has_active_presentation_attempt() {
            return Err(WorthUiApplicationCutoverDenial::MountedPresentationInFlight);
        }
        if !pending.basis.admits_session(self.session_identity()) {
            return Err(WorthUiApplicationCutoverDenial::ForeignActiveApplicationSession);
        }
        if let Some(reason) = retryable_boundary_denial(self, &pending, boundary) {
            return Err(WorthUiApplicationCutoverDenial::FrameBoundaryUnavailable {
                reason,
                retry: Box::new(WorthUiApplicationCutoverRetry {
                    pending,
                    admitted_delta,
                    lane_parity_report,
                }),
            });
        }
        let generations = self.validate_cutover_generation_basis(&pending, &admitted_delta)?;
        let prepared_catalog = self.prepare_catalog_activation(WorthUiCutoverPreparationInput {
            pending,
            admitted_delta,
            boundary,
            lane_parity_report,
        })?;
        let evidence = WorthUiPreparedCutoverEvidence {
            generations,
            visual_trace_source: prepared_catalog.visual_trace_source,
            reload_cost_seed: prepared_catalog.reload_cost_seed,
            runtime_basis: self.application.runtime_publication_basis(),
            host_session: self.host_session.identity(),
            font_collection: prepared_catalog.font_collection,
            candidate_graph,
            candidate_application_authority,
            candidate_service_policy_plan,
            appearance_succession: prepared_catalog.appearance_succession,
        };
        match prepared_catalog.prepared.into_activation() {
            Err(receipt) => Ok(seal_semantic_no_op(evidence, receipt)),
            Ok(activation) => {
                let activation = activation.into_application_activation().map_err(|_| {
                    WorthUiApplicationCutoverDenial::MissingAllocationCatalogSuccessorReceipt
                })?;
                Ok(seal_prepared_activation(evidence, activation))
            }
        }
    }

    fn prepare_catalog_activation(
        &mut self,
        input: WorthUiCutoverPreparationInput,
    ) -> Result<WorthUiPreparedCatalogActivation, WorthUiApplicationCutoverDenial> {
        let pending = input.pending;
        let successor_generation =
            crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
                self.identity,
                pending.next_app.generation_identity(),
            );
        let appearance_succession =
            appearance::prepare_successor_theme(self, &pending, successor_generation)?;
        let reload_cost_seed = pending.reload_cost_seed;
        let visual_trace_source = pending.next_app.visual_trace_source();
        let font_collection = std::sync::Arc::clone(pending.next_app.font_collection());
        let successor_planning_authority =
            std::rc::Rc::clone(pending.next_app.retained_planning_authority());
        let application_publication =
            crate::runtime::WorthUiPreparedApplicationPublication::replacement(
                self.application.prepared_authority(),
                pending.next_app,
            );
        let prepared = self
            .application
            .prepare_admitted_allocation_catalog_delta(
                pending.pending_activation,
                crate::runtime::UiAllocationCatalogDeltaActivationInput {
                    admitted_delta: input.admitted_delta,
                    active_graph: self.application.graph_snapshot().clone(),
                    graph_changed_nodes: pending.candidate_graph_changed_nodes,
                    boundary: input.boundary,
                    lane_parity_report: input.lane_parity_report,
                    candidate_query_binding: pending.candidate_query_binding,
                    successor_planning_authority,
                    application_publication,
                },
            )
            .map_err(WorthUiApplicationCutoverDenial::Activation)?;
        Ok(WorthUiPreparedCatalogActivation {
            prepared,
            reload_cost_seed,
            visual_trace_source,
            font_collection,
            appearance_succession,
        })
    }
}

fn replacement_pending_kind(
    kind: crate::facade::entry::active_application_session::UiPortalExitTerminalPendingKind,
) -> WorthUiPortalExitRetentionPendingKind {
    match kind {
        crate::facade::entry::active_application_session::UiPortalExitTerminalPendingKind::InFlight => {
            WorthUiPortalExitRetentionPendingKind::InFlight
        }
        crate::facade::entry::active_application_session::UiPortalExitTerminalPendingKind::Indeterminate => {
            WorthUiPortalExitRetentionPendingKind::Indeterminate
        }
        crate::facade::entry::active_application_session::UiPortalExitTerminalPendingKind::Reconstruction => {
            WorthUiPortalExitRetentionPendingKind::Reconstruction
        }
    }
}

fn seal_semantic_no_op(
    evidence: WorthUiPreparedCutoverEvidence,
    receipt: Box<crate::runtime::WorthUiSemanticNoOpReceipt>,
) -> WorthUiPreparedApplicationCutoverOutcome {
    let WorthUiPreparedCutoverEvidence {
        generations,
        reload_cost_seed,
        ..
    } = evidence;
    debug_assert_eq!(receipt.active_generation(), &generations.prior);
    debug_assert_eq!(receipt.candidate_generation(), &generations.active);
    let reload_cost = reload_cost_seed.finish(
        generations.prior,
        generations.active,
        receipt.equivalence().previous_fingerprint(),
        receipt.candidate_construction(),
        receipt.equivalence(),
    );
    WorthUiPreparedApplicationCutoverOutcome::SemanticNoOp(Box::new(
        WorthUiApplicationSemanticNoOpReceipt {
            receipt: *receipt,
            reload_cost,
        },
    ))
}

fn retryable_boundary_denial(
    session: &WorthUiActiveApplicationSession,
    pending: &WorthUiPendingApplicationCutover,
    boundary: crate::runtime::WorthUiFrameBoundary,
) -> Option<crate::runtime::WorthUiActivationGateDenialReason> {
    if !boundary.is_safe_to_activate() {
        return Some(crate::runtime::WorthUiActivationGateDenialReason::UnsafeFrameBoundary);
    }
    if boundary.host_session() != session.host_session_identity() {
        return Some(
            crate::runtime::WorthUiActivationGateDenialReason::ForeignFrameBoundarySession,
        );
    }
    let readiness_epoch = pending.pending_activation.frame_epoch();
    if boundary.frame_epoch() < readiness_epoch {
        return Some(crate::runtime::WorthUiActivationGateDenialReason::StaleFrameEpoch);
    }
    if boundary.frame_epoch() > readiness_epoch {
        return Some(crate::runtime::WorthUiActivationGateDenialReason::FutureFrameEpochMismatch);
    }
    (boundary.frame_epoch() != session.application.frame_epoch())
        .then_some(crate::runtime::WorthUiActivationGateDenialReason::BoundaryFrameEpochMismatch)
}
