use std::sync::atomic::{AtomicU64, Ordering};

use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::{
    WorthQueryConditionalEvaluationStop, WorthQueryOperationExecutionCounters,
};

use super::source::WorthQueryProjectionLifecycleSource;
use super::{WorthQueryProjectionPromotionCounters, WorthQueryProjectionPromotionDenialKind};

static NEXT_PROJECTION_LIFECYCLE_ATTEMPT: AtomicU64 = AtomicU64::new(1);

pub(super) struct WorthQueryLifecycleConditionalCoreReady {
    pub(super) counters: WorthQueryProjectionPromotionCounters,
    pub(super) attempt: u64,
    pub(super) operational_identity: String,
    pub(super) resource_name: String,
    pub(super) conditional_provenance:
        Vec<crate::domain_installation::WorthQueryConditionalProvenance>,
}

pub(super) enum WorthQueryLifecycleConditionalStopClass {
    Deferred,
    Denied,
    Failed,
}

pub(super) struct WorthQueryLifecycleConditionalCoreStop {
    pub(super) class: WorthQueryLifecycleConditionalStopClass,
    pub(super) kind: WorthQueryProjectionPromotionDenialKind,
    pub(super) detail: String,
    pub(super) counters: WorthQueryProjectionPromotionCounters,
}

pub(super) fn evaluate_fresh_lifecycle_conditionals<
    D,
    O,
    F,
    L: BasisOperationLane,
    S: WorthQueryProjectionLifecycleSource<D, O, F, L>,
>(
    source: &S,
    workspace: &mut crate::runtime::WorthQueryWorkspace,
    mut counters: WorthQueryProjectionPromotionCounters,
    identity_family: &'static str,
) -> Result<WorthQueryLifecycleConditionalCoreReady, WorthQueryLifecycleConditionalCoreStop> {
    let attempt = NEXT_PROJECTION_LIFECYCLE_ATTEMPT.fetch_add(1, Ordering::Relaxed);
    let snapshot =
        workspace
            .snapshot_identity()
            .map_err(|denial| WorthQueryLifecycleConditionalCoreStop {
                class: WorthQueryLifecycleConditionalStopClass::Denied,
                kind: WorthQueryProjectionPromotionDenialKind::Handle(denial),
                detail: denial.to_string(),
                counters: counters.clone(),
            })?;
    let operational_identity =
        lifecycle_identity(source, workspace, &snapshot, attempt, identity_family);
    let resource_name = format!("worth-query-installed-projection-{operational_identity}");
    counters.lifecycle_attempts = 1;
    let mut provenance = Vec::new();
    let publication_stage = source.publication_stage_identity().map(str::to_owned);
    let mut evaluation = WorthQueryLifecycleScopeEvaluation {
        source,
        workspace,
        snapshot: &snapshot,
        operational_identity: &operational_identity,
        attempt,
        counters: &mut counters,
        retained: &mut provenance,
    };
    evaluate_scope(&mut evaluation, None)?;
    if let Some(stage) = publication_stage.as_deref() {
        evaluate_scope(&mut evaluation, Some(stage))?;
    }
    counters.fresh_conditional_decisions = provenance.len();
    Ok(WorthQueryLifecycleConditionalCoreReady {
        counters,
        attempt,
        operational_identity,
        resource_name,
        conditional_provenance: provenance,
    })
}

struct WorthQueryLifecycleScopeEvaluation<'a, S> {
    source: &'a S,
    workspace: &'a mut crate::runtime::WorthQueryWorkspace,
    snapshot: &'a crate::memory_workspace::WorthQuerySnapshotIdentity,
    operational_identity: &'a str,
    attempt: u64,
    counters: &'a mut WorthQueryProjectionPromotionCounters,
    retained: &'a mut Vec<crate::domain_installation::WorthQueryConditionalProvenance>,
}

fn evaluate_scope<D, O, F, L: BasisOperationLane, S>(
    evaluation: &mut WorthQueryLifecycleScopeEvaluation<'_, S>,
    stage_identity: Option<&str>,
) -> Result<(), WorthQueryLifecycleConditionalCoreStop>
where
    S: WorthQueryProjectionLifecycleSource<D, O, F, L>,
{
    let scope = stage_identity.unwrap_or("operation");
    let execution_identity = format!("{}:{scope}", evaluation.operational_identity);
    let (resources, resource_evidence) = match stage_identity {
        Some(stage_identity) => (
            evaluation
                .source
                .stage_resources(stage_identity)
                .ok_or_else(|| missing_resource_evidence(evaluation.counters))?,
            evaluation
                .source
                .stage_resource_evidence(stage_identity)
                .ok_or_else(|| missing_resource_evidence(evaluation.counters))?,
        ),
        None => (
            evaluation.source.operation_resources(),
            evaluation.source.operation_resource_evidence(),
        ),
    };
    let mut operation_counters = WorthQueryOperationExecutionCounters::default();
    let outcome =
        crate::domain_installation::conditional_execution::evaluate_settled_projection_conditionals(
            evaluation.source.bound_operation(),
            crate::domain_installation::WorthQueryConditionalEvaluationPass {
                workspace: evaluation.workspace,
                snapshot: evaluation.snapshot,
                execution_identity: &execution_identity,
                scope: match stage_identity {
                    Some(stage_identity) => crate::domain_installation::WorthQueryConditionalEvaluationScope::WorkflowStage(stage_identity),
                    None => crate::domain_installation::WorthQueryConditionalEvaluationScope::Operation,
                },
                workflow_run_identity: evaluation.source.workflow_run_identity(),
                attempt: evaluation.attempt,
                resources,
                resource_evidence,
                counters: &mut operation_counters,
            },
        );
    retain_conditional_counters(evaluation.counters, operation_counters);
    match outcome {
        Ok(mut provenance) => evaluation.retained.append(&mut provenance),
        Err(WorthQueryConditionalEvaluationStop::Deferred(mut provenance)) => {
            evaluation.retained.append(&mut provenance);
            evaluation.counters.fresh_conditional_decisions = evaluation.retained.len();
            return Err(WorthQueryLifecycleConditionalCoreStop {
                class: WorthQueryLifecycleConditionalStopClass::Deferred,
                kind: WorthQueryProjectionPromotionDenialKind::ConditionalDeferred,
                detail: "fresh conditional evaluation did not admit live promotion".into(),
                counters: *evaluation.counters,
            });
        }
        Err(WorthQueryConditionalEvaluationStop::Failed { detail, .. }) => {
            return Err(WorthQueryLifecycleConditionalCoreStop {
                class: WorthQueryLifecycleConditionalStopClass::Failed,
                kind: WorthQueryProjectionPromotionDenialKind::ConditionalEvaluation,
                detail,
                counters: *evaluation.counters,
            });
        }
        Err(WorthQueryConditionalEvaluationStop::Reentry(_)) => {
            return Err(WorthQueryLifecycleConditionalCoreStop {
                class: WorthQueryLifecycleConditionalStopClass::Denied,
                kind: WorthQueryProjectionPromotionDenialKind::ConditionalReentry,
                detail: "Signal decision did not re-enter through this exact bound projection"
                    .into(),
                counters: *evaluation.counters,
            });
        }
    }
    Ok(())
}

fn missing_resource_evidence(
    counters: &WorthQueryProjectionPromotionCounters,
) -> WorthQueryLifecycleConditionalCoreStop {
    WorthQueryLifecycleConditionalCoreStop {
        class: WorthQueryLifecycleConditionalStopClass::Failed,
        kind: WorthQueryProjectionPromotionDenialKind::ConditionalReentry,
        detail: "retained conditional scope has no admitted execution resource evidence".into(),
        counters: *counters,
    }
}

fn lifecycle_identity<D, O, F, L: BasisOperationLane, S>(
    source: &S,
    workspace: &crate::runtime::WorthQueryWorkspace,
    snapshot: &crate::memory_workspace::WorthQuerySnapshotIdentity,
    attempt: u64,
    family: &'static str,
) -> String
where
    S: WorthQueryProjectionLifecycleSource<D, O, F, L>,
{
    let bound = source.bound_operation();
    crate::identity::hash_parts(&[
        family.into(),
        format!("settled:{}", source.identity()),
        format!("binding:{}", bound.binding_identity()),
        format!("capability:{}", bound.capability_identity()),
        format!("basis:{}", bound.basis().capability_digest()),
        format!(
            "runtime:{}",
            workspace.runtime_authority_identity().as_u64()
        ),
        format!(
            "generation:{}",
            bound.operation().installation_generation().ordinal()
        ),
        format!("snapshot:{}", snapshot.evidence_identity().as_str()),
        format!("attempt:{attempt}"),
    ])
}

fn retain_conditional_counters(
    target: &mut WorthQueryProjectionPromotionCounters,
    source: WorthQueryOperationExecutionCounters,
) {
    target.conditional_dependency_checks += source.conditional_dependency_checks;
    target.conditional_semantic_reads += source.conditional_semantic_reads;
    target.conditional_condition_checks += source.conditional_condition_checks;
    target.conditional_condition_deferrals += source.conditional_condition_deferrals;
    target.conditional_temporal_deferrals += source.conditional_temporal_deferrals;
    target.conditional_on_demand_deferrals += source.conditional_on_demand_deferrals;
    target.conditional_comparator_checks += source.conditional_comparator_checks;
    target.conditional_compute_contacts += source.conditional_compute_contacts;
    target.conditional_reverted_clean_outcomes += source.conditional_reverted_clean_outcomes;
    target.conditional_semantic_changes += source.conditional_semantic_changes;
    target.conditional_reuse_checks += source.conditional_reuse_checks;
    target.conditional_decisions_delivered += source.conditional_decisions_delivered;
}
