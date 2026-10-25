//! Retention projection preserves the original canonical carrier as bytes.
use super::{InvariantViolationDiagnosticProjection, RelationalDiagnosticValue};
use crate::validation::data::{
    CustomInvariantRefusal, CustomInvariantRuleId, CustomInvariantSemanticIdentity,
    CustomInvariantSemanticVersion, InvariantViolationFields,
};

#[test]
fn retained_violation_diagnostics_keep_canonical_refusal_bytes() {
    let bytes = br#"{"rule":1,"observed":"closed"}"#.to_vec();
    let fields = InvariantViolationFields::CustomInvariantViolation {
        identity: CustomInvariantSemanticIdentity {
            rule_id: CustomInvariantRuleId::new("tests.retained-refusal"),
            semantic_version: CustomInvariantSemanticVersion::new(1, 0),
        },
        refusal: CustomInvariantRefusal::DeclaredValue {
            value_identity: "tests.refusal.v1".into(),
            bytes: bytes.clone(),
        },
    };
    let encoded = serde_json::to_vec(&fields).unwrap();
    let retained: InvariantViolationFields = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(retained, fields);
    assert_eq!(serde_json::to_vec(&retained).unwrap(), encoded);
    let RelationalDiagnosticValue::Object(projected) =
        InvariantViolationDiagnosticProjection::from_fields(&retained).to_diagnostic_value()
    else {
        panic!("violation diagnostic object");
    };
    assert_eq!(
        projected.get("refusal"),
        Some(&RelationalDiagnosticValue::object([
            (
                "value_identity",
                RelationalDiagnosticValue::string("tests.refusal.v1")
            ),
            ("bytes", RelationalDiagnosticValue::CanonicalBytes(bytes)),
        ]))
    );
}

#[test]
fn partition_diagnostics_and_witnesses_retain_distinct_original_endpoints() {
    use crate::identity::data::{EntityId, KindId, PartitionId};
    use crate::schema::data::ContractId;
    use crate::symbols::data::ClientKey;
    use crate::transactions::data::{CreatedEntityRef, EntityReference};
    use crate::validation::data::{InvariantClass, InvariantViolation};
    let source = EntityReference::Existing(EntityId::new(PartitionId(1), 3, 1));
    let target = EntityReference::Created(CreatedEntityRef {
        partition_id: PartitionId(2),
        kind_id: KindId(5),
        client_key: ClientKey::raw("new-node"),
    });
    let fields = InvariantViolationFields::PartitionIsolation {
        contract_id: ContractId::new("same_partition"),
        relation_kind_id: KindId(6),
        relation_id: None,
        source_partition_id: source.partition_id(),
        target_partition_id: target.partition_id(),
        source: source.clone(),
        target: target.clone(),
    };
    let retained: InvariantViolationFields =
        serde_json::from_slice(&serde_json::to_vec(&fields).unwrap()).unwrap();
    assert_eq!(retained, fields);
    let InvariantViolationDiagnosticProjection::PartitionIsolation {
        source: actual_source,
        target: actual_target,
        ..
    } = InvariantViolationDiagnosticProjection::from_fields(&retained)
    else {
        panic!("typed partition fields")
    };
    assert_eq!(actual_source, &source);
    assert_eq!(actual_target, &target);
    let RelationalDiagnosticValue::Object(projected) =
        InvariantViolationDiagnosticProjection::from_fields(&retained).to_diagnostic_value()
    else {
        panic!("partition diagnostic object")
    };
    assert_eq!(
        projected.get("source"),
        Some(&RelationalDiagnosticValue::object([
            (
                "reference_kind",
                RelationalDiagnosticValue::string("existing")
            ),
            (
                "entity_id",
                RelationalDiagnosticValue::EntityId(EntityId::new(PartitionId(1), 3, 1))
            ),
        ]))
    );
    assert_eq!(
        projected.get("target"),
        Some(&RelationalDiagnosticValue::object([
            (
                "reference_kind",
                RelationalDiagnosticValue::string("created")
            ),
            (
                "partition_id",
                RelationalDiagnosticValue::PartitionId(PartitionId(2))
            ),
            ("kind_id", RelationalDiagnosticValue::KindId(KindId(5))),
            ("client_key", RelationalDiagnosticValue::string("new-node")),
        ]))
    );
    let violation = InvariantViolation {
        class: InvariantClass::CommitBoundary,
        code: crate::diagnostics::data::DiagnosticCode::InvariantViolation,
        detail: "partition".into(),
        fields,
    };
    let mut other = violation.clone();
    let InvariantViolationFields::PartitionIsolation { target, .. } = &mut other.fields else {
        unreachable!()
    };
    let EntityReference::Created(target) = target else {
        unreachable!()
    };
    target.client_key = ClientKey::raw("other-node");
    assert_ne!(violation.witness_key(), other.witness_key());
}
