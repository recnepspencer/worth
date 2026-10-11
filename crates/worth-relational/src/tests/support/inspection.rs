use super::*;

pub(crate) fn default_graph_budget() -> crate::facade::inspection::GraphInspectionBudget {
    crate::facade::inspection::GraphInspectionBudget {
        max_entities: 1_024,
        max_relations: 1_024,
        max_work_units: 4_096,
    }
}

pub(crate) fn default_connectivity_budget(
) -> crate::facade::inspection::ConnectivityInspectionBudget {
    crate::facade::inspection::ConnectivityInspectionBudget {
        max_entities: 1_024,
        max_relations: 1_024,
        max_frontier: 1_024,
        max_components: 1_024,
        max_work_units: 8_192,
    }
}

pub(crate) fn default_retention_request() -> crate::facade::inspection::RetentionInspectionRequest {
    crate::facade::inspection::RetentionInspectionRequest {
        max_entity_slots_scanned: 1_024,
        max_relation_slots_scanned: 1_024,
        max_work_units: 4_096,
    }
}

pub(crate) fn current_graph_request(
    partition_scope: Option<Vec<PartitionId>>,
    relation_kind_scope: Option<Vec<KindId>>,
    summary_only: bool,
) -> crate::facade::inspection::GraphInspectionRequest {
    crate::facade::inspection::GraphInspectionRequest {
        scope: crate::facade::inspection::InspectionScope::Current,
        partition_scope,
        relation_kind_scope,
        summary_only,
        budget: default_graph_budget(),
    }
}

pub(crate) fn version_graph_request(
    version_id: crate::facade::identity::VersionId,
    partition_scope: Option<Vec<PartitionId>>,
    relation_kind_scope: Option<Vec<KindId>>,
    summary_only: bool,
) -> crate::facade::inspection::GraphInspectionRequest {
    crate::facade::inspection::GraphInspectionRequest {
        scope: crate::facade::inspection::InspectionScope::Version(version_id),
        partition_scope,
        relation_kind_scope,
        summary_only,
        budget: default_graph_budget(),
    }
}

pub(crate) fn snapshot_graph_request(
    scope: crate::facade::inspection::InspectionScope,
    partition_scope: Option<Vec<PartitionId>>,
    relation_kind_scope: Option<Vec<KindId>>,
    summary_only: bool,
) -> crate::facade::inspection::GraphInspectionRequest {
    crate::facade::inspection::GraphInspectionRequest {
        scope,
        partition_scope,
        relation_kind_scope,
        summary_only,
        budget: default_graph_budget(),
    }
}

pub(crate) fn connectivity_request(
    scope: crate::facade::inspection::InspectionScope,
    partition_scope: Option<Vec<PartitionId>>,
    relation_kind_scope: Option<Vec<KindId>>,
    include_members: bool,
) -> crate::facade::inspection::ConnectivityInspectionRequest {
    crate::facade::inspection::ConnectivityInspectionRequest {
        scope,
        partition_scope,
        relation_kind_scope,
        include_members,
        budget: default_connectivity_budget(),
    }
}

pub(crate) fn retained_record_inspection(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch_id: &BranchId,
    version_id: crate::facade::identity::VersionId,
    target: RecordRef,
) -> crate::facade::inspection::HistoricalRecordInspection {
    runtime
        .inspect_what_happened()
        .inspect_historical_record(
            execution,
            branch_id,
            version_id,
            target,
            crate::facade::inspection::HistoricalInspectionMode::RetainedOnly,
        )
        .expect("read request remains live")
}

pub(crate) fn reconstructed_record_inspection(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch_id: &BranchId,
    version_id: crate::facade::identity::VersionId,
    target: RecordRef,
) -> crate::facade::inspection::HistoricalRecordInspection {
    runtime
        .inspect_what_happened()
        .inspect_historical_record(
            execution,
            branch_id,
            version_id,
            target,
            crate::facade::inspection::HistoricalInspectionMode::AllowCanonicalReconstruction,
        )
        .expect("read request remains live")
}

pub(crate) fn recent_commit_window(
    runtime: &RelationalRuntime,
    branch_id: &BranchId,
    limit: usize,
) -> crate::facade::inspection::RecentCommitInspectionWindow {
    runtime.inspect_what_happened().inspect_recent_commits(
        &crate::facade::inspection::RecentCommitInspectionRequest {
            branch_id: Some(branch_id.clone()),
            limit: limit as u64,
        },
    )
}
