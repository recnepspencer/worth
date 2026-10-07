use crate::basis::{
    preflight_execution_basis, resolve_snapshot_basis, BasisAuthorityFamily, BasisPreflightError,
    BasisResolutionError, BasisResolutionMode, ExecutionBasisIntent, SnapshotLineageClass,
};
use crate::execution::{execute_preflight_bundle, ExecutionError};
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::projection_consumption::ProjectionMaterializedFactPosture;
use crate::runtime::{
    WorthQueryCountResult, WorthQueryReadDenial, WorthQueryReadDenialKind, WorthQueryReadGraph,
    WorthQueryReadResult, WorthQueryRuntime,
};

use super::graph_read_access::WorthQueryGraphReadAccessExecutionRecorder;
use super::read_composition_materialization::materialize_read_rows;
use super::read_composition_runtime::{
    materialized_fact_posture_for_read_graph, WorthQueryExecutedCountGraph,
    WorthQueryExecutedReadGraph, WorthQueryExecutedReadProduct,
};

pub(in crate::runtime) fn execute_runtime_current_read_graph(
    runtime: &mut WorthQueryRuntime,
    read_graph: &WorthQueryReadGraph,
) -> Result<WorthQueryExecutedReadGraph, WorthQueryReadDenial> {
    let materialized = materialize_current_read_input(runtime, read_graph)?;
    let receipt = crate::runtime::WorthQueryReadReceipt::from_materialized_rows(
        read_graph,
        materialized.snapshot_identity,
        &materialized.execution,
        &materialized.rows,
        materialized.records_examined_count,
    )
    .with_materialized_fact_posture(materialized.fact_posture);
    Ok(WorthQueryExecutedReadProduct {
        graph_read_access_execution_counters: materialized.graph_read_access_counters,
        product: WorthQueryReadResult::new_with_source_rows(
            materialized.rows,
            materialized.maintenance_source_rows,
            receipt,
        ),
    })
}

pub(in crate::runtime) fn execute_runtime_current_count_graph(
    runtime: &mut WorthQueryRuntime,
    read_graph: &WorthQueryReadGraph,
) -> Result<WorthQueryExecutedCountGraph, WorthQueryReadDenial> {
    ensure_count_aggregate_plan(read_graph)?;
    let materialized = materialize_current_read_input(runtime, read_graph)?;
    let count = u64::try_from(materialized.rows.len()).map_err(|_| {
        WorthQueryReadDenial::new(
            WorthQueryReadDenialKind::ExecutionDenied,
            "materialized collection cardinality exceeds the count result domain",
        )
    })?;
    let receipt = crate::runtime::WorthQueryReadReceipt::from_materialized_count(
        read_graph,
        materialized.snapshot_identity,
        &materialized.execution,
        materialized.records_examined_count,
        materialized.rows.len(),
        count,
    )
    .with_materialized_fact_posture(materialized.fact_posture);
    Ok(WorthQueryExecutedReadProduct {
        graph_read_access_execution_counters: materialized.graph_read_access_counters,
        product: WorthQueryCountResult::new(count, receipt),
    })
}

struct WorthQueryCurrentReadMaterialization {
    snapshot_identity: WorthQuerySnapshotIdentity,
    execution: crate::execution::ExecutionResultEnvelope,
    rows: Vec<crate::memory_workspace::WorthQueryEntity>,
    maintenance_source_rows: Vec<crate::memory_workspace::WorthQueryEntity>,
    records_examined_count: usize,
    graph_read_access_counters: crate::runtime::WorthQueryGraphReadAccessExecutionCounters,
    fact_posture: Option<ProjectionMaterializedFactPosture>,
}

fn materialize_current_read_input(
    runtime: &mut WorthQueryRuntime,
    read_graph: &WorthQueryReadGraph,
) -> Result<WorthQueryCurrentReadMaterialization, WorthQueryReadDenial> {
    let admitted = admit_current_read_execution(runtime, read_graph)?;
    let snapshot_evidence_identity = admitted.snapshot_identity.evidence_identity();
    let mut graph_read_access_recorder =
        WorthQueryGraphReadAccessExecutionRecorder::entered_executor();
    let (rows, maintenance_source_rows, records_examined_count) =
        materialize_read_rows(runtime, read_graph)?.into_parts();
    graph_read_access_recorder.record_materialized_rows(rows.len());
    let fact_posture =
        materialized_fact_posture_for_read_graph(runtime, read_graph, &snapshot_evidence_identity);
    Ok(WorthQueryCurrentReadMaterialization {
        snapshot_identity: admitted.snapshot_identity,
        execution: admitted.execution,
        rows,
        maintenance_source_rows,
        records_examined_count,
        graph_read_access_counters: graph_read_access_recorder.finish(),
        fact_posture,
    })
}

struct WorthQueryCurrentReadExecutionAdmission {
    snapshot_identity: WorthQuerySnapshotIdentity,
    execution: crate::execution::ExecutionResultEnvelope,
}

fn admit_current_read_execution(
    runtime: &WorthQueryRuntime,
    read_graph: &WorthQueryReadGraph,
) -> Result<WorthQueryCurrentReadExecutionAdmission, WorthQueryReadDenial> {
    let snapshot_identity = runtime.current_snapshot_identity()?;
    let identity = crate::basis::ResolvedSnapshotIdentity::new(
        BasisAuthorityFamily::Runtime,
        None,
        snapshot_identity.evidence_identity(),
        read_graph.schema_basis().clone(),
        SnapshotLineageClass::CurrentHead,
    );
    let basis = resolve_snapshot_basis(
        runtime_basis_intent(),
        identity,
        BasisResolutionMode::RuntimeDirect,
    )
    .map_err(|error| {
        WorthQueryReadDenial::new(
            WorthQueryReadDenialKind::BasisResolutionDenied,
            basis_resolution_error_message(error),
        )
    })?;
    let preflight =
        preflight_execution_basis(read_graph.execution_plan().clone(), basis).map_err(|error| {
            WorthQueryReadDenial::new(
                WorthQueryReadDenialKind::BasisPreflightDenied,
                basis_preflight_error_message(error),
            )
        })?;
    let execution = execute_preflight_bundle(&preflight).map_err(|error| {
        WorthQueryReadDenial::new(
            WorthQueryReadDenialKind::ExecutionDenied,
            execution_error_message(error),
        )
    })?;
    Ok(WorthQueryCurrentReadExecutionAdmission {
        snapshot_identity,
        execution,
    })
}

fn ensure_count_aggregate_plan(
    read_graph: &WorthQueryReadGraph,
) -> Result<(), WorthQueryReadDenial> {
    let is_count = read_graph
        .execution_plan()
        .collection()
        .is_some_and(|collection| {
            matches!(
                collection.planning_context().result_family(),
                crate::collection::CollectionResultFamily::CountAggregate
            ) && matches!(
                collection
                    .post_read_shaping()
                    .aggregate_shape()
                    .function_family(),
                crate::collection::AggregateFunctionFamily::CountRows
            )
        });
    if is_count {
        Ok(())
    } else {
        Err(WorthQueryReadDenial::new(
            WorthQueryReadDenialKind::ExecutionDenied,
            "count execution requires an admitted count aggregate plan",
        ))
    }
}

fn runtime_basis_intent() -> ExecutionBasisIntent {
    ExecutionBasisIntent::new(
        BasisAuthorityFamily::Runtime,
        SnapshotLineageClass::CurrentHead,
        false,
    )
}

fn basis_resolution_error_message(error: BasisResolutionError) -> String {
    match error {
        BasisResolutionError::UnsupportedBasisKind => "unsupported basis kind".to_string(),
        BasisResolutionError::ResolutionIdentityMismatch => {
            "resolution identity mismatch".to_string()
        }
    }
}

fn basis_preflight_error_message(error: BasisPreflightError) -> String {
    match error {
        BasisPreflightError::BasisIntentMismatch => "basis intent mismatch".to_string(),
        BasisPreflightError::PlannedRouteBasisMismatch => {
            "planned route basis mismatch".to_string()
        }
    }
}

fn execution_error_message(error: ExecutionError) -> String {
    match error {
        ExecutionError::ExecutionInvariantViolation { message } => message.to_string(),
    }
}
