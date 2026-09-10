use crate::basis_lifecycle::BasisOperationLane;

use super::reentry::admit_conditional_decision;
use super::{
    WorthQueryConditionalComputeContext, WorthQueryConditionalProvenance,
    WorthQueryInstalledConditionalNode,
};

pub(crate) enum WorthQueryConditionalEvaluationStop {
    Deferred(Vec<WorthQueryConditionalProvenance>),
    Failed {
        kind: worth_runtime_bridge::facade::BridgeConditionalDenialKind,
        detail: String,
    },
    Reentry(super::WorthQueryConditionalAdmissionDenial),
}

#[derive(Clone, Copy)]
pub(crate) enum WorthQueryConditionalEvaluationScope<'a> {
    Operation,
    WorkflowStage(&'a str),
}

pub(crate) struct WorthQueryConditionalEvaluationPass<'a> {
    pub(crate) workspace: &'a crate::runtime::WorthQueryWorkspace,
    pub(crate) snapshot: &'a crate::memory_workspace::WorthQuerySnapshotIdentity,
    pub(crate) execution_identity: &'a str,
    pub(crate) scope: WorthQueryConditionalEvaluationScope<'a>,
    pub(crate) workflow_run_identity: Option<&'a str>,
    pub(crate) attempt: u64,
    pub(crate) resources: &'a super::super::WorthQueryAdmittedExecutionResourcePlan,
    pub(crate) resource_evidence: &'a super::super::WorthQueryExecutionResourceAttemptEvidence,
    pub(crate) counters: &'a mut super::super::WorthQueryOperationExecutionCounters,
}

pub(crate) struct WorthQueryOwnerImpactConditionalEvaluationPass<'a> {
    pub(crate) evaluation: WorthQueryConditionalEvaluationPass<'a>,
    pub(crate) location: &'a worth_query_installation::facade::WorthQueryConditionalNodeLocation,
}

pub(crate) fn evaluate_bound_conditionals<D, O, F, L: BasisOperationLane>(
    bound: &super::super::WorthQueryBoundDomainOperation<D, O, F, L>,
    evaluation: WorthQueryConditionalEvaluationPass<'_>,
) -> Result<Vec<WorthQueryConditionalProvenance>, WorthQueryConditionalEvaluationStop> {
    evaluate_conditionals(
        bound,
        evaluation,
        ConditionalAcceptance::FreshComputation,
        None,
    )
}

pub(crate) fn evaluate_settled_projection_conditionals<D, O, F, L: BasisOperationLane>(
    bound: &super::super::WorthQueryBoundDomainOperation<D, O, F, L>,
    evaluation: WorthQueryConditionalEvaluationPass<'_>,
) -> Result<Vec<WorthQueryConditionalProvenance>, WorthQueryConditionalEvaluationStop> {
    evaluate_conditionals(
        bound,
        evaluation,
        ConditionalAcceptance::SettledOutputContinuity,
        None,
    )
}

pub(crate) fn evaluate_owner_impact_conditionals<D, O, F, L: BasisOperationLane>(
    bound: &super::super::WorthQueryBoundDomainOperation<D, O, F, L>,
    owner_impact: WorthQueryOwnerImpactConditionalEvaluationPass<'_>,
) -> Result<Vec<WorthQueryConditionalProvenance>, WorthQueryConditionalEvaluationStop> {
    let WorthQueryOwnerImpactConditionalEvaluationPass {
        evaluation,
        location,
    } = owner_impact;
    evaluate_conditionals(
        bound,
        evaluation,
        ConditionalAcceptance::OwnerImpactObservation,
        Some(location),
    )
}

#[derive(Clone, Copy)]
enum ConditionalAcceptance {
    FreshComputation,
    SettledOutputContinuity,
    OwnerImpactObservation,
}

impl ConditionalAcceptance {
    fn admits(self, provenance: &WorthQueryConditionalProvenance) -> bool {
        match self {
            Self::FreshComputation => {
                provenance.class() == super::WorthQueryConditionalOutcomeClass::ComputedChanged
            }
            Self::SettledOutputContinuity => matches!(
                provenance.class(),
                super::WorthQueryConditionalOutcomeClass::ComputedChanged
                    | super::WorthQueryConditionalOutcomeClass::ComputedRevertedClean
                    | super::WorthQueryConditionalOutcomeClass::DependencyUnchanged
            ),
            Self::OwnerImpactObservation => true,
        }
    }
}

fn evaluate_conditionals<D, O, F, L: BasisOperationLane>(
    bound: &super::super::WorthQueryBoundDomainOperation<D, O, F, L>,
    mut evaluation: WorthQueryConditionalEvaluationPass<'_>,
    acceptance: ConditionalAcceptance,
    exact_location: Option<&worth_query_installation::facade::WorthQueryConditionalNodeLocation>,
) -> Result<Vec<WorthQueryConditionalProvenance>, WorthQueryConditionalEvaluationStop> {
    let mut admitted = Vec::new();
    let scope = evaluation.scope;
    for node in bound.conditional_nodes().iter().filter(|node| {
        exact_location.map_or_else(
            || location_matches(node, scope),
            |location| &node.location == location,
        )
    }) {
        let provenance = evaluate_installed_conditional_node(bound, node, &mut evaluation)?;
        let admitted_for_lane = acceptance.admits(&provenance);
        admitted.push(provenance);
        if !admitted_for_lane {
            return Err(WorthQueryConditionalEvaluationStop::Deferred(admitted));
        }
    }
    Ok(admitted)
}

pub(crate) fn evaluate_installed_conditional_node<D, O, F, L: BasisOperationLane>(
    bound: &super::super::WorthQueryBoundDomainOperation<D, O, F, L>,
    node: &WorthQueryInstalledConditionalNode,
    evaluation: &mut WorthQueryConditionalEvaluationPass<'_>,
) -> Result<WorthQueryConditionalProvenance, WorthQueryConditionalEvaluationStop> {
    evaluate_conditional_node(bound, node, evaluation)
}

fn evaluate_conditional_node<D, O, F, L: BasisOperationLane>(
    bound: &super::super::WorthQueryBoundDomainOperation<D, O, F, L>,
    node: &WorthQueryInstalledConditionalNode,
    evaluation: &mut WorthQueryConditionalEvaluationPass<'_>,
) -> Result<WorthQueryConditionalProvenance, WorthQueryConditionalEvaluationStop> {
    let authority = super::reentry::admit_conditional_authority(bound, node)
        .map_err(WorthQueryConditionalEvaluationStop::Reentry)?;
    let product = bound.product();
    let snapshot_identity = evaluation.snapshot.evidence_identity();
    let mut context =
        WorthQueryConditionalComputeContext::new(super::WorthQueryConditionalComputeContextParts {
            location: node.location.clone(),
            operation_identity: bound.definition().canonical_identity().to_string(),
            binding_identity: bound.binding_identity().to_string(),
            basis_identity: bound.basis().capability_digest().to_string(),
            workflow_run_identity: evaluation.workflow_run_identity.map(str::to_string),
            snapshot_identity: snapshot_identity.as_str().to_string(),
            attempt: evaluation.attempt,
            execution_resources: evaluation.resource_evidence.clone(),
            resource_envelope: evaluation.resources.shared_envelope(),
        });
    let bridge = evaluation
        .workspace
        .execute_installed_conditional(
            product,
            worth_runtime_bridge::facade::BridgeConditionalExecutionRequest {
                lowering: &node.lowering,
                query_binding_identity: bound.binding_identity(),
                query_capability_identity: bound.capability_identity(),
                snapshot_identity: snapshot_identity.as_str(),
                truth_branch_identity: None,
                bridge_snapshot_identity: (node.lowering.correspondence_count() != 0)
                    .then(|| product.bridge_snapshot_identity()),
                execution_identity: evaluation.execution_identity,
                attempt: evaluation.attempt,
            },
            &mut context,
        )
        .map_err(
            |(kind, detail, signal_counters, semantic_observation_reads)| {
                retain_conditional_counters(evaluation.counters, signal_counters);
                evaluation.counters.conditional_semantic_reads += semantic_observation_reads;
                WorthQueryConditionalEvaluationStop::Failed { kind, detail }
            },
        )?;
    let (bridge, product) = bridge.into_parts();
    retain_conditional_evidence_counters(evaluation.counters, &bridge);
    admit_conditional_decision(
        bound,
        authority,
        product,
        bridge,
        snapshot_identity.as_str(),
        evaluation.execution_identity,
        evaluation.attempt,
    )
    .map_err(WorthQueryConditionalEvaluationStop::Reentry)
}

fn retain_conditional_evidence_counters(
    counters: &mut super::super::WorthQueryOperationExecutionCounters,
    bridge: &worth_runtime_bridge::facade::BridgeConditionalDecisionEvidence,
) {
    counters.conditional_semantic_reads += bridge.semantic_observation_reads();
    retain_conditional_counters(counters, bridge.signal().counters());
}

fn retain_conditional_counters(
    counters: &mut super::super::WorthQueryOperationExecutionCounters,
    signal: worth_signal::facade::SignalConditionalDecisionCounters,
) {
    counters.conditional_request_admission_checks += signal.request_admission_checks;
    counters.conditional_contract_lookups += signal.contract_lookups;
    counters.conditional_dependency_observation_reads += signal.dependency_observation_reads;
    counters.conditional_dependency_checks += signal.dependency_version_checks;
    counters.conditional_condition_checks += signal.condition_checks;
    counters.conditional_condition_deferrals += signal.condition_deferrals;
    counters.conditional_temporal_deferrals += signal.temporal_deferrals;
    counters.conditional_on_demand_deferrals += signal.on_demand_deferrals;
    counters.conditional_comparator_checks += signal.comparator_checks;
    counters.conditional_compute_contacts += signal.compute_contacts;
    counters.conditional_output_version_reads += signal.output_version_reads;
    counters.conditional_runtime_dependency_edges_captured +=
        signal.runtime_dependency_edges_captured;
    counters.conditional_application_contacts += signal.application_contacts;
    counters.conditional_semantic_classifications += signal.semantic_classifications;
    counters.conditional_reverted_clean_outcomes += signal.reverted_clean_outcomes;
    counters.conditional_semantic_changes += signal.semantic_changes;
    counters.conditional_reuse_checks += signal.reuse_checks;
    counters.conditional_decisions_delivered += signal.decisions_delivered;
}

fn location_matches(
    node: &WorthQueryInstalledConditionalNode,
    scope: WorthQueryConditionalEvaluationScope<'_>,
) -> bool {
    match (&node.location, scope) {
        (
            worth_query_installation::facade::WorthQueryConditionalNodeLocation::Operation {
                ..
            },
            WorthQueryConditionalEvaluationScope::Operation,
        ) => true,
        (
            worth_query_installation::facade::WorthQueryConditionalNodeLocation::WorkflowStage {
                stage_identity: installed,
                ..
            },
            WorthQueryConditionalEvaluationScope::WorkflowStage(expected),
        ) => installed == expected,
        _ => false,
    }
}
