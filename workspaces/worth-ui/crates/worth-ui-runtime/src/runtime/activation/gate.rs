use super::query_aware_plan_outcome::WorthUiPreparedQueryAwarePlanOutcome;
#[cfg(test)]
use super::query_aware_plan_outcome::WorthUiQueryAwarePlanOutcome;
use super::{
    UiAllocationCatalogDeltaActivationInput, WorthUiAllocationCatalogActivationDenial,
    WorthUiAllocationCatalogPreparationStage,
};
#[cfg(any(test, feature = "certification-support"))]
use crate::runtime::WorthUiPlanSwapReceipt;
use crate::runtime::WorthUiRuntime;
use crate::runtime::{
    WorthUiExecutionPlan, WorthUiFrameBoundary, WorthUiLaneParityReport, WorthUiPendingActivation,
};

enum UiAllocationCatalogReplacementInput {
    #[cfg(any(test, feature = "certification-support"))]
    Complete(crate::graph::UiAdmittedAllocationCatalogBasisSet),
    Delta {
        admitted: Box<crate::graph::UiAdmittedAllocationCatalogDelta>,
        active_graph: crate::graph::UiGraphSnapshot,
        graph_changed_nodes: std::collections::BTreeSet<crate::graph::UiGraphNodeIdentity>,
    },
}

impl WorthUiRuntime {
    #[cfg(test)]
    pub(crate) fn activate_admitted_allocation_catalog_at_frame_boundary(
        &mut self,
        pending_activation: WorthUiPendingActivation,
        admitted_catalog: crate::graph::UiAdmittedAllocationCatalogBasisSet,
        boundary: WorthUiFrameBoundary,
        lane_parity_report: Option<WorthUiLaneParityReport>,
    ) -> Result<WorthUiPlanSwapReceipt, WorthUiAllocationCatalogActivationDenial> {
        let candidate_query_binding = pending_activation
            .candidate_application_authority()
            .query_binding_plan()
            .prepare_downstream_state();
        let successor_planning_authority =
            std::rc::Rc::clone(&self.retained_allocation_planning_evidence);
        self.activate_admitted_allocation_catalog_with_query_binding(
            pending_activation,
            admitted_catalog,
            boundary,
            lane_parity_report,
            candidate_query_binding,
            successor_planning_authority,
        )
        .map(|publication| publication.into_plan_swap_after_asserting_no_query_retirement())
    }

    #[cfg(test)]
    pub(crate) fn activate_admitted_allocation_catalog_with_query_binding(
        &mut self,
        pending_activation: WorthUiPendingActivation,
        admitted_catalog: crate::graph::UiAdmittedAllocationCatalogBasisSet,
        boundary: WorthUiFrameBoundary,
        lane_parity_report: Option<WorthUiLaneParityReport>,
        candidate_query_binding: worth_ui_query_binding::WorthUiRuntimeQueryBinding,
        successor_planning_authority: std::rc::Rc<
            crate::runtime::WorthUiRetainedAllocationPlanningEvidenceRegistry,
        >,
    ) -> Result<WorthUiQueryAwarePlanOutcome, WorthUiAllocationCatalogActivationDenial> {
        let mut lane_parity_report = Some(lane_parity_report);
        let prepared = self
            .prepare_admitted_allocation_catalog_with_boundary_source_and_query_binding(
                pending_activation,
                UiAllocationCatalogReplacementInput::Complete(admitted_catalog),
                &mut |_, _, _, _| {
                    Ok((
                        boundary,
                        lane_parity_report
                            .take()
                            .expect("boundary source runs once"),
                    ))
                },
                candidate_query_binding,
                successor_planning_authority,
                None,
            )?;
        Ok(prepared.commit_once(self, None))
    }

    pub(crate) fn prepare_admitted_allocation_catalog_delta_with_query_binding(
        &mut self,
        pending_activation: WorthUiPendingActivation,
        input: UiAllocationCatalogDeltaActivationInput,
    ) -> Result<WorthUiPreparedQueryAwarePlanOutcome, WorthUiAllocationCatalogActivationDenial>
    {
        let UiAllocationCatalogDeltaActivationInput {
            admitted_delta,
            active_graph,
            graph_changed_nodes,
            boundary,
            lane_parity_report,
            candidate_query_binding,
            successor_planning_authority,
            application_publication,
        } = input;
        let mut lane_parity_report = Some(lane_parity_report);
        self.prepare_admitted_allocation_catalog_with_boundary_source_and_query_binding(
            pending_activation,
            UiAllocationCatalogReplacementInput::Delta {
                admitted: Box::new(admitted_delta),
                active_graph,
                graph_changed_nodes,
            },
            &mut |_, _, _, _| {
                Ok((
                    boundary,
                    lane_parity_report
                        .take()
                        .expect("boundary source runs once"),
                ))
            },
            candidate_query_binding,
            successor_planning_authority,
            Some(application_publication),
        )
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn activate_admitted_allocation_catalog_with_boundary_source<F>(
        &mut self,
        pending_activation: WorthUiPendingActivation,
        admitted_catalog: crate::graph::UiAdmittedAllocationCatalogBasisSet,
        boundary_source: F,
    ) -> Result<WorthUiPlanSwapReceipt, WorthUiAllocationCatalogActivationDenial>
    where
        F: FnOnce(
            &mut WorthUiRuntime,
            &crate::runtime::UiAllocationReceipt,
            &WorthUiExecutionPlan,
            &crate::runtime::planning::WorthUiExecutionPlanLoweringFacts,
        ) -> Result<
            (WorthUiFrameBoundary, Option<WorthUiLaneParityReport>),
            WorthUiAllocationCatalogActivationDenial,
        >,
    {
        let candidate_query_binding = pending_activation
            .candidate_application_authority()
            .query_binding_plan()
            .prepare_downstream_state();
        let successor_planning_authority =
            std::rc::Rc::clone(&self.retained_allocation_planning_evidence);
        let mut boundary_source = Some(boundary_source);
        let prepared = self
            .prepare_admitted_allocation_catalog_with_boundary_source_and_query_binding(
                pending_activation,
                UiAllocationCatalogReplacementInput::Complete(admitted_catalog),
                &mut |runtime, receipt, plan, facts| {
                    boundary_source.take().expect("boundary source runs once")(
                        runtime, receipt, plan, facts,
                    )
                },
                candidate_query_binding,
                successor_planning_authority,
                None,
            )?;
        Ok(prepared
            .commit_once(self, None)
            .into_plan_swap_after_asserting_no_query_retirement())
    }

    #[inline(never)]
    fn prepare_admitted_allocation_catalog_with_boundary_source_and_query_binding(
        &mut self,
        pending_activation: WorthUiPendingActivation,
        catalog_input: UiAllocationCatalogReplacementInput,
        boundary_source: &mut dyn FnMut(
            &mut WorthUiRuntime,
            &crate::runtime::UiAllocationReceipt,
            &WorthUiExecutionPlan,
            &crate::runtime::planning::WorthUiExecutionPlanLoweringFacts,
        ) -> Result<
            (WorthUiFrameBoundary, Option<WorthUiLaneParityReport>),
            WorthUiAllocationCatalogActivationDenial,
        >,
        candidate_query_binding: worth_ui_query_binding::WorthUiRuntimeQueryBinding,
        successor_planning_authority: std::rc::Rc<
            crate::runtime::WorthUiRetainedAllocationPlanningEvidenceRegistry,
        >,
        application_publication: Option<super::WorthUiPreparedApplicationPublication>,
    ) -> Result<WorthUiPreparedQueryAwarePlanOutcome, WorthUiAllocationCatalogActivationDenial>
    {
        let (prepared, delta_lowering, catalog_successor_receipt) = match catalog_input {
            #[cfg(any(test, feature = "certification-support"))]
            UiAllocationCatalogReplacementInput::Complete(admitted_catalog) => {
                deny_if_certification_interrupted("catalog activation preparation")?;
                self.prepare_allocation_catalog_activation(&pending_activation, admitted_catalog)
                    .map(|prepared| (prepared, None, None))
            }
            UiAllocationCatalogReplacementInput::Delta {
                admitted,
                active_graph,
                graph_changed_nodes,
            } => {
                deny_if_certification_interrupted("allocation delta closure")?;
                let closure = self
                .admit_allocation_catalog_delta_closure(
                    &pending_activation,
                    &active_graph,
                    *admitted,
                    &graph_changed_nodes,
                )
                .map_err(WorthUiAllocationCatalogActivationDenial::AllocationDelta)?;
                deny_if_certification_interrupted("catalog activation preparation")?;
                self.prepare_allocation_catalog_delta_activation(&pending_activation, closure)
                    .map(|(mut prepared, receipt)| {
                        let lowering = prepared.successor_lowering_input(&pending_activation);
                        prepared.bind_catalog_successor_lowering(&lowering);
                        (prepared, Some(lowering), Some(receipt))
                    })
            }
        }
            .map_err(|denial| {
                let stage = match denial {
                    crate::runtime::launch::UiAllocationCatalogPreparationDenial::PlanningAdmission(_) => WorthUiAllocationCatalogPreparationStage::PlanningAdmission,
                    crate::runtime::launch::UiAllocationCatalogPreparationDenial::CatalogPlanning(_) => WorthUiAllocationCatalogPreparationStage::CatalogPlanning,
                    crate::runtime::launch::UiAllocationCatalogPreparationDenial::ReceiptCommit(outcome) => match outcome.as_ref() {
                        #[cfg(test)]
                        crate::runtime::UiAllocationReceiptCommitOutcome::RecomputePending(_) => WorthUiAllocationCatalogPreparationStage::ReceiptRecomputePending,
                        crate::runtime::UiAllocationReceiptCommitOutcome::Denied(denial) => match denial.as_ref() {
                            crate::runtime::UiAllocationReceiptCommitDenial::CatalogBindingCardinalityMismatch => WorthUiAllocationCatalogPreparationStage::ReceiptCatalogBindingCardinalityMismatch,
                            crate::runtime::UiAllocationReceiptCommitDenial::CatalogBindingIdentityMismatch { .. } => WorthUiAllocationCatalogPreparationStage::ReceiptCatalogBindingIdentityMismatch,
                            crate::runtime::UiAllocationReceiptCommitDenial::CatalogActivationAuthority(_) => WorthUiAllocationCatalogPreparationStage::ReceiptCatalogActivationAuthority,
                            crate::runtime::UiAllocationReceiptCommitDenial::CandidatePlanningDenied(_) => WorthUiAllocationCatalogPreparationStage::ReceiptCandidatePlanningDenied,
                            #[cfg(test)]
                            crate::runtime::UiAllocationReceiptCommitDenial::ReuseDenied(_) => WorthUiAllocationCatalogPreparationStage::ReceiptReuseDenied,
                            crate::runtime::UiAllocationReceiptCommitDenial::AuthorityCounterExhausted(_) => WorthUiAllocationCatalogPreparationStage::ReceiptAuthorityCounterExhausted,
                            crate::runtime::UiAllocationReceiptCommitDenial::EvidenceCounterExhausted => WorthUiAllocationCatalogPreparationStage::ReceiptEvidenceCounterExhausted,
                        },
                        #[cfg(test)]
                        crate::runtime::UiAllocationReceiptCommitOutcome::Committed(_) => WorthUiAllocationCatalogPreparationStage::UnexpectedCommittedReceipt,
                    },
                };
                WorthUiAllocationCatalogActivationDenial::Preparation(stage)
            })?;
        let receipt = prepared.primary_receipt().clone();
        deny_if_certification_interrupted("lowering authority")?;
        let lowering_authority = if let Some(lowering_input) = delta_lowering {
            crate::runtime::planning::WorthUiExecutionPlanLoweringAuthority::seal_catalog_successor(
                pending_activation,
                lowering_input,
                self.active.frame_epoch(),
            )
        } else {
            let lowering_input = receipt
                .lowering_input()
                .map_err(WorthUiAllocationCatalogActivationDenial::Freshness)?;
            crate::runtime::planning::WorthUiExecutionPlanLoweringAuthority::seal(
                pending_activation,
                lowering_input,
                self.active.frame_epoch(),
            )
        }
            .map_err(|denial| match denial {
                crate::runtime::planning::WorthUiExecutionPlanLoweringAuthorityDenial::CandidateGraphAuthorityMismatch => {
                    WorthUiAllocationCatalogActivationDenial::CandidateGraphAuthority
                }
                crate::runtime::planning::WorthUiExecutionPlanLoweringAuthorityDenial::CandidateArtifactAuthorityMismatch => {
                    WorthUiAllocationCatalogActivationDenial::CandidateArtifactAuthority
                }
                crate::runtime::planning::WorthUiExecutionPlanLoweringAuthorityDenial::ForeignAllocationProjection => {
                    WorthUiAllocationCatalogActivationDenial::AllocationProjection
                }
                crate::runtime::planning::WorthUiExecutionPlanLoweringAuthorityDenial::MissingQueryPosture => {
                    WorthUiAllocationCatalogActivationDenial::MissingQueryPosture
                }
                crate::runtime::planning::WorthUiExecutionPlanLoweringAuthorityDenial::UnexpectedQueryPosture => {
                    WorthUiAllocationCatalogActivationDenial::UnexpectedQueryPosture
                }
                crate::runtime::planning::WorthUiExecutionPlanLoweringAuthorityDenial::QueryDefinitionNotInstalled => {
                    WorthUiAllocationCatalogActivationDenial::QueryDefinitionNotInstalled
                }
                crate::runtime::planning::WorthUiExecutionPlanLoweringAuthorityDenial::ForeignQueryInstalledAuthority => {
                    WorthUiAllocationCatalogActivationDenial::ForeignQueryInstalledAuthority
                }
                crate::runtime::planning::WorthUiExecutionPlanLoweringAuthorityDenial::RegionalDelta(denial) => {
                    match denial {
                        crate::runtime::planning::plan_topology::WorthUiPlanRegionDeltaDenial::DuplicateCandidateRegion => WorthUiAllocationCatalogActivationDenial::RegionalDeltaDuplicateCandidateRegion,
                    }
                }
                crate::runtime::planning::WorthUiExecutionPlanLoweringAuthorityDenial::PlanInput(denial) => {
                    WorthUiAllocationCatalogActivationDenial::PlanInput(denial)
                }
            })?;
        let handles = self.authorize_regional_successor_handles(lowering_authority.facts());
        deny_if_certification_interrupted("topology assembly")?;
        let (candidate_plan, lane_admission) = self
            .assemble_execution_plan_topology_with_admission(lowering_authority.facts(), &handles)
            .map_err(WorthUiAllocationCatalogActivationDenial::TopologyAssembly)?;
        deny_if_certification_interrupted("frame boundary source")?;
        let (boundary, lane_parity_report) =
            boundary_source(self, &receipt, &candidate_plan, lowering_authority.facts())?;
        deny_if_certification_interrupted("execution bundle seal")?;
        let candidate_bundle = crate::runtime::active::WorthUiSealedExecutionPlanBundle::seal(
            lowering_authority.facts(),
            candidate_plan,
            &lane_admission,
            self.host_plan_binding,
        )
        .map_err(|denial| match denial {
            crate::runtime::active::WorthUiExecutionPlanBundleDenial::ForeignLoweringAuthority => {
                WorthUiAllocationCatalogActivationDenial::ExecutionPlanAuthorityMismatch
            }
            crate::runtime::active::WorthUiExecutionPlanBundleDenial::OrdinaryPlan(denial) => {
                WorthUiAllocationCatalogActivationDenial::OrdinaryPlan(denial)
            }
            crate::runtime::active::WorthUiExecutionPlanBundleDenial::VirtualizedPlan(denial) => {
                WorthUiAllocationCatalogActivationDenial::VirtualizedPlan(denial)
            }
            crate::runtime::active::WorthUiExecutionPlanBundleDenial::CanvasSpatialPlan(denial) => {
                WorthUiAllocationCatalogActivationDenial::CanvasSpatialPlan(denial)
            }
            crate::runtime::active::WorthUiExecutionPlanBundleDenial::RealtimeOverlayPlan(
                denial,
            ) => WorthUiAllocationCatalogActivationDenial::RealtimeOverlayPlan(denial),
        })?;
        let plan_decision = self
            .active
            .active_plan_ref()
            .classify_candidate(&candidate_bundle);
        match plan_decision {
            crate::runtime::WorthUiExecutablePlanDecision::Denied(denial) => {
                return Err(
                    WorthUiAllocationCatalogActivationDenial::ExecutableEquivalence(denial),
                );
            }
            crate::runtime::WorthUiExecutablePlanDecision::ExactSemanticNoOp(summary) => {
                let application_contract_unchanged = application_publication.as_ref().is_none_or(
                    super::WorthUiPreparedApplicationPublication::
                        permits_execution_plan_semantic_no_op,
                );
                if prepared.operational_meaning_unchanged() && application_contract_unchanged {
                    return Ok(WorthUiPreparedQueryAwarePlanOutcome::SemanticNoOp(
                        Box::new(crate::runtime::WorthUiSemanticNoOpReceipt::new(
                            candidate_bundle.generation_identity().clone(),
                            self.active.generation_identity().clone(),
                            summary,
                            candidate_bundle
                                .cross_lane_receipt()
                                .construction_counters(),
                        )),
                    ));
                }
            }
            crate::runtime::WorthUiExecutablePlanDecision::BoundedChangedRegions(_)
            | crate::runtime::WorthUiExecutablePlanDecision::RebuildRequired(_) => {}
        }
        let query_changes = self
            .active
            .active_plan_ref()
            .query_succession_changes(&candidate_bundle);
        deny_if_certification_interrupted("Query succession")?;
        let query_succession = candidate_query_binding
            .prepare_regional_succession(&self.query_binding, query_changes)
            .map_err(WorthUiAllocationCatalogActivationDenial::QuerySuccession)?;
        let (pending_activation, _committed_input, plan_input) =
            lowering_authority.into_replacement_parts();
        let successor_application_authority =
            pending_activation.candidate_application_authority().clone();
        match prepared.activate(
            self,
            super::committed_allocation_attempt::UiCommittedAllocationActivationInput {
                pending_activation,
                plan_input: &plan_input,
                handle_allocation: &handles,
                candidate_bundle,
                query_succession,
                successor_application_authority,
                successor_planning_authority,
                application_publication,
                boundary,
                lane_parity_report: lane_parity_report.as_ref(),
            },
        ) {
            Ok(activation) => Ok(WorthUiPreparedQueryAwarePlanOutcome::Activation(Box::new(
                super::query_aware_plan_outcome::WorthUiPreparedQueryAwarePlanSwap::new(
                    activation,
                    plan_decision,
                    catalog_successor_receipt,
                ),
            ))),
            Err(denial) => Err(WorthUiAllocationCatalogActivationDenial::Attempt(Box::new(
                denial,
            ))),
        }
    }
}

fn deny_if_certification_interrupted(
    stage: &'static str,
) -> Result<(), WorthUiAllocationCatalogActivationDenial> {
    if super::certification_precommit_interruption(stage) {
        Err(WorthUiAllocationCatalogActivationDenial::CertificationBoundary(stage))
    } else {
        Ok(())
    }
}
