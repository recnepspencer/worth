use crate::diagnostics::BridgeRouteRecord;
use crate::error::{BridgeDeliveryError, BridgeDeliveryErrorKind};
use crate::facade::RuntimeBridge;
use crate::routing::outcome::BridgeRouteOutcomeReference;
use crate::routing::result::{
    BridgeBulkResultSummary, BridgeBulkWorkloadResult, BridgeRouteResult, BridgeRouteResultSummary,
};
use crate::routing::{
    BridgeBulkWorkloadPlan, BridgeExecutionCounts, BridgeParallelAdmissionClass,
    BridgePlannedRoute, BridgeRouteSourceSummary,
};
use crate::snapshot::validate_snapshot_read_result_contract;

use super::context::{delivery_context, reject_delivery};
use super::requests::{BridgePreparedDeliveryRequest, BridgeSignalEvaluationRequest};
use super::snapshot::{first_snapshot_read_coordinate, open_planned_snapshot};

pub(crate) fn deliver_planned_route(
    runtime: &RuntimeBridge,
    route: BridgePlannedRoute,
    request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<BridgeRouteResult, BridgeDeliveryError> {
    let prepared = prepare_planned_route_for_delivery(route);
    deliver_prepared_route(runtime, prepared, request)
}

pub(crate) fn deliver_bulk_workload_plan(
    runtime: &RuntimeBridge,
    plan: BridgeBulkWorkloadPlan,
    request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<BridgeBulkWorkloadResult, BridgeDeliveryError> {
    if plan.planned_routes().is_empty() {
        return Err(BridgeDeliveryError::new(
            BridgeDeliveryErrorKind::BulkDeliveryRejected,
            "Bridge bulk delivery requires at least one planned route.",
        ));
    }

    let execution_plan = plan.execution_plan();
    validate_bulk_delivery_mode(execution_plan, execution_plan.selected_mode())?;

    let route_results = request
        .in_scope(|_| {
            plan.planned_routes()
                .iter()
                .cloned()
                .map(|route| {
                    deliver_prepared_route(
                        runtime,
                        prepare_planned_route_for_delivery(route),
                        request,
                    )
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .map_err(|denial| {
            BridgeDeliveryError::new(
                BridgeDeliveryErrorKind::ExecutionDenied(denial.into()),
                "Bridge execution scope refused bulk delivery",
            )
        })??;
    let delivered_target_count = route_results
        .iter()
        .map(|result| result.receipt().delivered_target_count())
        .sum();
    let summary = BridgeBulkResultSummary::new(
        plan.workload_identity().clone(),
        plan.canonical_planning_identity().clone(),
        std::sync::Arc::from(execution_plan.digest().to_owned()),
        std::sync::Arc::from(execution_plan.reduced_artifact().digest().to_owned()),
        execution_plan.selected_mode(),
        execution_plan.counters().clone(),
        route_results.len(),
        delivered_target_count,
    );

    Ok(BridgeBulkWorkloadResult::new(summary, route_results))
}

pub(crate) fn validate_bulk_delivery_mode(
    execution_plan: &crate::routing::AdmittedBridgeExecutionPlan,
    selected_mode: crate::routing::BridgePreparationMode,
) -> Result<(), BridgeDeliveryError> {
    if matches!(
        execution_plan.parallel_admission().class(),
        BridgeParallelAdmissionClass::ParallelPreparationRejected
    ) && !matches!(selected_mode, crate::routing::BridgePreparationMode::Serial)
    {
        return Err(BridgeDeliveryError::new(
            BridgeDeliveryErrorKind::BulkDeliveryRejected,
            "Bridge bulk delivery refused a plan whose selected mode did not honor the rejected parallel-admission class.",
        ));
    }
    Ok(())
}

pub(crate) fn prepare_planned_route_for_delivery(
    route: BridgePlannedRoute,
) -> BridgePreparedDeliveryRequest {
    BridgePreparedDeliveryRequest::new(route.into_prepared_delivery())
}

pub(crate) fn deliver_prepared_route(
    runtime: &RuntimeBridge,
    prepared: BridgePreparedDeliveryRequest,
    request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<BridgeRouteResult, BridgeDeliveryError> {
    request
        .in_scope(|_| deliver_prepared_route_in_scope(runtime, prepared, request))
        .map_err(|denial| {
            BridgeDeliveryError::new(
                BridgeDeliveryErrorKind::ExecutionDenied(denial.into()),
                "Bridge execution scope refused delivery",
            )
        })?
}

fn deliver_prepared_route_in_scope(
    runtime: &RuntimeBridge,
    prepared: BridgePreparedDeliveryRequest,
    request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<BridgeRouteResult, BridgeDeliveryError> {
    let failure_base = prepared.failure_source();
    let route_identity = prepared.routing_summary().route_identity().clone();
    let lowering_plan = prepared.validated_lowering_plan().plan();
    let mut counters = *prepared.counters();
    let read_packet = prepared.read_packet();
    let snapshot_reader = match super::snapshot::open_snapshot_reader(
        runtime,
        lowering_plan.source_snapshot(),
        request,
    ) {
        Ok(snapshot_reader) => snapshot_reader,
        Err(error) => {
            let failure = BridgeDeliveryError::new(
                BridgeDeliveryErrorKind::SnapshotAcquisitionFailure,
                format!(
                    "Bridge failed to open snapshot `{}`: {error}",
                    lowering_plan.source_snapshot().as_str()
                ),
            )
            .with_context(delivery_context(
                route_identity.clone(),
                lowering_plan.source_snapshot().clone(),
            ));
            return Err(reject_delivery(runtime, failure_base.clone(), failure));
        }
    };
    let snapshot = match crate::snapshot::AdmittedSnapshotContext::admit_for(
        crate::snapshot::BridgeSnapshotContext::bind(snapshot_reader),
        lowering_plan.source_snapshot(),
    ) {
        Ok(snapshot) => snapshot,
        Err(bound_snapshot_identity) => {
            let counters = counters.with_snapshot_identity_mismatch();
            let error = BridgeDeliveryError::new(
                BridgeDeliveryErrorKind::SnapshotIdentityMismatch,
                format!(
                    "Snapshot reader bound `{}` but planned route required `{}`.",
                    bound_snapshot_identity.as_str(),
                    lowering_plan.source_snapshot().as_str()
                ),
            )
            .with_context(delivery_context(
                route_identity.clone(),
                lowering_plan.source_snapshot().clone(),
            ));
            return Err(reject_delivery(
                runtime,
                failure_base.clone().with_counters(counters),
                error,
            ));
        }
    };
    let read_result = snapshot
        .read_packet(read_packet, request)
        .map_err(|error| {
            let kind = error.delivery_kind(BridgeDeliveryErrorKind::SnapshotReadFailure);
            BridgeDeliveryError::new(
                kind,
                format!(
                    "Bridge failed to execute packetized snapshot reads for `{}`: {error}",
                    lowering_plan.source_snapshot().as_str()
                ),
            )
            .with_context(delivery_context(
                route_identity.clone(),
                lowering_plan.source_snapshot().clone(),
            ))
        })?;
    if read_result.snapshot_identity() != lowering_plan.source_snapshot() {
        let counters = counters.with_snapshot_identity_mismatch();
        let error = BridgeDeliveryError::new(
            BridgeDeliveryErrorKind::SnapshotIdentityMismatch,
            format!(
                "Snapshot read returned `{}` but planned route required `{}`.",
                read_result.snapshot_identity().as_str(),
                lowering_plan.source_snapshot().as_str()
            ),
        )
        .with_context(delivery_context(
            route_identity.clone(),
            lowering_plan.source_snapshot().clone(),
        ));
        return Err(reject_delivery(
            runtime,
            failure_base.clone().with_counters(counters),
            error,
        ));
    }
    counters = counters.with_validation_record_count(read_result.records().len());
    let validated_reads = match validate_snapshot_read_result_contract(read_packet, read_result) {
        Ok(validated_reads) => validated_reads,
        Err(error) => {
            let failure = BridgeDeliveryError::new(
                error.delivery_kind(BridgeDeliveryErrorKind::SnapshotReadContractViolation),
                format!(
                    "Bridge snapshot read contract failed for `{}`: {error}",
                    lowering_plan.source_snapshot().as_str()
                ),
            )
            .with_context(
                delivery_context(
                    route_identity.clone(),
                    lowering_plan.source_snapshot().clone(),
                )
                .with_snapshot_read(first_snapshot_read_coordinate(read_packet)),
            );
            return Err(reject_delivery(runtime, failure_base.clone(), failure));
        }
    };

    let lowered = prepared.into_inner().into_lowered_execution(counters);
    let lowered_failure_base = lowered.failure_source();
    let delivery = crate::routing::BridgeSignalInvalidationDelivery::new(
        lowered.artifact(),
        lowered.contract_proof(),
    );
    let receipt = runtime
        .signal_sink
        .deliver_invalidation(delivery, request)
        .map_err(|error| {
            reject_delivery(
                runtime,
                lowered_failure_base.clone(),
                BridgeDeliveryError::new(
                    error.delivery_kind(),
                    format!("Signal bridge sink rejected invalidation delivery: {error}"),
                )
                .with_context(
                    delivery_context(
                        lowered.artifact().route_identity().clone(),
                        lowered.artifact().source_snapshot().clone(),
                    )
                    .with_invalidation_identity(lowered.artifact().invalidation_identity().clone())
                    .with_subscription_slice_identity(
                        lowered.artifact().subscription_slice_identity().clone(),
                    ),
                ),
            )
        })?;
    let result_summary = BridgeRouteResultSummary::new(
        BridgeRouteOutcomeReference::new(
            lowered.artifact().route_identity().clone(),
            lowered.artifact().invalidation_identity().clone(),
            BridgeRouteSourceSummary::new(
                lowered.artifact().source_branch().clone(),
                lowered.artifact().source_commit().clone(),
                lowered.artifact().source_patch().clone(),
                lowered.artifact().source_snapshot().clone(),
            ),
            lowered.artifact().subscription_slice_identity().clone(),
        ),
        lowered.contract_proof().clone(),
        lowered.routing_summary().routing_entry_count(),
        BridgeExecutionCounts::new(
            lowered.artifact().invalidation_targets().targets().len(),
            lowered.artifact().subscription_slices().len(),
            validated_reads.records().len(),
        ),
        receipt.delivered_target_count(),
    );
    let retain_route_record = lowered
        .route_scope()
        .route_planning_policy()
        .map(|policy| policy.route_artifacts())
        .unwrap_or_else(|| runtime.policy().record_route_artifacts());
    if retain_route_record {
        runtime.diagnostic_sink.record_route(BridgeRouteRecord::new(
            lowered.artifact().route_identity().clone(),
            lowered.artifact().invalidation_identity().clone(),
            lowered.artifact().source_branch().clone(),
            lowered.artifact().source_commit().clone(),
            lowered.artifact().source_patch().clone(),
            lowered.artifact().source_snapshot().clone(),
            lowered.contract_proof().clone(),
            lowered.artifact().subscription_slice_identity().clone(),
            std::sync::Arc::clone(lowered.route_record_entries()),
            std::sync::Arc::clone(lowered.artifact().subscription_slices().shared()),
            std::sync::Arc::clone(lowered.artifact().invalidation_targets().shared()),
            *lowered.counters(),
        ));
    }

    Ok(BridgeRouteResult::new(
        lowered.routing_summary().clone(),
        result_summary,
        *lowered.artifact().counters(),
        lowered.artifact().clone(),
        receipt,
    ))
}

pub(crate) fn prepare_signal_evaluation(
    runtime: &RuntimeBridge,
    route: BridgePlannedRoute,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<BridgeSignalEvaluationRequest, BridgeDeliveryError> {
    let prepared = prepare_planned_route_for_delivery(route);
    let counters = *prepared.counters();
    let read_packet = prepared.read_packet().clone();
    let snapshot = open_planned_snapshot(
        runtime,
        prepared.validated_lowering_plan().plan().source_snapshot(),
        execution,
    )?;
    let artifact = prepared
        .into_inner()
        .into_lowered_execution(counters)
        .artifact()
        .clone();

    Ok(BridgeSignalEvaluationRequest::new(
        artifact,
        read_packet,
        snapshot,
    ))
}
