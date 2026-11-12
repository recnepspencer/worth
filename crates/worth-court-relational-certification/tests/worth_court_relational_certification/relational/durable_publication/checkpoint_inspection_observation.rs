use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::RecordRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct InspectionTruthBundle {
    pub graph_summary: worth_relational::facade::inspection::GraphInspectionSummary,
    pub kind_summary: worth_relational::facade::inspection::KindInspectionSummary,
    pub connectivity_summary: worth_relational::facade::inspection::ConnectivityInspectionSummary,
    pub historical_record: worth_relational::facade::inspection::HistoricalRecordInspection,
    pub branch_head: Option<worth_relational::facade::inspection::CommitInspection>,
    pub latest_commit: worth_relational::facade::inspection::CommitInspection,
    pub recent_commits: worth_relational::facade::inspection::RecentCommitInspectionWindow,
}

pub(super) fn capture_inspection_truth_bundle(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch_id: &BranchId,
    entity_id: worth_relational::facade::identity::EntityId,
    historical_version: worth_relational::facade::identity::VersionId,
) -> InspectionTruthBundle {
    let inspection = runtime.inspect_what_happened();
    let latest_commit_id = runtime
        .history()
        .historical_latest_commit()
        .expect("the original latest-commit selection remains available")
        .commit_id;
    InspectionTruthBundle {
        graph_summary: inspection
            .graph_summary(
                execution,
                &worth_relational::facade::inspection::GraphInspectionRequest {
                    scope: worth_relational::facade::inspection::InspectionScope::Current,
                    partition_scope: None,
                    relation_kind_scope: None,
                    summary_only: true,
                    budget: default_graph_budget(),
                },
            )
            .expect("read request remains live"),
        kind_summary: inspection
            .kind_summary(
                execution,
                &worth_relational::facade::inspection::KindInspectionRequest {
                    scope: worth_relational::facade::inspection::InspectionScope::Current,
                    partition_scope: None,
                    kind_id: KindId(1),
                    record_class: worth_relational::facade::inspection::InspectionRecordClass::Entity,
                },
            )
            .expect("read request remains live"),
        connectivity_summary: inspection
            .connectivity_summary(
                execution,
                &worth_relational::facade::inspection::ConnectivityInspectionRequest {
                    scope: worth_relational::facade::inspection::InspectionScope::Current,
                    partition_scope: None,
                    relation_kind_scope: None,
                    include_members: false,
                    budget: default_connectivity_budget(),
                },
            )
            .expect("read request remains live"),
        historical_record: inspection
            .inspect_historical_record(
                execution,
                branch_id,
                historical_version,
                RecordRef::Entity(entity_id),
                worth_relational::facade::inspection::HistoricalInspectionMode::AllowCanonicalReconstruction,
            )
            .expect("read request remains live"),
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
                &worth_relational::facade::inspection::RecentCommitInspectionRequest {
                    branch_id: Some(branch_id.clone()),
                    limit: 8,
                },
            )
            .unwrap(),
    }
}
fn default_graph_budget() -> worth_relational::facade::inspection::GraphInspectionBudget {
    worth_relational::facade::inspection::GraphInspectionBudget {
        max_entities: 1_024,
        max_relations: 1_024,
        max_work_units: 4_096,
    }
}

fn default_connectivity_budget(
) -> worth_relational::facade::inspection::ConnectivityInspectionBudget {
    worth_relational::facade::inspection::ConnectivityInspectionBudget {
        max_entities: 1_024,
        max_relations: 1_024,
        max_frontier: 1_024,
        max_components: 1_024,
        max_work_units: 8_192,
    }
}
