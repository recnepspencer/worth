use std::sync::atomic::{AtomicU64, Ordering};

use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::{
    WorthQueryConditionalEvaluationStop, WorthQueryOperationExecutionCounters,
};

use super::conditional_core::{
    WorthQueryLifecycleConditionalCoreStop, WorthQueryLifecycleConditionalStopClass,
};
use super::source::WorthQueryProjectionLifecycleSource;
use super::{WorthQueryProjectionPromotionCounters, WorthQueryProjectionPromotionDenialKind};

static NEXT_OWNER_DELIVERY_ATTEMPT: AtomicU64 = AtomicU64::new(1);

pub(super) struct WorthQueryOwnerConditionalEvaluation {
    pub(super) provenance: crate::domain_installation::WorthQueryConditionalProvenance,
    pub(super) counters: WorthQueryOperationExecutionCounters,
    pub(super) retained_seed:
        crate::domain_installation::conditional_execution::WorthQueryRetainedConditionalDecision,
}

pub(super) fn evaluate_owner_delivered_conditional<
    D,
    O,
    F,
    L: BasisOperationLane,
    S: WorthQueryProjectionLifecycleSource<D, O, F, L>,
>(
    source: &S,
    delivery: &worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt,
    workspace: &mut crate::runtime::WorthQueryWorkspace,
) -> Result<WorthQueryOwnerConditionalEvaluation, WorthQueryLifecycleConditionalCoreStop> {
    let location =
        crate::domain_installation::conditional_execution::query_location_from_bridge_candidate(
            delivery.change_set().dependency(),
        );
    let mut counters = WorthQueryProjectionPromotionCounters::default();
    counters.lifecycle_attempts = 1;
    require_owner_conditional_location(source, &location, counters)?;
    let snapshot = owner_snapshot(delivery, counters)?;
    let attempt = NEXT_OWNER_DELIVERY_ATTEMPT.fetch_add(1, Ordering::Relaxed);
    let execution_identity = owner_execution_identity(delivery, &snapshot, attempt);
    let scope = owner_conditional_scope(&location);
    let (resources, resource_evidence) = match &location {
        worth_query_installation::facade::WorthQueryConditionalNodeLocation::Operation {
            ..
        } => (
            source.operation_resources(),
            source.operation_resource_evidence(),
        ),
        worth_query_installation::facade::WorthQueryConditionalNodeLocation::WorkflowStage {
            stage_identity,
            ..
        } => (
            source.stage_resources(stage_identity).ok_or_else(|| {
                owner_reentry_stop(
                    "owner conditional stage has no admitted resource plan",
                    counters,
                )
            })?,
            source
                .stage_resource_evidence(stage_identity)
                .ok_or_else(|| {
                    owner_reentry_stop(
                        "owner conditional stage has no admitted resource evidence",
                        counters,
                    )
                })?,
        ),
    };
    let mut operation_counters = WorthQueryOperationExecutionCounters::default();
    let provenance =
        crate::domain_installation::conditional_execution::evaluate_owner_impact_conditionals(
            source.bound_operation(),
            crate::domain_installation::WorthQueryOwnerImpactConditionalEvaluationPass {
                evaluation: crate::domain_installation::WorthQueryConditionalEvaluationPass {
                    workspace,
                    snapshot: &snapshot,
                    execution_identity: &execution_identity,
                    scope,
                    workflow_run_identity: source.workflow_run_identity(),
                    attempt,
                    resources,
                    resource_evidence,
                    counters: &mut operation_counters,
                },
                location: &location,
            },
        )
        .map_err(|stop| owner_evaluation_stop(stop, counters))?
        .into_iter()
        .find(|provenance| provenance.location() == &location)
        .ok_or_else(|| {
            owner_reentry_stop(
                "owner delivery conditional was not evaluated by the retained operation",
                counters,
            )
        })?;
    let retained_seed = provenance.retain_for_reentry();
    Ok(WorthQueryOwnerConditionalEvaluation {
        provenance,
        counters: operation_counters,
        retained_seed,
    })
}

pub(super) struct WorthQueryOwnerConditionalReentryPass<'a, S> {
    pub(super) source: &'a S,
    pub(super) delivery: &'a worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt,
    pub(super) seed: &'a crate::domain_installation::conditional_execution::WorthQueryRetainedConditionalDecision,
    pub(super) workspace: &'a crate::runtime::WorthQueryWorkspace,
    pub(super) work: &'a mut super::WorthQueryLiveProjectionRefreshWork,
}

pub(super) fn reenter_owner_delivered_conditional<
    D,
    O,
    F,
    L: BasisOperationLane,
    S: WorthQueryProjectionLifecycleSource<D, O, F, L>,
>(
    pass: WorthQueryOwnerConditionalReentryPass<'_, S>,
) -> Result<
    crate::domain_installation::WorthQueryConditionalProvenance,
    WorthQueryLifecycleConditionalCoreStop,
> {
    let WorthQueryOwnerConditionalReentryPass {
        source,
        delivery,
        seed,
        workspace,
        work,
    } = pass;
    work.begin_conditional_decision_reentry();
    let location =
        crate::domain_installation::conditional_execution::query_location_from_bridge_candidate(
            delivery.change_set().dependency(),
        );
    let counters = WorthQueryProjectionPromotionCounters::default();
    require_owner_conditional_location(source, &location, counters)?;
    let snapshot = owner_snapshot(delivery, counters)?;
    let bound = source.bound_operation();
    let product = seed
        .admit_product(bound.product().map_err(|_| {
            owner_reentry_stop("conditional reentry requires product custody", counters)
        })?)
        .map_err(|_| {
            owner_reentry_stop(
                "retained conditional belongs to a different product occurrence",
                counters,
            )
        })?;
    let node = installed_owner_conditional_node(bound, &location, counters)?;
    let authority =
        crate::domain_installation::conditional_execution::admit_conditional_authority(bound, node)
            .map_err(|_| {
                owner_reentry_stop("conditional authority continuity was denied", counters)
            })?;
    let bridge = workspace
        .reenter_retained_conditional_decision(
            &product,
            worth_runtime_bridge::facade::BridgeConditionalDecisionReentryRequest {
                seed: seed.seed(),
                lowering: &node.lowering,
                query_binding_identity: bound.binding_identity(),
                query_capability_identity: bound.capability_identity(),
                snapshot_identity: snapshot.evidence_identity().as_str(),
                bridge_snapshot_identity: Some(product.bridge_snapshot_identity()),
            },
        )
        .map_err(|(detail, reentry_counters)| {
            work.retain_conditional_decision_reentry(reentry_counters);
            owner_reentry_stop(&detail, counters)
        })?;
    work.retain_conditional_decision_reentry(bridge.reentry_counters());
    admit_reentered_query_decision(ReenteredQueryDecisionAdmission {
        bound,
        authority,
        product,
        bridge,
        snapshot: &snapshot,
        counters,
    })
}

fn require_owner_conditional_location<D, O, F, L, S>(
    source: &S,
    location: &worth_query_installation::facade::WorthQueryConditionalNodeLocation,
    counters: WorthQueryProjectionPromotionCounters,
) -> Result<(), WorthQueryLifecycleConditionalCoreStop>
where
    L: BasisOperationLane,
    S: WorthQueryProjectionLifecycleSource<D, O, F, L>,
{
    if source.admits_conditional_location(location) {
        return Ok(());
    }
    Err(owner_reentry_stop(
        "owner delivery conditional is outside the retained live source",
        counters,
    ))
}

fn owner_conditional_scope(
    location: &worth_query_installation::facade::WorthQueryConditionalNodeLocation,
) -> crate::domain_installation::WorthQueryConditionalEvaluationScope<'_> {
    match location {
        worth_query_installation::facade::WorthQueryConditionalNodeLocation::Operation {
            ..
        } => crate::domain_installation::WorthQueryConditionalEvaluationScope::Operation,
        worth_query_installation::facade::WorthQueryConditionalNodeLocation::WorkflowStage {
            stage_identity,
            ..
        } => crate::domain_installation::WorthQueryConditionalEvaluationScope::WorkflowStage(
            stage_identity,
        ),
    }
}

fn installed_owner_conditional_node<'a, D, O, F, L: BasisOperationLane>(
    bound: &'a crate::domain_installation::WorthQueryBoundDomainOperation<D, O, F, L>,
    location: &worth_query_installation::facade::WorthQueryConditionalNodeLocation,
    counters: WorthQueryProjectionPromotionCounters,
) -> Result<
    &'a crate::domain_installation::WorthQueryInstalledConditionalNode,
    WorthQueryLifecycleConditionalCoreStop,
> {
    bound
        .conditional_nodes()
        .iter()
        .find(|node| &node.location == location)
        .map(std::sync::Arc::as_ref)
        .ok_or_else(|| {
            owner_reentry_stop(
                "owner delivery conditional is not installed for the retained operation",
                counters,
            )
        })
}

struct ReenteredQueryDecisionAdmission<'a, D, O, F, L: BasisOperationLane> {
    bound: &'a crate::domain_installation::WorthQueryBoundDomainOperation<D, O, F, L>,
    authority:
        crate::domain_installation::conditional_execution::WorthQueryConditionalAuthorityAdmission,
    product:
        std::sync::Arc<worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease>,
    bridge: worth_runtime_bridge::facade::BridgeConditionalDecisionEvidence,
    snapshot: &'a crate::memory_workspace::WorthQuerySnapshotIdentity,
    counters: WorthQueryProjectionPromotionCounters,
}

fn admit_reentered_query_decision<D, O, F, L: BasisOperationLane>(
    admission: ReenteredQueryDecisionAdmission<'_, D, O, F, L>,
) -> Result<
    crate::domain_installation::WorthQueryConditionalProvenance,
    WorthQueryLifecycleConditionalCoreStop,
> {
    let execution_identity = admission.bridge.signal_execution_projection().to_owned();
    let attempt = admission.bridge.attempt();
    crate::domain_installation::conditional_execution::admit_conditional_decision(
        admission.bound,
        admission.authority,
        admission.product,
        admission.bridge,
        admission.snapshot.evidence_identity().as_str(),
        &execution_identity,
        attempt,
    )
    .map_err(|_| {
        owner_reentry_stop(
            "retained conditional failed exact Query reentry",
            admission.counters,
        )
    })
}

pub(super) fn require_settled_owner_decision(
    provenance: &crate::domain_installation::WorthQueryConditionalProvenance,
) -> Result<(), WorthQueryLifecycleConditionalCoreStop> {
    use crate::domain_installation::WorthQueryConditionalOutcomeClass as Outcome;
    if matches!(
        provenance.class(),
        Outcome::DeferredByCondition | Outcome::DeferredTemporal | Outcome::DeferredOnDemand
    ) {
        return Err(owner_stop(
            WorthQueryLifecycleConditionalStopClass::Deferred,
            WorthQueryProjectionPromotionDenialKind::ConditionalDeferred,
            "owner delivery conditional evaluation was deferred",
            WorthQueryProjectionPromotionCounters::default(),
        ));
    }
    Ok(())
}

fn owner_snapshot(
    delivery: &worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt,
    counters: WorthQueryProjectionPromotionCounters,
) -> Result<
    crate::memory_workspace::WorthQuerySnapshotIdentity,
    WorthQueryLifecycleConditionalCoreStop,
> {
    crate::memory_workspace::WorthQuerySnapshotIdentity::from_admitted_bridge_snapshot_identity(
        delivery.change_set().admitted_snapshot_identity(),
    )
    .ok_or_else(|| {
        owner_stop(
            WorthQueryLifecycleConditionalStopClass::Failed,
            WorthQueryProjectionPromotionDenialKind::ConditionalEvaluation,
            "owner delivery lacks a relational snapshot identity",
            counters,
        )
    })
}

fn owner_execution_identity(
    delivery: &worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt,
    snapshot: &crate::memory_workspace::WorthQuerySnapshotIdentity,
    attempt: u64,
) -> String {
    crate::identity::hash_parts(&[
        "worth_query_owner_delivery_conditional_v2".into(),
        format!(
            "commit:{}",
            delivery
                .change_set()
                .commit_identity()
                .bridge_admission_evidence()
                .terminal_projection_for_reporting()
        ),
        format!("snapshot:{}", snapshot.evidence_identity().as_str()),
        format!("attempt:{attempt}"),
    ])
}

fn owner_reentry_stop(
    detail: &str,
    counters: WorthQueryProjectionPromotionCounters,
) -> WorthQueryLifecycleConditionalCoreStop {
    owner_stop(
        WorthQueryLifecycleConditionalStopClass::Denied,
        WorthQueryProjectionPromotionDenialKind::ConditionalReentry,
        detail,
        counters,
    )
}

fn owner_stop(
    class: WorthQueryLifecycleConditionalStopClass,
    kind: WorthQueryProjectionPromotionDenialKind,
    detail: &str,
    counters: WorthQueryProjectionPromotionCounters,
) -> WorthQueryLifecycleConditionalCoreStop {
    WorthQueryLifecycleConditionalCoreStop {
        class,
        kind,
        detail: detail.into(),
        counters,
    }
}

fn owner_evaluation_stop(
    stop: WorthQueryConditionalEvaluationStop,
    counters: WorthQueryProjectionPromotionCounters,
) -> WorthQueryLifecycleConditionalCoreStop {
    match stop {
        WorthQueryConditionalEvaluationStop::Deferred(_) => owner_stop(
            WorthQueryLifecycleConditionalStopClass::Deferred,
            WorthQueryProjectionPromotionDenialKind::ConditionalDeferred,
            "owner delivery conditional evaluation was deferred",
            counters,
        ),
        WorthQueryConditionalEvaluationStop::Failed { detail, .. } => owner_stop(
            WorthQueryLifecycleConditionalStopClass::Failed,
            WorthQueryProjectionPromotionDenialKind::ConditionalEvaluation,
            &detail,
            counters,
        ),
        WorthQueryConditionalEvaluationStop::Reentry(_) => owner_reentry_stop(
            "owner delivery conditional failed exact Query reentry",
            counters,
        ),
    }
}
