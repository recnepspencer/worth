use crate::diagnostics::{BridgeFailureSource, BridgeReplaySummary, BridgeRouteRecord};
use crate::error::{BridgeErrorContext, BridgeReplayError, BridgeReplayErrorKind};
use crate::facade::{BridgeRouteRequest, RuntimeBridge};
use crate::routing::counters::BridgeRoutingCounters;
use crate::routing::outcome::BridgeRouteOutcomeReference;

use super::BridgeRouteSourceSummary;

pub(crate) fn replay_route_record(
    runtime: &RuntimeBridge,
    record: &BridgeRouteRecord,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<BridgeReplaySummary, BridgeReplayError> {
    let planned_result = match record.contract_proof().route_planning_policy() {
        Some(route_policy) => runtime
            .plan_committed_patch_with_mapping_context_and_route_policy_for_replay(
                BridgeRouteRequest::for_commit(record.source_commit().clone()),
                record.mapping_context().clone(),
                route_policy,
                execution,
            ),
        None => match record.contract_proof().route_planning_policy_digest() {
            Some(route_policy_digest) => runtime
                .plan_committed_patch_with_mapping_context_and_route_policy_digest_for_replay(
                    BridgeRouteRequest::for_commit(record.source_commit().clone()),
                    record.mapping_context().clone(),
                    route_policy_digest,
                    execution,
                ),
            None => runtime.plan_committed_patch_with_mapping_context(
                BridgeRouteRequest::for_commit(record.source_commit().clone()),
                record.mapping_context().clone(),
                execution,
            ),
        },
    };
    let planned = match planned_result {
        Ok(planned) => planned,
        Err(error) => {
            let replay_error = BridgeReplayError::new(
                BridgeReplayErrorKind::RouteMismatch,
                format!("Bridge replay failed to reconstruct the planned route: {error}"),
            )
            .with_context(replay_context(record));
            return Err(reject_replay(
                runtime,
                record,
                BridgeRoutingCounters::default().with_route_replay_mismatch(),
                replay_error,
            ));
        }
    };
    let prepared = planned.into_prepared_delivery();
    let contract_proof = prepared.contract_proof().clone();
    let validated_lowering_plan = prepared.validated_lowering_plan().clone();
    let routing_summary = prepared.routing_summary().clone();
    let counters = *prepared.counters();
    let source_digest = contract_proof.source_digest().clone();
    let replay_counters = counters.with_route_replay_mismatch();
    if routing_summary.route_identity() != record.route_identity() {
        let error = BridgeReplayError::new(
            BridgeReplayErrorKind::RouteMismatch,
            format!(
                "Bridge replay reconstructed route `{}` but original route was `{}`.",
                routing_summary.route_identity().as_str(),
                record.route_identity().as_str()
            ),
        )
        .with_context(
            replay_context(record)
                .with_invalidation_identity(record.invalidation_identity().clone()),
        );
        return Err(reject_replay(runtime, record, replay_counters, error));
    }
    if source_digest != *record.source_digest() {
        let error = BridgeReplayError::new(
            BridgeReplayErrorKind::DigestMismatch,
            format!(
                "Bridge replay reconstructed digest `{}` but original digest was `{}`.",
                source_digest.as_str(),
                record.source_digest().as_str()
            ),
        )
        .with_context(
            replay_context(record)
                .with_invalidation_identity(record.invalidation_identity().clone()),
        );
        return Err(reject_replay(runtime, record, replay_counters, error));
    }

    let lowering_provenance_digest = contract_proof.lowering_provenance_digest().to_owned();
    let lowering_summary_digest = contract_proof.lowering_summary_digest().to_owned();
    let artifact =
        crate::routing::lowering::lower_validated_route(validated_lowering_plan, counters);
    if artifact.invalidation_identity() != record.invalidation_identity() {
        let error = BridgeReplayError::new(
            BridgeReplayErrorKind::InvalidationMismatch,
            format!(
                "Bridge replay reconstructed invalidation `{}` but original invalidation was `{}`.",
                artifact.invalidation_identity().as_str(),
                record.invalidation_identity().as_str()
            ),
        )
        .with_context(
            replay_context(record)
                .with_invalidation_identity(record.invalidation_identity().clone()),
        );
        return Err(reject_replay(runtime, record, replay_counters, error));
    }
    if artifact.subscription_slice_identity() != record.subscription_slice_identity() {
        let error = BridgeReplayError::new(
            BridgeReplayErrorKind::SubscriptionSliceMismatch,
            format!(
                "Bridge replay reconstructed subscription slices `{}` but original slices were `{}`.",
                artifact.subscription_slice_identity().as_str(),
                record.subscription_slice_identity().as_str()
            ),
        )
        .with_context(
            replay_context(record)
                .with_invalidation_identity(record.invalidation_identity().clone())
                .with_subscription_slice_identity(record.subscription_slice_identity().clone()),
        );
        return Err(reject_replay(runtime, record, replay_counters, error));
    }

    if contract_proof.planning_provenance_digest()
        != record.contract_proof().planning_provenance_digest()
        || contract_proof.planning_summary_digest()
            != record.contract_proof().planning_summary_digest()
        || contract_proof.route_planning_policy_digest()
            != record.contract_proof().route_planning_policy_digest()
    {
        let error = BridgeReplayError::new(
            BridgeReplayErrorKind::PlanningContractMismatch,
            format!(
                "Bridge replay reconstructed planning contract `{}` / `{}` / policy {:?} but original planning contract was `{}` / `{}` / policy {:?}.",
                contract_proof.planning_provenance_digest(),
                contract_proof.planning_summary_digest(),
                contract_proof.route_planning_policy_digest(),
                record.contract_proof().planning_provenance_digest(),
                record.contract_proof().planning_summary_digest(),
                record.contract_proof().route_planning_policy_digest()
            ),
        )
        .with_context(replay_context(record).with_invalidation_identity(record.invalidation_identity().clone()));
        return Err(reject_replay(runtime, record, replay_counters, error));
    }

    if artifact.subscription_slice_identity() == record.subscription_slice_identity()
        && (artifact.invalidation_identity() == record.invalidation_identity())
        && (lowering_provenance_digest != record.contract_proof().lowering_provenance_digest()
            || lowering_summary_digest != record.contract_proof().lowering_summary_digest())
    {
        let error = BridgeReplayError::new(
            BridgeReplayErrorKind::LoweringContractMismatch,
            format!(
                "Bridge replay reconstructed lowering contract `{}` / `{}` but original lowering contract was `{}` / `{}`.",
                lowering_provenance_digest,
                lowering_summary_digest,
                record.contract_proof().lowering_provenance_digest(),
                record.contract_proof().lowering_summary_digest()
            ),
        )
        .with_context(replay_context(record).with_invalidation_identity(record.invalidation_identity().clone()));
        return Err(reject_replay(runtime, record, replay_counters, error));
    }

    Ok(BridgeReplaySummary::new(BridgeRouteOutcomeReference::new(
        routing_summary.route_identity().clone(),
        artifact.invalidation_identity().clone(),
        BridgeRouteSourceSummary::new(
            routing_summary.source_branch().clone(),
            routing_summary.source_commit().clone(),
            routing_summary.source_patch().clone(),
            routing_summary.source_snapshot().clone(),
        ),
        artifact.subscription_slice_identity().clone(),
    )))
}

fn replay_context(record: &BridgeRouteRecord) -> BridgeErrorContext {
    BridgeErrorContext::replay(
        record.route_identity().clone(),
        record.source_snapshot().clone(),
    )
}

fn replay_failure_source(
    record: &BridgeRouteRecord,
    counters: BridgeRoutingCounters,
) -> BridgeFailureSource {
    BridgeFailureSource::new(
        record.source_commit().clone(),
        record.source_patch().clone(),
        record.source_snapshot().clone(),
        counters,
    )
    .with_route_identity(record.route_identity().clone())
    .with_invalidation_identity(record.invalidation_identity().clone())
    .with_subscription_slice_identity(record.subscription_slice_identity().clone())
    .with_contract_proof(record.contract_proof().clone())
}

fn reject_replay(
    runtime: &RuntimeBridge,
    record: &BridgeRouteRecord,
    counters: BridgeRoutingCounters,
    error: BridgeReplayError,
) -> BridgeReplayError {
    runtime
        .diagnostic_sink
        .record_replay_failure(replay_failure_source(record, counters), &error);
    error
}
