use crate::memory_workspace::{WorthQueryEntity, WorthQuerySnapshotIdentity};
use crate::planning::{FallbackDisposition, PlannedExecutionRoute};
use crate::query_context::{QueryContextExecutionArtifact, QueryContextExecutionFamily};
use crate::runtime::read_composition_relationship_proof::support_profile_for_relationship_proof;

use super::read_receipt_support::{materialized_count_result_digest, materialized_result_digest};
use super::{
    WorthQueryReadBreadth, WorthQueryReadExecutionEngine, WorthQueryReadFallbackClass,
    WorthQueryReadGraph, WorthQueryReadReceipt, WorthQueryReadRelationshipProofPosture,
};

impl WorthQueryReadReceipt {
    pub(in crate::runtime) fn from_materialized_count(
        read_graph: &WorthQueryReadGraph,
        snapshot_identity: WorthQuerySnapshotIdentity,
        execution: &crate::execution::ExecutionResultEnvelope,
        records_examined_count: usize,
        input_row_count: usize,
        count: u64,
    ) -> Self {
        let execution_counters = execution
            .counters()
            .clone()
            .with_count_aggregate_input(records_examined_count, input_row_count);
        let breadth = WorthQueryReadBreadth {
            planned_read_surface_count: read_graph
                .execution_plan()
                .counters()
                .planned_read_surface_count(),
            planned_traversal_clause_count: read_graph
                .execution_plan()
                .counters()
                .planned_traversal_clause_count()
                .max(read_graph.declared_traversal_clause_count()),
            planned_traversal_depth_limit: read_graph
                .execution_plan()
                .counters()
                .planned_traversal_depth_limit()
                .max(read_graph.declared_traversal_depth_limit()),
            execution_query_projection_count: read_graph
                .declarative_request()
                .query_projection()
                .len(),
            execution_read_operation_count: execution_counters.execution_read_operation_count(),
            execution_records_examined_count: records_examined_count,
            execution_records_emitted_count: 1,
            execution_page_width: 1,
            execution_page_truncation_count: 0,
            execution_cursor_advance_count: execution_counters.cursor_advance_count(),
            execution_materialized_relation_count: execution_counters.materialized_relation_count(),
            execution_aggregate_input_count: input_row_count,
            execution_rollup_input_count: input_row_count,
        };
        let result_digest = materialized_count_result_digest(
            execution.report().query_digest().as_str(),
            execution.report().basis_digest().as_str(),
            input_row_count,
            count,
        )
        .as_str()
        .to_string();
        Self::from_parts(
            read_graph,
            execution.report().basis_digest().as_str(),
            snapshot_identity,
            execution_engine_for_planned_route(read_graph),
            fallback_class_for_planned_route(read_graph),
            execution_counters.execution_fallback_taken_count(),
            breadth,
            result_digest,
        )
    }

    pub(in crate::runtime) fn from_materialized_rows(
        read_graph: &WorthQueryReadGraph,
        snapshot_identity: WorthQuerySnapshotIdentity,
        execution: &crate::execution::ExecutionResultEnvelope,
        rows: &[WorthQueryEntity],
        records_examined_count: usize,
    ) -> Self {
        let execution_counters = execution
            .counters()
            .clone()
            .with_materialized_rows(records_examined_count, rows.len());
        let result_digest = materialized_result_digest(
            execution.report().query_digest().as_str(),
            execution.report().basis_digest().as_str(),
            rows,
        )
        .as_str()
        .to_string();
        Self::from_parts(
            read_graph,
            execution.report().basis_digest().as_str(),
            snapshot_identity,
            execution_engine_for_planned_route(read_graph),
            fallback_class_for_planned_route(read_graph),
            execution_counters.execution_fallback_taken_count(),
            WorthQueryReadBreadth {
                planned_read_surface_count: read_graph
                    .execution_plan()
                    .counters()
                    .planned_read_surface_count(),
                planned_traversal_clause_count: read_graph
                    .execution_plan()
                    .counters()
                    .planned_traversal_clause_count()
                    .max(read_graph.declared_traversal_clause_count()),
                planned_traversal_depth_limit: read_graph
                    .execution_plan()
                    .counters()
                    .planned_traversal_depth_limit()
                    .max(read_graph.declared_traversal_depth_limit()),
                execution_query_projection_count: read_graph
                    .declarative_request()
                    .query_projection()
                    .len(),
                execution_read_operation_count: execution_counters.execution_read_operation_count(),
                execution_records_examined_count: execution_counters
                    .execution_records_examined_count(),
                execution_records_emitted_count: execution_counters
                    .execution_records_emitted_count(),
                execution_page_width: execution_counters.page_width(),
                execution_page_truncation_count: execution_counters.page_truncation_count(),
                execution_cursor_advance_count: execution_counters.cursor_advance_count(),
                execution_materialized_relation_count: execution_counters
                    .materialized_relation_count(),
                execution_aggregate_input_count: execution_counters.aggregate_input_count(),
                execution_rollup_input_count: execution_counters.rollup_input_count(),
            },
            result_digest,
        )
    }

    pub(in crate::runtime) fn from_query_context_execution(
        read_graph: &WorthQueryReadGraph,
        snapshot_identity: WorthQuerySnapshotIdentity,
        context_execution: &QueryContextExecutionArtifact,
        rows: &[WorthQueryEntity],
    ) -> Self {
        let result_digest = materialized_result_digest(
            context_execution.query_digest(),
            context_execution.basis_digest(),
            rows,
        )
        .as_str()
        .to_string();
        let mut receipt = Self::from_parts(
            read_graph,
            context_execution.basis_digest(),
            snapshot_identity,
            execution_engine_for_query_context(context_execution.family()),
            fallback_class_for_planned_route(read_graph),
            0,
            WorthQueryReadBreadth {
                planned_read_surface_count: read_graph
                    .execution_plan()
                    .counters()
                    .planned_read_surface_count(),
                planned_traversal_clause_count: read_graph
                    .execution_plan()
                    .counters()
                    .planned_traversal_clause_count()
                    .max(read_graph.declared_traversal_clause_count()),
                planned_traversal_depth_limit: read_graph
                    .execution_plan()
                    .counters()
                    .planned_traversal_depth_limit()
                    .max(read_graph.declared_traversal_depth_limit()),
                execution_query_projection_count: read_graph
                    .declarative_request()
                    .query_projection()
                    .len(),
                execution_read_operation_count: context_execution
                    .counters()
                    .context_execution_count(),
                execution_records_examined_count: context_execution
                    .counters()
                    .materialized_row_count(),
                execution_records_emitted_count: rows.len(),
                execution_page_width: context_execution.counters().materialized_row_count(),
                execution_page_truncation_count: 0,
                execution_cursor_advance_count: 0,
                execution_materialized_relation_count: rows.len(),
                execution_aggregate_input_count: 0,
                execution_rollup_input_count: 0,
            },
            result_digest,
        );
        receipt.materialized_fact_posture = context_execution.materialized_fact_posture().cloned();
        receipt
    }

    fn from_parts(
        read_graph: &WorthQueryReadGraph,
        basis_digest: &str,
        snapshot_identity: WorthQuerySnapshotIdentity,
        execution_engine: WorthQueryReadExecutionEngine,
        fallback_class: WorthQueryReadFallbackClass,
        fallback_count: usize,
        breadth: WorthQueryReadBreadth,
        result_digest: String,
    ) -> Self {
        let relationship_proof_admission = read_graph.relationship_proof_admission().cloned();
        let relationship_proof_support_profile = relationship_proof_admission
            .as_ref()
            .map(support_profile_for_relationship_proof);
        let policy_aware_plan = read_graph.policy_aware_plan();
        Self {
            read_graph_identity: read_graph.evidence_identity().clone(),
            graph_family: read_graph.family().clone(),
            collection_result_family: read_graph
                .execution_plan()
                .collection()
                .map(|collection| collection.planning_context().result_family().clone()),
            execution_plan_evidence_identity: read_graph
                .execution_plan()
                .query()
                .plan_digest()
                .evidence_identity(),
            canonical_query_digest: read_graph.canonical().query().digest().as_str().to_string(),
            query_digest: read_graph.query_digest().to_string(),
            basis_digest: basis_digest.to_string(),
            result_digest,
            snapshot_identity,
            scope_class: read_graph.scope_class().clone(),
            execution_engine,
            fallback_class,
            fallback_count,
            operator_families: read_graph.operator_families(),
            built_in_operator_coverage: read_graph.built_in_operators().to_vec(),
            relationship_proof_posture: if relationship_proof_admission.is_some() {
                WorthQueryReadRelationshipProofPosture::DescriptorAdmittedSyntheticRuntime
            } else {
                WorthQueryReadRelationshipProofPosture::NotRequired
            },
            relationship_proof_admission,
            relationship_proof_support_profile,
            policy_narrowing_digest: policy_aware_plan.map(|plan| {
                plan.core()
                    .seam()
                    .source_narrowed_artifact_digest()
                    .to_string()
            }),
            policy_aware_plan_digest: policy_aware_plan
                .map(|plan| plan.core().digest().as_str().to_string()),
            policy_execution_seam_identity: policy_aware_plan
                .map(|plan| plan.core().seam().identity().as_str().to_string()),
            policy_executor_semantic_rediscovery_count: policy_aware_plan
                .map(|plan| plan.core().report().executor_semantic_rediscovery_count())
                .unwrap_or(0),
            breadth,
            materialized_fact_posture: None,
            graph_read_access_plan: None,
            graph_read_access_plan_consumption: None,
            graph_read_access_summary: None,
            graph_read_access_complexity_counters: None,
            decision_trace_envelope: None,
            execution_provenance: None,
        }
    }
}

fn fallback_class_for_planned_route(
    read_graph: &WorthQueryReadGraph,
) -> WorthQueryReadFallbackClass {
    match read_graph.execution_plan().query().fallback() {
        FallbackDisposition::Forbidden | FallbackDisposition::AdmittedButUnused => {
            WorthQueryReadFallbackClass::None
        }
        FallbackDisposition::AdmittedAndSelected => {
            WorthQueryReadFallbackClass::SnapshotIndexedDebt
        }
    }
}

fn execution_engine_for_planned_route(
    read_graph: &WorthQueryReadGraph,
) -> WorthQueryReadExecutionEngine {
    match read_graph.execution_plan().query().route() {
        PlannedExecutionRoute::RuntimeSnapshotRead
        | PlannedExecutionRoute::RuntimeExpandedSnapshotRead
        | PlannedExecutionRoute::StoreSnapshotRead => {
            WorthQueryReadExecutionEngine::QueryRuntimeCurrent
        }
    }
}

fn execution_engine_for_query_context(
    family: &QueryContextExecutionFamily,
) -> WorthQueryReadExecutionEngine {
    match family {
        QueryContextExecutionFamily::RuntimeCurrent => {
            WorthQueryReadExecutionEngine::QueryRuntimeCurrent
        }
        QueryContextExecutionFamily::RuntimeBranch => {
            WorthQueryReadExecutionEngine::QueryRuntimeBranch
        }
        QueryContextExecutionFamily::HistoricalMaterialized => {
            WorthQueryReadExecutionEngine::QueryRuntimeHistorical
        }
        QueryContextExecutionFamily::PreviewDerivedHistorical => {
            WorthQueryReadExecutionEngine::QueryRuntimePreviewDerived
        }
    }
}
