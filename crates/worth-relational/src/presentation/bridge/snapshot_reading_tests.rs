use std::sync::Arc;

use worth_foundational::facade::{AspectKey, AspectValue, ScalarAspectType};

use crate::facade::config::CascadeDeletePolicy;
use crate::presentation::bridge::relational_test_support::{
    changed_relations, create_entity, create_entity_outcome, create_relation_outcome,
    runtime_with_declared_aspect_schema,
};
use worth_runtime_bridge::facade::{
    RelationalBridgeRecordIdentityParts, SnapshotReadContract, SnapshotReadPacket,
    SnapshotReadRequest, TruthSnapshotReader,
};

use super::bridge_snapshot_identity_for_handle;
use crate::presentation::bridge::snapshot_reading::RuntimePublicationSnapshotReader;

#[test]
fn snapshot_reader_reads_published_entity_values_without_projection_surface() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "visible");
    let published_snapshot = created.snapshot.clone();
    let published_entity = runtime
        .read_truth()
        .read_snapshot(&published_snapshot)
        .expect("published snapshot read")
        .entities()
        .first()
        .expect("published entity record")
        .entity_id;

    let snapshot_identity = bridge_snapshot_identity_for_handle(&published_snapshot);
    let reader = reader_for_current(
        runtime,
        snapshot_identity.clone(),
        published_snapshot.version_id(),
    );
    let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_relational_record(
        RelationalBridgeRecordIdentityParts::entity(
            published_entity.partition_id.0,
            published_entity.local_slot.0,
            published_entity.generation.0,
        ),
        scalar_string_contract("name"),
    )]);

    let result = reader.read_packet(&packet).expect("published packet read");

    assert_eq!(result.snapshot_identity(), &snapshot_identity);
    assert_eq!(result.records().len(), 1);
    assert_eq!(
        result.records()[0].scalar_aspect_value(),
        Some(&AspectValue::String("visible".into()))
    );
}

#[test]
fn snapshot_reader_reads_published_relation_field_aspects_from_authoritative_state() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let created_relation = create_relation_outcome(&runtime, source, target, "visible-edge");
    let relation = changed_relations(&created_relation)[0];
    let published_snapshot = created_relation.snapshot.clone();

    let snapshot_identity = bridge_snapshot_identity_for_handle(&published_snapshot);
    let reader = reader_for_current(
        runtime,
        snapshot_identity.clone(),
        published_snapshot.version_id(),
    );
    let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_relational_record(
        RelationalBridgeRecordIdentityParts::relation(
            relation.partition_id.0,
            relation.local_slot.0,
            relation.generation.0,
        ),
        scalar_string_contract("label"),
    )]);

    let result = reader
        .read_packet(&packet)
        .expect("published relation aspect packet read");

    assert_eq!(result.snapshot_identity(), &snapshot_identity);
    assert_eq!(result.records().len(), 1);
    assert_eq!(
        result.records()[0].scalar_aspect_value(),
        Some(&AspectValue::String("visible-edge".into()))
    );
}

#[test]
fn snapshot_reader_rejects_undeclared_dotted_document_paths() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let created = create_entity_outcome(&runtime, "visible");
    let published_snapshot = created.snapshot.clone();
    let published_entity =
        crate::presentation::bridge::relational_test_support::changed_entities(&created)[0];

    let snapshot_identity = bridge_snapshot_identity_for_handle(&published_snapshot);
    let reader = reader_for_current(
        runtime,
        snapshot_identity.clone(),
        published_snapshot.version_id(),
    );
    let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_relational_record(
        RelationalBridgeRecordIdentityParts::entity(
            published_entity.partition_id.0,
            published_entity.local_slot.0,
            published_entity.generation.0,
        ),
        scalar_string_contract("profile.name"),
    )]);

    let error = reader
        .read_packet(&packet)
        .expect_err("flat aspect path should not satisfy dotted aspect path");

    assert!(
        error
            .to_string()
            .contains("could not resolve aspect `profile.name`"),
        "unexpected bridge snapshot error: {error}"
    );
}

#[test]
fn snapshot_reader_rejects_untyped_bridge_record_identity() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "visible");
    let published_snapshot = created.snapshot.clone();

    let snapshot_identity = bridge_snapshot_identity_for_handle(&published_snapshot);
    let reader = reader_for_current(runtime, snapshot_identity, published_snapshot.version_id());
    let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_coarse(
        // allowed-untyped-negative-test
        "legacy-row",
        scalar_string_contract("name"),
    )]);

    let error = reader
        .read_packet(&packet)
        .expect_err("relational snapshot reads require typed record identity parts");

    assert!(
        error
            .to_string()
            .contains("requires typed record identity parts"),
        "unexpected bridge snapshot error: {error}"
    );
}

#[test]
fn snapshot_reader_reports_missing_record_as_authoritative_absence() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "visible");
    let published_snapshot = created.snapshot.clone();
    let snapshot_identity = bridge_snapshot_identity_for_handle(&published_snapshot);
    let reader = reader_for_current(runtime, snapshot_identity, published_snapshot.version_id());
    let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_relational_record(
        RelationalBridgeRecordIdentityParts::entity(1, 999, 1),
        scalar_string_contract("name"),
    )]);

    let result = reader
        .read_packet(&packet)
        .expect("absence is authoritative");

    assert_eq!(result.records().len(), 1);
    assert!(result.records()[0].is_absent());
}

fn runtime_with_test_schema() -> crate::facade::runtime::RelationalRuntime {
    runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations)
}

fn reader_for_current(
    runtime: crate::facade::runtime::RelationalRuntime,
    snapshot_identity: worth_runtime_bridge::facade::TruthSnapshotIdentity,
    expected_version: crate::facade::identity::VersionId,
) -> RuntimePublicationSnapshotReader {
    let branch = crate::facade::history::BranchId("main".to_owned());
    let identity = runtime
        .branch_identity(&branch)
        .expect("main branch identity");
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("main branch basis");
    let observation = basis.observation();
    assert_eq!(observation.version_id(), expected_version);
    RuntimePublicationSnapshotReader::for_observation_authority(
        crate::facade::change_source::RelationalRuntimeHandle::immutable(Arc::new(runtime)),
        snapshot_identity,
        observation,
        None,
    )
}

fn aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).expect("valid test aspect key")
}

fn scalar_string_contract(aspect: &str) -> SnapshotReadContract {
    SnapshotReadContract::scalar(aspect_key(aspect), ScalarAspectType::String)
}
