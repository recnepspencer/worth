use worth_foundational::facade::{
    aspects, AspectContract, AspectContractRevision, AspectIdentity, AspectKey, AspectValue,
    FieldKey, ScalarAspectType, StructAspectValue,
};
use worth_runtime_bridge::facade::{
    encode_snapshot_aspect_read_value, materialize_relational_authoritative_row_set,
};
use worth_runtime_bridge::facade::{
    RelationalBridgeRecordIdentityParts, SnapshotReadContract, SnapshotReadPacket,
    SnapshotReadPacketResult, SnapshotReadRecord, SnapshotReadRequest,
};

use super::super::super::{
    ConsumedProjectionFactSet, MaterializedProjectionContract, ProjectMaterializedFacts,
    ProjectionConsumptionSource, ProjectionFactExtractionError, ProjectionFactKind,
    ProjectionSourceFamily,
};
use super::support::{
    admitted, binding, canonical_field_path, phase_four_commit_identity,
    phase_four_snapshot_identity, phase_four_truth_snapshot_identity, relational_row_set,
    test_entity_identity, write_receipt,
};
use crate::runtime::{WorthQueryMutationTargetClass, WorthQueryWriteReceipt};

#[test]
fn relational_row_set_extracts_identity_and_field_facts() {
    let row_set = relational_row_set();
    let contract = admitted(
        ProjectionConsumptionSource::from_relational_row_set(&row_set),
        binding(&["identity.id", "profile.display_name"]),
        ProjectMaterializedFacts::declare()
            .entity_identities()
            .view_local_identities()
            .display_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            )
            .derived_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            ),
    )
    .bind_contract();

    let consumed = contract.extract_from_relational_row_set(&row_set).unwrap();

    assert_eq!(consumed.entity_identities().len(), 2);
    assert_eq!(
        consumed.entity_identities()[0].source_row_identity(),
        "entity:1:1:1"
    );
    assert_eq!(
        consumed.entity_identities()[0].entity_identity(),
        &test_entity_identity("task-1")
    );
    assert_eq!(consumed.view_local_identities().len(), 2);
    assert_eq!(
        consumed.view_local_identities()[0].view_local_identity(),
        "entity:1:1:1"
    );
    assert_eq!(consumed.display_fields().len(), 2);
    assert_eq!(consumed.derived_fields().len(), 2);
    assert_eq!(consumed.counters().declared_fact_family_count(), 4);
    assert_eq!(consumed.counters().extracted_fact_count(), 8);
    assert_eq!(consumed.counters().source_row_width_consumed(), 6);
    assert_eq!(
        consumed.display_fields()[0]
            .field_path()
            .canonical_field_path(),
        Some(&canonical_field_path("profile.display_name"))
    );
}

#[test]
fn relational_row_set_preserves_struct_facts_and_typed_refinement_denials() {
    let entity = RelationalBridgeRecordIdentityParts::entity(1, 1, 1);
    let identity_read = relational_snapshot_read(entity, "identity.id");
    let profile_read = SnapshotReadRequest::for_relational_record(
        entity,
        SnapshotReadContract::new(AspectContract::struct_aspect(
            AspectKey::new("profile").unwrap(),
            AspectIdentity(27),
            AspectContractRevision(1),
            aspects()
                .struct_fields()
                .required("name", ScalarAspectType::String)
                .finish()
                .unwrap(),
        )),
    );
    let profile = StructAspectValue::new([(
        FieldKey::new("name").unwrap(),
        AspectValue::String("Ada".into()),
    )])
    .unwrap();
    let packet = SnapshotReadPacket::new(vec![identity_read.clone(), profile_read.clone()]);
    let row_set = materialize_relational_authoritative_row_set(
        &packet,
        &SnapshotReadPacketResult::new(
            phase_four_truth_snapshot_identity("snapshot-struct"),
            vec![
                SnapshotReadRecord::for_request(
                    &identity_read,
                    AspectValue::String("task-1".into()),
                ),
                SnapshotReadRecord::for_request(&profile_read, profile.clone()),
            ],
        ),
    )
    .unwrap();
    let contract = admitted(
        ProjectionConsumptionSource::from_relational_row_set(&row_set),
        binding(&["profile"]),
        ProjectMaterializedFacts::declare().derived_field_path(
            crate::projection_consumption::projection_fact_field_path_from_segments([
                FieldKey::new("profile").unwrap(),
            ]),
        ),
    )
    .bind_contract();

    let consumed = contract.extract_from_relational_row_set(&row_set).unwrap();
    let fact = &consumed.derived_fields()[0];
    assert_eq!(fact.as_struct().unwrap(), &profile);
    let denial = fact.as_int64().unwrap_err();
    assert_eq!(
        denial.expected(),
        worth_foundational::facade::AspectValuePosture::Scalar(ScalarAspectType::Int64)
    );
    assert_eq!(
        denial.actual(),
        worth_foundational::facade::AspectValuePosture::Struct
    );
    assert_eq!(
        denial.field_path().canonical_field_path(),
        Some(&canonical_field_path("profile"))
    );
    assert_eq!(
        denial.source_family(),
        ProjectionSourceFamily::RelationalRowSet
    );
    assert_eq!(denial.projection_authority(), contract.contract_digest());
}

#[test]
fn write_receipt_extracts_aftermath_and_source_reference_facts() {
    let receipt = write_receipt();
    let contract = admitted(
        ProjectionConsumptionSource::from_write_receipt(&receipt),
        binding(&[]),
        ProjectMaterializedFacts::declare()
            .target_identity()
            .source_references()
            .effect_continuity_facts()
            .relation_endpoints(),
    )
    .bind_contract();

    let consumed = contract.extract_from_write_receipt(&receipt).unwrap();

    assert_eq!(
        consumed.target_identities()[0].target_identity(),
        &test_entity_identity("task-1")
    );
    assert_eq!(consumed.source_references().len(), 2);
    assert_eq!(
        consumed.effect_continuity_facts()[0]
            .prior_authoritative_identity()
            .label(),
        "task-0"
    );
    assert_eq!(
        consumed.relation_endpoints()[0].target_class(),
        Some(WorthQueryMutationTargetClass::Entity)
    );
    assert_eq!(consumed.relation_endpoints()[0].collection(), Some("tasks"));
    assert_eq!(consumed.counters().source_evidence_lookup_width(), 4);
    assert_eq!(consumed.counters().extracted_fact_count(), 5);
}

#[test]
fn effect_continuity_fact_digest_changes_with_typed_authority_identity() {
    let left = write_receipt();
    let right = WorthQueryWriteReceipt::test_only(
        phase_four_commit_identity("commit:test"),
        phase_four_snapshot_identity("snapshot:test"),
        WorthQueryMutationTargetClass::Entity,
        Some("tasks"),
        Some(test_entity_identity("task-1")),
        Some("bridge-record:test"),
        Some("$same_batch_target"),
        Some(
            crate::runtime::WorthQueryContinuityMutationEvidence::test_only(
                crate::runtime::WorthQueryContinuityMutationFamily::RebindExistingTarget,
                crate::runtime::WorthQueryContinuityOutcomeClass::ContinuesAsSingleSuccessor,
                "task-0",
                vec!["task-2".to_string()],
                Some(test_entity_identity("task-1")),
                Some("tasks"),
            ),
        ),
    );
    let contract = admitted(
        ProjectionConsumptionSource::from_write_receipt(&left),
        binding(&[]),
        ProjectMaterializedFacts::declare()
            .effect_continuity_facts()
            .source_references(),
    )
    .bind_contract();

    let left_consumed = contract.extract_from_write_receipt(&left).unwrap();
    let right_consumed = contract.extract_from_write_receipt(&right).unwrap();

    assert_ne!(
        left_consumed.fact_set_digest(),
        right_consumed.fact_set_digest()
    );
    assert_ne!(
        left_consumed.effect_continuity_facts()[0].successor_authoritative_identities()[0]
            .evidence_identity(),
        right_consumed.effect_continuity_facts()[0].successor_authoritative_identities()[0]
            .evidence_identity()
    );
}

#[test]
fn extraction_rejects_missing_field_evidence_and_family_mismatch() {
    let row_set = relational_row_set();
    let display_contract = admitted(
        ProjectionConsumptionSource::from_relational_row_set(&row_set),
        binding(&["profile.missing"]),
        ProjectMaterializedFacts::declare().display_field_path(
            crate::projection_consumption::projection_fact_field_path_from_segments([
                worth_foundational::facade::FieldKey::new("profile")
                    .expect("projection fact field segment should admit"),
                worth_foundational::facade::FieldKey::new("missing")
                    .expect("projection fact field segment should admit"),
            ]),
        ),
    )
    .bind_contract();
    let field_error = display_contract
        .extract_from_relational_row_set(&row_set)
        .unwrap_err();
    assert!(matches!(
        field_error,
        ProjectionFactExtractionError::MissingDeclaredFieldEvidence { .. }
    ));

    let entity_one_display_read = relational_snapshot_read(
        RelationalBridgeRecordIdentityParts::entity(1, 1, 1),
        "profile.display_name",
    );
    let entity_two_display_read = relational_snapshot_read(
        RelationalBridgeRecordIdentityParts::entity(1, 2, 1),
        "profile.display_name",
    );
    let missing_identity_packet = SnapshotReadPacket::new(vec![
        entity_one_display_read.clone(),
        entity_two_display_read.clone(),
    ]);
    let missing_identity_row_set = materialize_relational_authoritative_row_set(
        &missing_identity_packet,
        &SnapshotReadPacketResult::new(
            phase_four_truth_snapshot_identity("snapshot-a"),
            vec![
                SnapshotReadRecord::for_request(
                    &entity_one_display_read,
                    aspect_value(
                        crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                            "Task One",
                        ),
                    ),
                ),
                SnapshotReadRecord::for_request(
                    &entity_two_display_read,
                    aspect_value(
                        crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                            "Task Two",
                        ),
                    ),
                ),
            ],
        ),
    )
    .unwrap();
    let identity_contract = admitted(
        ProjectionConsumptionSource::from_relational_row_set(&missing_identity_row_set),
        binding(&["identity.id"]),
        ProjectMaterializedFacts::declare().entity_identities(),
    )
    .bind_contract();
    let identity_error = identity_contract
        .extract_from_relational_row_set(&missing_identity_row_set)
        .unwrap_err();
    assert!(matches!(
        identity_error,
        ProjectionFactExtractionError::MissingDeclaredFieldEvidence {
            fact_kind: ProjectionFactKind::EntityIdentity,
            ..
        }
    ));

    let _ = ProjectionSourceFamily::BridgeTruthViewRowSet;
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
        SnapshotReadContract::scalar(
            AspectKey::new(aspect).expect("valid snapshot aspect key"),
            ScalarAspectType::String,
        ),
    )
}

#[test]
fn write_receipt_extraction_rejects_source_identity_mismatch() {
    let contract = admitted(
        ProjectionConsumptionSource::from_write_receipt(&write_receipt()),
        binding(&[]),
        ProjectMaterializedFacts::declare().target_identity(),
    )
    .bind_contract();
    let mismatched_receipt = WorthQueryWriteReceipt::test_only(
        phase_four_commit_identity("commit:other"),
        phase_four_snapshot_identity("snapshot:test"),
        WorthQueryMutationTargetClass::Entity,
        Some("tasks"),
        Some(test_entity_identity("task-1")),
        None,
        None,
        None,
    );

    let error = contract
        .extract_from_write_receipt(&mismatched_receipt)
        .unwrap_err();
    assert!(matches!(
        error,
        ProjectionFactExtractionError::SourceIdentityMismatch { .. }
    ));
}

#[test]
fn write_receipt_extraction_rejects_missing_admitted_evidence() {
    let contract = admitted(
        ProjectionConsumptionSource::from_write_receipt(&write_receipt()),
        binding(&[]),
        ProjectMaterializedFacts::declare()
            .target_identity()
            .effect_continuity_facts()
            .relation_endpoints(),
    )
    .bind_contract();
    let stripped_receipt = WorthQueryWriteReceipt::test_only(
        phase_four_commit_identity("commit:test"),
        phase_four_snapshot_identity("snapshot:test"),
        WorthQueryMutationTargetClass::Entity,
        None,
        None,
        None,
        None,
        None,
    );

    let error = contract
        .extract_from_write_receipt(&stripped_receipt)
        .unwrap_err();
    assert!(matches!(
        error,
        ProjectionFactExtractionError::MissingWriteReceiptEvidence { .. }
    ));
}

#[test]
fn bridge_row_set_extraction_surface_is_publicly_reachable() {
    let _extract: fn(
        &MaterializedProjectionContract,
        &worth_runtime_bridge::facade::BridgeMaterializedRowSetArtifact,
    ) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> =
        MaterializedProjectionContract::extract_from_bridge_truth_view_row_set;
    let _ = ProjectionSourceFamily::BridgeTruthViewRowSet;
}
