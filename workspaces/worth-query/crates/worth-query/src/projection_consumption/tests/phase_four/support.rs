use crate::authoring::{
    AspectFieldSelector, AuthoredResultShapeField, GuidedAuthoringPath, RawAuthoredQuery,
    RawAuthoredResultShape, RootEntityKey,
};
use crate::authorized_projection::{
    AuthorizedProjectionArtifact, AuthorizedProjectionCounters, MaskedProjectionArtifact,
    PolicyFieldInfluenceSet,
};
use crate::memory_workspace::{
    WorthQueryCommitIdentity, WorthQueryEntity, WorthQueryEntityIdentity,
    WorthQuerySnapshotIdentity,
};
use crate::query_context::{QueryContextExecutionArtifact, QueryContextExecutionFamily};
use crate::runtime::{
    WorthQueryContinuityMutationEvidence, WorthQueryContinuityMutationFamily,
    WorthQueryContinuityOutcomeClass, WorthQueryMutationTargetClass, WorthQueryReadExecutionEngine,
    WorthQueryReadReceipt, WorthQueryReadResult, WorthQueryWriteReceipt,
};
use worth_foundational::facade::{AspectKey, AspectValue, ScalarAspectType};
use worth_foundational::facade::{CanonicalFieldPath, FieldKey};
use worth_runtime_bridge::facade::{
    encode_snapshot_aspect_read_value, materialize_relational_authoritative_row_set,
    project_relational_grouped_truth, RelationalAuthoritativeRowSetArtifact,
    RelationalGroupedProjectionArtifact, RelationalGroupedProjectionContract,
};
use worth_runtime_bridge::facade::{
    RelationalBridgeRecordIdentityParts, RelationalBridgeSnapshotIdentityParts,
    SnapshotReadContract, SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadRecord,
    SnapshotReadRequest, TruthSnapshotIdentity,
};

use super::super::super::{
    declare_projection_consumption, evaluate_projection_consumption_eligibility,
    AdmittedProjectionConsumption, ProjectMaterializedFacts, ProjectionConsumptionBindingContext,
    ProjectionConsumptionEligibility, ProjectionConsumptionSource,
};

pub(crate) fn binding(visible_fields: &[&str]) -> ProjectionConsumptionBindingContext {
    ProjectionConsumptionBindingContext::test_only(
        "result-shape:test",
        "authorized-projection:test",
        crate::projection_consumption::test_authorized_field_paths(visible_fields),
    )
}

pub(crate) fn authorized_projection(
    query_digest: &str,
    result_shape_digest: &str,
    visible_fields: &[&str],
) -> AuthorizedProjectionArtifact {
    AuthorizedProjectionArtifact::new(
        query_digest,
        result_shape_digest,
        "policy:test",
        "tenant-schema:test",
        crate::projection_consumption::test_authorized_field_paths(visible_fields),
        MaskedProjectionArtifact::new(Vec::new(), Vec::new()),
        "narrowed-result-shape:test".to_string(),
        PolicyFieldInfluenceSet::new(&["influence:test".to_string()], 1),
        AuthorizedProjectionCounters::default(),
    )
}

pub(crate) fn binding_for_result_shape(
    result_shape_digest: &str,
    visible_fields: &[&str],
) -> ProjectionConsumptionBindingContext {
    ProjectionConsumptionBindingContext::test_only_with_projection_metadata(
        result_shape_digest,
        "query:test",
        result_shape_digest,
        "authorized-projection:test",
        "narrowed-result-shape:test",
        "policy:test",
        "tenant-schema:test",
        crate::projection_consumption::test_authorized_field_paths(visible_fields),
    )
}

pub(crate) fn admitted(
    source: ProjectionConsumptionSource,
    binding: ProjectionConsumptionBindingContext,
    requested: ProjectMaterializedFacts,
) -> AdmittedProjectionConsumption {
    let declaration = declare_projection_consumption(source, binding, requested).unwrap();
    match evaluate_projection_consumption_eligibility(&declaration) {
        ProjectionConsumptionEligibility::Admitted(admitted) => admitted,
        ProjectionConsumptionEligibility::AdmittedWithWarnings(admitted, _) => admitted,
        other => panic!("expected admitted eligibility, got {other:?}"),
    }
}

pub(crate) fn relational_row_set() -> RelationalAuthoritativeRowSetArtifact {
    let entity_one = RelationalBridgeRecordIdentityParts::entity(1, 1, 1);
    let entity_two = RelationalBridgeRecordIdentityParts::entity(1, 2, 1);
    let entity_one_identity = relational_snapshot_read(entity_one, "identity.id");
    let entity_one_lane = relational_snapshot_read(entity_one, "status.lane");
    let entity_one_display = relational_snapshot_read(entity_one, "profile.display_name");
    let entity_two_identity = relational_snapshot_read(entity_two, "identity.id");
    let entity_two_lane = relational_snapshot_read(entity_two, "status.lane");
    let entity_two_display = relational_snapshot_read(entity_two, "profile.display_name");
    let packet = SnapshotReadPacket::new(vec![
        entity_one_identity.clone(),
        entity_one_lane.clone(),
        entity_one_display.clone(),
        entity_two_identity.clone(),
        entity_two_lane.clone(),
        entity_two_display.clone(),
    ]);
    let result = SnapshotReadPacketResult::new(
        phase_four_truth_snapshot_identity("snapshot-a"),
        vec![
            SnapshotReadRecord::for_request(
                &entity_one_identity,
                aspect_value(
                    crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("task-1"),
                ),
            ),
            SnapshotReadRecord::for_request(
                &entity_one_lane,
                aspect_value(
                    crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("todo"),
                ),
            ),
            SnapshotReadRecord::for_request(
                &entity_one_display,
                aspect_value(
                    crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                        "Task One",
                    ),
                ),
            ),
            SnapshotReadRecord::for_request(
                &entity_two_identity,
                aspect_value(
                    crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("task-2"),
                ),
            ),
            SnapshotReadRecord::for_request(
                &entity_two_lane,
                aspect_value(
                    crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("doing"),
                ),
            ),
            SnapshotReadRecord::for_request(
                &entity_two_display,
                aspect_value(
                    crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                        "Task Two",
                    ),
                ),
            ),
        ],
    );
    materialize_relational_authoritative_row_set(&packet, &result).unwrap()
}

pub(crate) fn relational_grouped_projection() -> RelationalGroupedProjectionArtifact {
    project_relational_grouped_truth(
        &relational_row_set(),
        grouped_projection_contract("status", "identity.id", "status.lane"),
    )
    .expect("grouped projection")
}

fn aspect_value(value: AspectValue) -> AspectValue {
    encode_snapshot_aspect_read_value(&value)
}

fn relational_snapshot_read(
    entity: RelationalBridgeRecordIdentityParts,
    aspect: &str,
) -> SnapshotReadRequest {
    SnapshotReadRequest::for_relational_record(
        entity,
        SnapshotReadContract::scalar(aspect_key(aspect), ScalarAspectType::String),
    )
}

fn grouped_projection_contract(
    grouping_aspect: &str,
    identity_binding_aspect: &str,
    grouping_binding_aspect: &str,
) -> RelationalGroupedProjectionContract {
    RelationalGroupedProjectionContract::new(
        aspect_key(grouping_aspect),
        aspect_key(identity_binding_aspect),
        aspect_key(grouping_binding_aspect),
    )
}

fn aspect_key(label: &str) -> AspectKey {
    AspectKey::new(label).expect("test aspect key must be foundational")
}

pub(crate) fn write_receipt() -> WorthQueryWriteReceipt {
    WorthQueryWriteReceipt::test_only(
        phase_four_commit_identity("commit:test"),
        phase_four_snapshot_identity("snapshot:test"),
        WorthQueryMutationTargetClass::Entity,
        Some("tasks"),
        Some(test_entity_identity("task-1")),
        Some("bridge-record:test"),
        Some("$same_batch_target"),
        Some(WorthQueryContinuityMutationEvidence::test_only(
            WorthQueryContinuityMutationFamily::RebindExistingTarget,
            WorthQueryContinuityOutcomeClass::ContinuesAsSingleSuccessor,
            "task-0",
            vec!["task-1".to_string()],
            Some(test_entity_identity("task-1")),
            Some("tasks"),
        )),
    )
}

pub(crate) fn write_receipt_without_source_references() -> WorthQueryWriteReceipt {
    WorthQueryWriteReceipt::test_only(
        phase_four_commit_identity("commit:test:no-source-ref"),
        phase_four_snapshot_identity("snapshot:test"),
        WorthQueryMutationTargetClass::Entity,
        Some("tasks"),
        Some(test_entity_identity("task-1")),
        None,
        None,
        Some(WorthQueryContinuityMutationEvidence::test_only(
            WorthQueryContinuityMutationFamily::RebindExistingTarget,
            WorthQueryContinuityOutcomeClass::ContinuesAsSingleSuccessor,
            "task-0",
            vec!["task-1".to_string()],
            Some(test_entity_identity("task-1")),
            Some("tasks"),
        )),
    )
}

pub(crate) fn read_result() -> WorthQueryReadResult {
    WorthQueryReadResult::test_only(
        vec![
            entity_from_projection(
                test_entity_identity("task-1"),
                [
                    ("profile.display_name", text_value("Task One")),
                    ("metrics.priority", int_value(1)),
                ],
            ),
            entity_from_projection(
                test_entity_identity("task-2"),
                [
                    ("profile.display_name", text_value("Task Two")),
                    ("metrics.priority", int_value(2)),
                ],
            ),
        ],
        WorthQueryReadReceipt::test_only(
            "read-graph:test",
            "query:test",
            "basis:test",
            "result:test",
            WorthQueryReadExecutionEngine::QueryRuntimeCurrent,
        ),
    )
}

pub(crate) fn entity_from_projection(
    identity: WorthQueryEntityIdentity,
    values: impl IntoIterator<Item = (&'static str, AspectValue)>,
) -> WorthQueryEntity {
    WorthQueryEntity::from_native_field_values(
        identity,
        values
            .into_iter()
            .map(|(path, value)| (canonical_field_path(path), value))
            .collect(),
    )
}

pub(crate) fn text_value(value: impl Into<String>) -> AspectValue {
    crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(value)
}

pub(crate) fn int_value(value: i64) -> AspectValue {
    AspectValue::Int64(value)
}

pub(crate) fn canonical_field_path(path: &str) -> CanonicalFieldPath {
    CanonicalFieldPath::new(
        path.split('.')
            .map(|segment| FieldKey::new(segment.to_string()))
            .collect::<Option<Vec<_>>>()
            .expect("test field path should be canonical"),
    )
    .expect("test field path should not be empty")
}

pub(crate) fn phase_four_commit_identity(label: &str) -> WorthQueryCommitIdentity {
    WorthQueryCommitIdentity::from_relational_commit_id(phase_four_fixture_position(
        "commit", label,
    ))
}

pub(crate) fn phase_four_snapshot_identity(label: &str) -> WorthQuerySnapshotIdentity {
    WorthQuerySnapshotIdentity::from_relational_snapshot(phase_four_snapshot_parts(label))
}

pub(crate) fn phase_four_truth_snapshot_identity(label: &str) -> TruthSnapshotIdentity {
    TruthSnapshotIdentity::from_relational_snapshot(phase_four_snapshot_parts(label))
}

fn phase_four_snapshot_parts(label: &str) -> RelationalBridgeSnapshotIdentityParts {
    RelationalBridgeSnapshotIdentityParts::new(
        phase_four_fixture_position("snapshot", label),
        phase_four_fixture_position("snapshot-version", label),
    )
}

fn phase_four_fixture_position(namespace: &str, evidence: &str) -> u64 {
    let mut acc = 14_695_981_039_346_656_037_u64;
    for byte in namespace.bytes().chain(evidence.bytes()) {
        acc ^= u64::from(byte);
        acc = acc.wrapping_mul(1_099_511_628_211_u64);
    }
    acc
}

pub(crate) fn test_entity_identity(identity: &str) -> WorthQueryEntityIdentity {
    crate::memory_workspace::admit_authored_entity_label(identity)
}

pub(crate) fn read_result_shape() -> crate::canonicalization::CanonicalResultShapeArtifact {
    let query = RawAuthoredQuery::detail_builder(RootEntityKey::new("task").unwrap())
        .project(AspectFieldSelector::new("profile", "display_name").unwrap())
        .project(AspectFieldSelector::new("metrics", "priority").unwrap())
        .build()
        .unwrap();
    let result_shape = RawAuthoredResultShape::detail_builder()
        .field(
            AuthoredResultShapeField::new("profile", "display_name", "profile.display_name")
                .unwrap(),
        )
        .field(AuthoredResultShapeField::new("metrics", "priority", "metrics.priority").unwrap())
        .build()
        .unwrap();
    GuidedAuthoringPath::canonicalize_detail(query, result_shape)
        .unwrap()
        .result_shape()
        .clone()
}

pub(crate) fn query_context_execution_current() -> QueryContextExecutionArtifact {
    QueryContextExecutionArtifact::test_only(
        QueryContextExecutionFamily::RuntimeCurrent,
        "query:test",
        "basis:test",
        "result:test",
        "result-shape:test",
        vec!["payload-row-0".to_string(), "payload-row-1".to_string()],
        None,
        None,
    )
}

pub(crate) fn query_context_execution_historical() -> QueryContextExecutionArtifact {
    QueryContextExecutionArtifact::test_only(
        QueryContextExecutionFamily::HistoricalMaterialized,
        "query:test",
        "basis:test",
        "result:test",
        "result-shape:test",
        vec!["historical-row-0".to_string()],
        Some("materialization-path:test"),
        None,
    )
}

pub(crate) fn query_context_execution_preview() -> QueryContextExecutionArtifact {
    QueryContextExecutionArtifact::test_only(
        QueryContextExecutionFamily::PreviewDerivedHistorical,
        "query:test",
        "basis:test",
        "result:test",
        "result-shape:test",
        vec!["preview-row-0".to_string()],
        None,
        Some("preview-provenance:test"),
    )
}
