use super::*;

use crate::facade::diagnostics::RelationalDiagnosticsProfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PerfDiagnosticsPolicy {
    GeometryOperationalHotPath,
    GeometryRichCertification,
    ChipOperationalHotPath,
    ChipRichCertification,
}

pub(crate) fn apply_perf_diagnostics_policy(
    runtime: &mut RelationalRuntime,
    policy: PerfDiagnosticsPolicy,
) {
    let profile = match policy {
        PerfDiagnosticsPolicy::GeometryOperationalHotPath => {
            RelationalDiagnosticsProfile::geometry_operational_hot_path()
        }
        PerfDiagnosticsPolicy::GeometryRichCertification => {
            RelationalDiagnosticsProfile::geometry_rich_certification()
        }
        PerfDiagnosticsPolicy::ChipOperationalHotPath => {
            RelationalDiagnosticsProfile::chip_operational_hot_path()
        }
        PerfDiagnosticsPolicy::ChipRichCertification => {
            RelationalDiagnosticsProfile::chip_rich_certification()
        }
    };
    runtime.configure_diagnostics_for_test(|configured| *configured = profile);
}

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
    snapshot_graph_request(
        crate::facade::inspection::InspectionScope::Current,
        partition_scope,
        relation_kind_scope,
        summary_only,
    )
}

pub(crate) fn version_graph_request(
    version_id: crate::facade::identity::VersionId,
    partition_scope: Option<Vec<PartitionId>>,
    relation_kind_scope: Option<Vec<KindId>>,
    summary_only: bool,
) -> crate::facade::inspection::GraphInspectionRequest {
    snapshot_graph_request(
        crate::facade::inspection::InspectionScope::Version(version_id),
        partition_scope,
        relation_kind_scope,
        summary_only,
    )
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch_id: &BranchId,
    limit: usize,
) -> crate::facade::inspection::RecentCommitInspectionWindow {
    runtime
        .inspect_what_happened()
        .inspect_recent_commits(
            execution,
            &crate::facade::inspection::RecentCommitInspectionRequest {
                branch_id: Some(branch_id.clone()),
                limit: limit as u64,
            },
        )
        .unwrap()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InspectionTruthBundle {
    pub graph_summary: crate::facade::inspection::GraphInspectionSummary,
    pub kind_summary: crate::facade::inspection::KindInspectionSummary,
    pub connectivity_summary: crate::facade::inspection::ConnectivityInspectionSummary,
    pub historical_record: crate::facade::inspection::HistoricalRecordInspection,
    pub retention_summary: crate::facade::inspection::RetentionInspectionSummary,
    pub record_retention: crate::facade::inspection::RecordRetentionInspection,
    pub branch_head: Option<crate::facade::inspection::CommitInspection>,
    pub latest_commit: crate::facade::inspection::CommitInspection,
    pub recent_commits: crate::facade::inspection::RecentCommitInspectionWindow,
}

pub(crate) fn capture_inspection_truth_bundle(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch_id: &BranchId,
    entity_id: crate::facade::identity::EntityId,
    historical_version: crate::facade::identity::VersionId,
) -> InspectionTruthBundle {
    let inspection = runtime.inspect_what_happened();
    let latest_commit_id = runtime
        .history()
        .latest_commit()
        .map(|commit| commit.commit_id)
        .expect("latest commit");
    InspectionTruthBundle {
        graph_summary: inspection
            .graph_summary(
                execution,
                &current_graph_request(None, None, true),
            )
            .expect("read request remains live"),
        kind_summary: inspection
            .kind_summary(
                execution,
                &crate::facade::inspection::KindInspectionRequest {
                    scope: crate::facade::inspection::InspectionScope::Current,
                    partition_scope: None,
                    kind_id: KindId(1),
                    record_class: crate::facade::inspection::InspectionRecordClass::Entity,
                },
            )
            .expect("read request remains live"),
        connectivity_summary: inspection
            .connectivity_summary(
                execution,
                &connectivity_request(
                    crate::facade::inspection::InspectionScope::Current, None, None, false,
                ),
            )
            .expect("read request remains live"),
        historical_record: inspection
            .inspect_historical_record(
                execution,
                branch_id,
                historical_version,
                RecordRef::Entity(entity_id),
                crate::facade::inspection::HistoricalInspectionMode::AllowCanonicalReconstruction,
            )
            .expect("read request remains live"),
        retention_summary: inspection
            .retention_summary(
                execution,
                &crate::tests::support::inspection::default_retention_request(),
            )
            .unwrap(),
        record_retention: inspection
            .inspect_record_retention(execution, RecordRef::Entity(entity_id))
            .unwrap()
            .expect("record retention"),
        branch_head: inspection
            .inspect_branch_head(execution, branch_id)
            .unwrap(),
        latest_commit: inspection
            .inspect_commit(execution, latest_commit_id)
            .unwrap()
            .expect("latest commit inspection"),
        recent_commits: inspection
            .inspect_recent_commits(
                execution,
                &crate::facade::inspection::RecentCommitInspectionRequest {
                    branch_id: Some(branch_id.clone()),
                    limit: 8,
                },
            )
            .unwrap(),
    }
}
