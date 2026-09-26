use super::{
    CustomInvariantFailureIdentity, CustomInvariantFailureKind, CustomInvariantFailurePhase,
    InvariantViolation, InvariantViolationFields, RelationEndpointBoundary,
    StorageInconsistencyFailure, StorageInconsistencyLookup, StorageInconsistencyScan,
};
use crate::diagnostics::data::DiagnosticCode;
use crate::identity::data::{KindId, PartitionId, VersionId};
use crate::transactions::data::{CreatedEntityRef, EntityReference};
use crate::validation::data::InvariantClass;
use crate::validation::data::InvariantWitnessBasis;
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, CanonicalFieldPath, FieldKey, LocatorAuthority,
};

#[test]
fn snapshot_entity_limit_fields_use_named_version_surface() {
    let violation = InvariantViolation {
        class: InvariantClass::SnapshotAudit,
        code: DiagnosticCode::InvariantViolation,
        detail: "snapshot entity limit".to_string(),
        fields: InvariantViolationFields::SnapshotEntityLimit {
            version_id: VersionId::new(8),
            visible_entities: 13,
            limit: 5,
        },
    };

    match violation.fields {
        InvariantViolationFields::SnapshotEntityLimit {
            version_id,
            visible_entities,
            limit,
        } => {
            assert_eq!(version_id, VersionId::new(8));
            assert_eq!(visible_entities, 13);
            assert_eq!(limit, 5);
        }
        fields => panic!("expected typed snapshot entity limit fields, got {fields:?}"),
    }
}

#[test]
fn unique_entity_field_violation_uses_foundational_field_and_value_carriers() {
    let field = FieldKey::new("email").expect("valid field key");
    let field_locator = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        AspectKey::new("profile.email").expect("valid aspect key"),
        CanonicalFieldPath::single(field.clone()),
    );
    let value = AspectValue::String("duplicate@example.test".into());
    let violation = InvariantViolation {
        class: InvariantClass::CommitBoundary,
        code: DiagnosticCode::InvariantViolation,
        detail: "unique field duplicate".to_string(),
        fields: InvariantViolationFields::UniqueEntityField {
            field_locator: field_locator.clone(),
            value: value.clone(),
        },
    };

    match &violation.fields {
        InvariantViolationFields::UniqueEntityField {
            field_locator: observed_field_locator,
            value: observed_value,
        } => {
            assert_eq!(observed_field_locator, &field_locator);
            assert_eq!(observed_value, &value);
        }
        fields => panic!("expected typed unique entity field violation, got {fields:?}"),
    }
    let witness = violation.witness_key();
    let witness_parts = witness.as_str().split(':').collect::<Vec<_>>();
    assert_eq!(witness_parts.len(), 4);
    assert_eq!(witness_parts[0], "unique_entity_aspect_field");
    assert_eq!(witness_parts[1], "InvariantViolation");
    assert!(!witness.as_str().contains("profile.email"));
    assert!(!witness.as_str().contains(":email:"));
    assert!(!violation
        .witness_key()
        .as_str()
        .contains("duplicate@example.test"));
    match witness.basis() {
        InvariantWitnessBasis::UniqueEntityAspectField {
            field_locator: witness_field_locator,
            value: witness_value,
            field_locator_canonical_bytes,
            value_canonical_bytes,
        } => {
            assert_eq!(witness_field_locator, &field_locator);
            assert_eq!(witness_value, &value);
            assert!(!field_locator_canonical_bytes.is_empty());
            assert!(!value_canonical_bytes.is_empty());
        }
        basis => panic!("expected typed unique entity aspect-field witness basis, got {basis:?}"),
    }
}

#[test]
fn relation_endpoint_kind_mismatch_witness_key_uses_named_kind_identity_surface() {
    let source = EntityReference::Created(CreatedEntityRef {
        partition_id: PartitionId::new(3),
        kind_id: KindId::new(11),
        client_key: crate::symbols::data::ClientKey::raw("source"),
    });
    let target = EntityReference::Created(CreatedEntityRef {
        partition_id: PartitionId::new(4),
        kind_id: KindId::new(12),
        client_key: crate::symbols::data::ClientKey::raw("target"),
    });
    let violation = InvariantViolation {
        class: InvariantClass::CommitBoundary,
        code: DiagnosticCode::InvariantViolation,
        detail: "endpoint mismatch".to_string(),
        fields: InvariantViolationFields::RelationEndpointKindMismatch {
            contract_id: "contract".into(),
            relation_kind_id: KindId::new(9),
            source: source.clone(),
            target: target.clone(),
            source_kind_id: KindId::new(11),
            target_kind_id: KindId::new(12),
            boundary: RelationEndpointBoundary::Source,
        },
    };

    assert_eq!(
        violation.witness_key().as_str(),
        format!("endpoint_kind_mismatch:9:{source:?}:{target:?}:source")
    );
}

#[test]
fn sidecar_consistency_fields_use_named_partition_surface() {
    let violation = InvariantViolation {
        class: InvariantClass::AlwaysOnStructural,
        code: DiagnosticCode::InvariantViolation,
        detail: "sidecar mismatch".to_string(),
        fields: InvariantViolationFields::SidecarConsistency {
            partition_id: PartitionId::new(7),
            slot: 42,
            missing_label: "kind_sidecar".to_string(),
        },
    };

    match violation.fields {
        InvariantViolationFields::SidecarConsistency {
            partition_id,
            slot,
            missing_label,
        } => {
            assert_eq!(partition_id, PartitionId::new(7));
            assert_eq!(slot, 42);
            assert_eq!(missing_label, "kind_sidecar");
        }
        fields => panic!("expected typed sidecar consistency fields, got {fields:?}"),
    }
}

#[test]
fn storage_inconsistency_fields_use_typed_labels_and_preserve_field_context() {
    let expected_field = FieldKey::new("email").expect("valid field key");
    let violation = InvariantViolation {
        class: InvariantClass::AlwaysOnStructural,
        code: DiagnosticCode::StorageInconsistencyDetected,
        detail: "storage inconsistency".to_string(),
        fields: InvariantViolationFields::StorageInconsistency {
            entity_id: None,
            partition_id: Some(PartitionId::new(7)),
            slot: Some(42),
            field: Some(expected_field.clone()),
            missing_label: Some("kind_sidecar".to_string()),
            scan: Some(StorageInconsistencyScan::HistoricalUniqueEntityAspectField),
            lookup: Some(StorageInconsistencyLookup::EntityKindInState),
            failure: Some(StorageInconsistencyFailure::MissingKindId),
        },
    };

    match violation.fields {
        InvariantViolationFields::StorageInconsistency {
            partition_id,
            slot,
            field,
            missing_label,
            scan,
            lookup,
            failure,
            ..
        } => {
            assert_eq!(partition_id, Some(PartitionId::new(7)));
            assert_eq!(slot, Some(42));
            assert_eq!(field.as_ref(), Some(&expected_field));
            assert_eq!(missing_label.as_deref(), Some("kind_sidecar"));
            assert_eq!(
                scan,
                Some(StorageInconsistencyScan::HistoricalUniqueEntityAspectField)
            );
            assert_eq!(lookup, Some(StorageInconsistencyLookup::EntityKindInState));
            assert_eq!(failure, Some(StorageInconsistencyFailure::MissingKindId));
        }
        fields => panic!("expected typed storage inconsistency fields, got {fields:?}"),
    }
}

#[test]
fn storage_inconsistency_witness_key_uses_typed_labels() {
    let violation = InvariantViolation {
        class: InvariantClass::AlwaysOnStructural,
        code: DiagnosticCode::StorageInconsistencyDetected,
        detail: "storage inconsistency".to_string(),
        fields: InvariantViolationFields::StorageInconsistency {
            entity_id: None,
            partition_id: Some(PartitionId::new(7)),
            slot: Some(42),
            field: Some(FieldKey::new("email").expect("valid field key")),
            missing_label: None,
            scan: Some(StorageInconsistencyScan::LiveRecordSidecar),
            lookup: Some(StorageInconsistencyLookup::EntityKindInState),
            failure: Some(StorageInconsistencyFailure::MissingSlot),
        },
    };

    assert_eq!(
        violation.witness_key().as_str(),
        "storage_inconsistency:None:Some(PartitionId(7)):Some(42):email:live_record_sidecar:entity_kind_in_state:missing_slot"
    );
}

#[test]
fn custom_invariant_failure_fields_use_typed_phase_and_failure_labels() {
    let violation = InvariantViolation {
        class: InvariantClass::CommitBoundary,
        code: DiagnosticCode::InvariantViolation,
        detail: "custom invariant failed".to_string(),
        fields: InvariantViolationFields::CustomInvariantFailure {
            identity: CustomInvariantFailureIdentity::new(
                crate::validation::data::CustomInvariantSemanticIdentity {
                    rule_id: crate::validation::data::CustomInvariantRuleId::new("custom.rule"),
                    semantic_version: crate::validation::data::CustomInvariantSemanticVersion::new(
                        2, 3,
                    ),
                },
            ),
            phase: CustomInvariantFailurePhase::Execution,
            failure: CustomInvariantFailureKind::Panic,
            detail: "panic detail".to_string(),
        },
    };

    match violation.fields {
        InvariantViolationFields::CustomInvariantFailure {
            identity,
            phase,
            failure,
            detail,
        } => {
            assert_eq!(identity.semantic_identity().rule_id.as_str(), "custom.rule");
            assert_eq!(identity.semantic_identity().semantic_version.major, 2);
            assert_eq!(identity.semantic_identity().semantic_version.minor, 3);
            assert_eq!(phase, CustomInvariantFailurePhase::Execution);
            assert_eq!(failure, CustomInvariantFailureKind::Panic);
            assert_eq!(detail, "panic detail");
        }
        fields => panic!("expected typed custom invariant failure fields, got {fields:?}"),
    }
}

#[test]
fn custom_invariant_failure_witness_key_uses_typed_phase_and_failure_labels() {
    let violation = InvariantViolation {
        class: InvariantClass::CommitBoundary,
        code: DiagnosticCode::InvariantViolation,
        detail: "custom invariant failed".to_string(),
        fields: InvariantViolationFields::CustomInvariantFailure {
            identity: CustomInvariantFailureIdentity::new(
                crate::validation::data::CustomInvariantSemanticIdentity {
                    rule_id: crate::validation::data::CustomInvariantRuleId::new("custom.rule"),
                    semantic_version: crate::validation::data::CustomInvariantSemanticVersion::new(
                        2, 3,
                    ),
                },
            ),
            phase: CustomInvariantFailurePhase::Preparation,
            failure: CustomInvariantFailureKind::PreparationError,
            detail: "prepare detail".to_string(),
        },
    };

    assert_eq!(
        violation.witness_key().as_str(),
        "custom_failure:custom.rule:2.3:preparation:preparation_error"
    );
}
