use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
pub(super) use worth_harness::facade::{
    DiagnosticsHarnessAdapter, HarnessAdapter, ReplayHarnessAdapter,
};

pub(super) use crate::config::data::PublicationConfig;
pub(super) use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
pub(super) use crate::config::data::{DurableLogPolicy, DurableLogRetentionMode};
pub(super) use crate::facade::config::{
    RelationalRuntimeProfile, StorageLayoutConfig, VisibilityCachePolicy,
};
pub(super) use crate::facade::diagnostics::{
    DiagnosticCode, DiagnosticsArtifactKind, DiagnosticsScope,
};
pub(super) use crate::facade::durability::{DurabilityMode, DurableStoreLayout};
pub(super) use crate::facade::harness::RelationalHarnessAdapter;
pub(super) use crate::facade::history::{
    AspectHistoryCommitSpan, AspectHistoryEntry, AspectResolutionContext, BranchId,
    HistoryAspectQueryTarget,
};
pub(super) use crate::facade::identity::{KindId, LineageId, PartitionId, RelationId};
pub(super) use crate::facade::publication::{
    PatchStreamPosition, PatchStreamRequest, PublicationStage, PublicationStatus,
    SubscriberResumeRequest, SubscriberStreamFailureClass,
};
pub(super) use crate::facade::query::{
    DeterministicQueryPlanKey, PlannedQueryPacket, QueryAccessContract, QueryExecutionShape,
    QueryLocalityClass, QueryOrderingContract, QueryParallelLegality, QueryParallelProfitability,
    QueryPlanEvidenceBasis, QueryScope, QuerySerialReason, ReductionDiscipline,
};
pub(super) use crate::facade::runtime::{
    EntityReadRecord, InvariantCatalog, InvariantClass, InvariantRegistration, InvariantRule,
    ProjectionAspectFilter, ProjectionAspectFilterMode, ProjectionAspectScope, RelationalRuntime,
    RelationalRuntimeApi,
};
pub(super) use crate::facade::schema::{
    AspectBinding, DeclaredAspectContractBinding, EntityKindRegistration,
    KindAspectContractDeclarations, RelationIntegrityDeclarations, RelationKindRegistration,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
pub(super) use crate::facade::transactions::{
    AspectFieldPatch, BulkEntityCreateIntent, CommitResult, CreateIntent, DeleteEntityIntent,
    DeleteRelationIntent, EntityMutationIntent, MutationIntent, PatchVsTruthDeltaReport, RecordRef,
    RelationMutationIntent, ReplaceEntityIntent, TransactionCommitError, UpdateEntityFieldsIntent,
    UpdateRelationEndpointsIntent, WorkerIntentBatch,
};
pub(super) use crate::mvcc::RelationalTransactionValidationInput;
pub(super) use crate::publication::cdc::planning::checkpoint_for_schema_version;
pub(super) use crate::publication::patch::data::{
    ordered_aspect_keys, PatchDetail, RecordStructuralChange,
};
pub(super) use crate::symbols::data::ClientKeySymbolPolicy;
use crate::tests::harness::model::truth_model::VisibleTruthSummary;
pub(super) use worth_foundational::facade::AspectKey;

// Test helper index:
// - `schema`: baseline schema builders plus declared-aspect fixtures
// - `runtime`: runtime builders, persisted-runtime builders, and store paths
// - `records`: common entity/relation verbs and branch-aware mutation helpers
// - `history`: aspect/history digest helpers and small read utilities
// - `inspection`: canonical inspection request builders and historical read helpers
// - `durability`: branch/recovery helpers for persisted round trips
// - `relation_integrity`: schema fixtures and scenario helpers for milestone-4 legality work
// - `savepoint`: hostile savepoint residue assertions for patch/subscriber surfaces
//
// Prefer reusing these helpers before introducing new ad hoc setup in test files.
#[path = "support/aspect_field_patches.rs"]
mod aspect_field_patches;
#[path = "support/durability.rs"]
mod durability;
#[path = "support/history.rs"]
mod history;
#[path = "support/inspection.rs"]
mod inspection;
#[path = "support/leased_records.rs"]
mod leased_records;
#[path = "support/records.rs"]
mod records;
#[path = "support/relation_integrity.rs"]
mod relation_integrity;
#[path = "support/runtime.rs"]
mod runtime;
#[path = "support/savepoint.rs"]
mod savepoint;
#[path = "support/schema.rs"]
mod schema;
#[path = "support/transaction_authority.rs"]
mod transaction_authority;

pub(crate) use aspect_field_patches::*;
pub(crate) use durability::*;
pub(crate) use history::*;
pub(crate) use inspection::*;
pub(crate) use leased_records::*;
pub(crate) use records::*;
pub(crate) use relation_integrity::*;
pub(crate) use runtime::*;
pub(crate) use savepoint::*;
pub(crate) use schema::*;
pub(crate) use transaction_authority::*;

pub(crate) fn diagnostic_field<'a>(
    entry: &'a crate::facade::diagnostics::RelationalDiagnosticsEntry,
    field: &str,
) -> &'a crate::diagnostics::data::RelationalDiagnosticValue {
    diagnostic_field_optional(entry, field)
        .unwrap_or_else(|| panic!("diagnostic field '{field}' missing from {entry:?}"))
}

pub(crate) fn diagnostic_field_optional<'a>(
    entry: &'a crate::facade::diagnostics::RelationalDiagnosticsEntry,
    field: &str,
) -> Option<&'a crate::diagnostics::data::RelationalDiagnosticValue> {
    let crate::diagnostics::data::RelationalDiagnosticValue::Object(fields) = entry.fields.root()
    else {
        panic!("diagnostic entry fields are not an object: {entry:?}");
    };
    fields.get(field)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AspectTruthBundle {
    pub visible_truth: VisibleTruthSummary,
    pub latest_patch: Option<crate::facade::publication::PublishedAuthoritativePatchEnvelope>,
    pub latest_replay: Option<crate::facade::runtime::RelationalReplayRecord>,
    pub diagnostics: crate::facade::diagnostics::RelationalDiagnosticsFacade,
    pub entity_history_digests: Vec<(
        crate::facade::identity::EntityId,
        crate::facade::history::AspectHistoryDigest,
    )>,
    pub relation_history_digests: Vec<(RelationId, crate::facade::history::AspectHistoryDigest)>,
    pub lineage_history_digests: Vec<(
        LineageId,
        crate::facade::history::LineageAspectResolutionDigest,
    )>,
}

pub(super) fn capture_aspect_truth_bundle(
    runtime: &RelationalRuntime,
    entity_ids: &[crate::facade::identity::EntityId],
    relation_ids: &[RelationId],
    lineage_ids: &[LineageId],
) -> AspectTruthBundle {
    AspectTruthBundle {
        visible_truth: VisibleTruthSummary::capture(runtime),
        latest_patch: runtime.publication().artifacts().latest_patch(),
        latest_replay: runtime.publication().artifacts().latest_replay(),
        diagnostics: runtime.publication().diagnostics(),
        entity_history_digests: entity_ids
            .iter()
            .map(|entity_id| {
                (
                    *entity_id,
                    entity_aspect_history_digest(runtime, *entity_id, None),
                )
            })
            .collect(),
        relation_history_digests: relation_ids
            .iter()
            .map(|relation_id| {
                (
                    *relation_id,
                    relation_aspect_history_digest(runtime, *relation_id, None),
                )
            })
            .collect(),
        lineage_history_digests: lineage_ids
            .iter()
            .map(|lineage_id| {
                (
                    *lineage_id,
                    lineage_aspect_history_digest(runtime, *lineage_id, None),
                )
            })
            .collect(),
    }
}

pub(super) fn planned_explicit_query(
    runtime: &RelationalRuntime,
    snapshot: &crate::snapshots::data::SnapshotHandle,
    label: &str,
    targets: Vec<RecordRef>,
) -> crate::query::data::SnapshotPinnedQueryPlan {
    runtime
        .read_truth()
        .plan_query_packet(
            snapshot,
            explicit_query_packet(runtime, snapshot, label, targets),
        )
        .expect("planned explicit query")
}

pub(super) fn explicit_query_packet(
    runtime: &RelationalRuntime,
    snapshot: &crate::snapshots::data::SnapshotHandle,
    label: &str,
    targets: Vec<RecordRef>,
) -> PlannedQueryPacket {
    let context = runtime
        .read_truth()
        .query_plan_context(snapshot)
        .expect("query plan context");
    PlannedQueryPacket::explicit_targets(label, context, targets)
}

pub(super) fn execute_explicit_query(
    runtime: &RelationalRuntime,
    snapshot: &crate::snapshots::data::SnapshotHandle,
    label: &str,
    targets: Vec<RecordRef>,
) -> crate::query::data::QueryExecutionOutcome {
    runtime
        .read_truth()
        .execute_query_plan(planned_explicit_query(runtime, snapshot, label, targets))
        .expect("query execution outcome")
}

pub(super) fn assert_stable_aspect_truth_bundle_eq(
    expected: &AspectTruthBundle,
    actual: &AspectTruthBundle,
) {
    assert_eq!(expected.visible_truth, actual.visible_truth);
    assert_eq!(
        expected.entity_history_digests,
        actual.entity_history_digests
    );
    assert_eq!(
        expected.relation_history_digests,
        actual.relation_history_digests
    );
    assert_eq!(
        expected.lineage_history_digests,
        actual.lineage_history_digests
    );
}

pub(super) fn assert_recovered_commit_truth_matches(
    original_runtime: &RelationalRuntime,
    recovered_runtime: &RelationalRuntime,
    commit_id: crate::facade::history::CommitId,
    entity_ids: &[crate::facade::identity::EntityId],
    relation_ids: &[RelationId],
    lineage_ids: &[LineageId],
) {
    let original_envelope = original_runtime
        .replay()
        .canonical_commit_envelope(commit_id)
        .unwrap();
    let recovered_envelope = recovered_runtime
        .replay()
        .canonical_commit_envelope(commit_id)
        .unwrap();
    let original_bundle =
        capture_aspect_truth_bundle(original_runtime, entity_ids, relation_ids, lineage_ids);
    let recovered_bundle =
        capture_aspect_truth_bundle(recovered_runtime, entity_ids, relation_ids, lineage_ids);

    assert_stable_aspect_truth_bundle_eq(&original_bundle, &recovered_bundle);
    assert_eq!(original_envelope, recovered_envelope);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct InspectionTruthBundle {
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

pub(super) fn capture_inspection_truth_bundle(
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
        graph_summary: inspection.graph_summary(
            &crate::facade::inspection::GraphInspectionRequest {
                scope: crate::facade::inspection::InspectionScope::Current,
                partition_scope: None,
                relation_kind_scope: None,
                summary_only: true,
                budget: crate::tests::support::inspection::default_graph_budget(),
            },
        ),
        kind_summary: inspection.kind_summary(&crate::facade::inspection::KindInspectionRequest {
            scope: crate::facade::inspection::InspectionScope::Current,
            partition_scope: None,
            kind_id: KindId(1),
            record_class: crate::facade::inspection::InspectionRecordClass::Entity,
        }),
        connectivity_summary: inspection.connectivity_summary(
            &crate::facade::inspection::ConnectivityInspectionRequest {
                scope: crate::facade::inspection::InspectionScope::Current,
                partition_scope: None,
                relation_kind_scope: None,
                include_members: false,
                budget: crate::tests::support::inspection::default_connectivity_budget(),
            },
        ),
        historical_record: inspection.inspect_historical_record(
            branch_id,
            historical_version,
            RecordRef::Entity(entity_id),
            crate::facade::inspection::HistoricalInspectionMode::AllowCanonicalReconstruction,
        ),
        retention_summary: inspection
            .retention_summary(&crate::tests::support::inspection::default_retention_request()),
        record_retention: inspection
            .inspect_record_retention(RecordRef::Entity(entity_id))
            .expect("record retention"),
        branch_head: inspection.inspect_branch_head(branch_id),
        latest_commit: inspection
            .inspect_commit(latest_commit_id)
            .expect("latest commit inspection"),
        recent_commits: inspection.inspect_recent_commits(
            &crate::facade::inspection::RecentCommitInspectionRequest {
                branch_id: Some(branch_id.clone()),
                limit: 8,
            },
        ),
    }
}

pub(super) fn assert_patch_truth_invariants(result: &CommitResult) -> PatchVsTruthDeltaReport {
    let patch_vs_truth = result.patch_vs_truth_delta_report();
    let tag_accuracy = result.aspect_tag_accuracy_report();

    assert!(
        patch_vs_truth.exact_match,
        "patch surface diverged from canonical aspect truth: {:?}",
        patch_vs_truth
    );
    assert_eq!(patch_vs_truth.records_checked, result.patch().len() as u64);
    assert_eq!(tag_accuracy.records_checked, result.patch().len() as u64);
    assert_eq!(
        tag_accuracy.correctly_tagged_records,
        result.patch().len() as u64
    );

    patch_vs_truth
}

pub(super) fn assert_direct_history_origin_invariants(
    entries: &[AspectHistoryEntry],
    target: RecordRef,
) {
    assert!(
        !entries.is_empty(),
        "expected direct aspect history entries for {:?}",
        target
    );
    assert!(entries.iter().all(|entry| entry.origin.target == target));
    assert!(entries.iter().all(|entry| matches!(
        entry.resolution,
        AspectResolutionContext::DirectRecordHistory
    )));
}

pub(super) fn assert_lineage_history_origin_invariants(
    entries: &[AspectHistoryEntry],
    start_lineage_id: LineageId,
) {
    assert!(
        !entries.is_empty(),
        "expected lineage-aware aspect history entries for {:?}",
        start_lineage_id
    );
    assert!(entries.iter().all(|entry| matches!(
        entry.resolution,
        AspectResolutionContext::ResolvedViaLineage {
            start_lineage_id: resolved_start,
            ..
        } if resolved_start == start_lineage_id
    )));
}
