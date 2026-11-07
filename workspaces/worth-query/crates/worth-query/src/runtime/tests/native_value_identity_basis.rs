use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::{
    prepare_aspect_value_identity_basis, prepare_struct_aspect_value_identity_basis, AspectValue,
    CanonicalBigInt, CanonicalDate, CanonicalDecimal, CanonicalF32, CanonicalF64,
    CanonicalRational, CanonicalTime, CanonicalTimestamp, CanonicalTimestampTz, ContentRefId,
    ContractValidationInput, EntityId, FieldKey, InternedString, PartitionId, StructAspectValue,
    Symbol,
};

use crate::projection_consumption::{
    bind_materialized_projection_contract, declare_projection_consumption,
    evaluate_projection_consumption_eligibility, projection_fact_field_path_from_segments,
    ConsumedFieldValueFact, ConsumedNativeValue, ConsumedProjectionContractProvenance,
    ConsumedProjectionFactInventory, ConsumedProjectionFactSet, ConsumedProjectionSourceTruth,
    ProjectMaterializedFacts, ProjectionConsumptionBindingContext,
    ProjectionConsumptionEligibility, ProjectionConsumptionSource,
    ProjectionFactExtractionCounters, ProjectionSourceFamily,
};
use crate::runtime::{
    WorthQueryDesiredAspectValue, WorthQueryRetainedFieldPath, WorthQueryRetainedMaterializedRow,
};

#[test]
fn every_native_scalar_uses_one_foundational_basis_across_query_identity_contexts() {
    let mut mutation_identities = BTreeSet::new();
    let mut retained_identities = BTreeSet::new();
    let mut projection_identities = BTreeSet::new();

    for value in scalar_samples() {
        let canonical = prepare_aspect_value_identity_basis(&value)
            .as_str()
            .to_string();
        let (mutation, retained, projection) = scalar_context_identities(value);
        assert!(mutation.contains(&canonical));
        assert!(retained.contains(&canonical));
        assert_ne!(mutation, retained, "artifact domains must remain separated");
        mutation_identities.insert(mutation);
        retained_identities.insert(retained);
        projection_identities.insert(projection);
    }

    assert_eq!(mutation_identities.len(), 24);
    assert_eq!(retained_identities.len(), 24);
    assert_eq!(projection_identities.len(), 24);
}

#[test]
fn struct_identity_basis_is_preserved_without_flattening_and_remains_domain_separated() {
    let value = StructAspectValue::new([
        (
            FieldKey::new("label").unwrap(),
            AspectValue::String("native;field=label".into()),
        ),
        (FieldKey::new("count").unwrap(), AspectValue::UInt32(7)),
    ])
    .unwrap();
    let canonical = prepare_struct_aspect_value_identity_basis(&value)
        .as_str()
        .to_string();
    let (mutation, retained, projection) = struct_context_identities(value.clone());

    assert!(mutation.contains(&canonical));
    assert!(retained.contains(&canonical));
    assert_ne!(mutation, retained);
    assert!(!projection.is_empty());
}

#[test]
fn width_reference_and_interning_distinctions_survive_every_query_identity_context() {
    let pairs = [
        (AspectValue::Int32(7), AspectValue::Int64(7)),
        (AspectValue::UInt32(7), AspectValue::Int32(7)),
        (
            AspectValue::ContentRef(ContentRefId([7; 32])),
            AspectValue::UInt64(7),
        ),
        (
            AspectValue::String(InternedString::Raw("7".into())),
            AspectValue::String(InternedString::Symbol(Symbol(7))),
        ),
    ];
    for (left, right) in pairs {
        let left = scalar_context_identities(left);
        let right = scalar_context_identities(right);
        assert_ne!(left.0, right.0);
        assert_ne!(left.1, right.1);
        assert_ne!(left.2, right.2);
    }
}

fn scalar_context_identities(value: AspectValue) -> (String, String, String) {
    let mutation =
        WorthQueryDesiredAspectValue::set_native(ContractValidationInput::Scalar(value.clone()))
            .terminal_digest_material();
    let retained_path = retained_path();
    let retained = WorthQueryRetainedMaterializedRow::from_native_values(
        BTreeMap::from([(retained_path, value.clone())]),
        BTreeMap::new(),
    )
    .unwrap()
    .terminal_digest_parts()
    .remove(0);
    let fact = ConsumedFieldValueFact::new(&contract(), "row:test", projection_path(), value);
    let projection = fact_set(fact).fact_set_digest().to_string();
    (mutation, retained, projection)
}

fn struct_context_identities(value: StructAspectValue) -> (String, String, String) {
    let mutation =
        WorthQueryDesiredAspectValue::set_native(ContractValidationInput::Struct(value.clone()))
            .terminal_digest_material();
    let retained = WorthQueryRetainedMaterializedRow::from_native_values(
        BTreeMap::new(),
        BTreeMap::from([(retained_path(), value.clone())]),
    )
    .unwrap()
    .terminal_digest_parts()
    .remove(0);
    let fact = ConsumedFieldValueFact::new_native(
        &contract(),
        "row:test",
        projection_path(),
        ConsumedNativeValue::struct_value(value),
    );
    let projection = fact_set(fact).fact_set_digest().to_string();
    (mutation, retained, projection)
}

fn fact_set(fact: ConsumedFieldValueFact) -> ConsumedProjectionFactSet {
    let contract = contract();
    ConsumedProjectionFactSet::new(
        ConsumedProjectionContractProvenance::from_contract(&contract),
        ConsumedProjectionSourceTruth::from_contract(
            &contract,
            crate::projection_consumption::ConsumedNativeLayoutProof::from_contract(&contract, 1),
        ),
        ProjectionFactExtractionCounters::new(1, 1, 1, 1, 0),
        ConsumedProjectionFactInventory {
            entity_identities: Vec::new(),
            view_local_identities: Vec::new(),
            memberships: Vec::new(),
            display_fields: vec![fact],
            derived_fields: Vec::new(),
            target_identities: Vec::new(),
            source_references: Vec::new(),
            effect_continuity_facts: Vec::new(),
            relation_endpoints: Vec::new(),
        },
    )
    .expect("fixture digest must fit")
}

fn contract() -> crate::projection_consumption::MaterializedProjectionContract {
    let source = ProjectionConsumptionSource::test_only(
        ProjectionSourceFamily::QueryReadReceipt,
        Some("query:test"),
        Some("basis:test"),
        Some("result:test"),
        Some("result-shape:test"),
        "read-graph:test",
    );
    let binding = ProjectionConsumptionBindingContext::test_only(
        "result-shape:test",
        "authorized-projection:test",
        crate::projection_consumption::test_authorized_field_paths(&["native.value"]),
    );
    let declaration = declare_projection_consumption(
        source,
        binding,
        ProjectMaterializedFacts::declare().display_field_path(projection_path()),
    )
    .unwrap();
    let ProjectionConsumptionEligibility::Admitted(admitted) =
        evaluate_projection_consumption_eligibility(&declaration)
    else {
        panic!("native identity fixture must admit")
    };
    bind_materialized_projection_contract(&admitted).expect("fixture digest must fit")
}

fn retained_path() -> WorthQueryRetainedFieldPath {
    WorthQueryRetainedFieldPath::from_canonical_field_path(
        worth_foundational::facade::CanonicalFieldPath::new([
            FieldKey::new("native").unwrap(),
            FieldKey::new("value").unwrap(),
        ])
        .unwrap(),
    )
}

fn projection_path() -> crate::projection_consumption::ProjectionFactFieldPath {
    projection_fact_field_path_from_segments([
        FieldKey::new("native").unwrap(),
        FieldKey::new("value").unwrap(),
    ])
}

fn scalar_samples() -> Vec<AspectValue> {
    vec![
        AspectValue::Null,
        AspectValue::Bool(true),
        AspectValue::Int8(-7),
        AspectValue::Int16(-320),
        AspectValue::Int32(-32_000),
        AspectValue::Int64(-12),
        AspectValue::UInt8(7),
        AspectValue::UInt16(320),
        AspectValue::UInt32(32_000),
        AspectValue::UInt64(12),
        AspectValue::Float32(CanonicalF32::from_f32(1.5)),
        AspectValue::Float64(CanonicalF64::from_f64(2.5)),
        AspectValue::Decimal(CanonicalDecimal::new("12.50")),
        AspectValue::BigInt(CanonicalBigInt::new("-12345678901234567890")),
        AspectValue::Rational(
            CanonicalRational::new(CanonicalBigInt::new("22"), CanonicalBigInt::new("7")).unwrap(),
        ),
        AspectValue::String(InternedString::Raw("alpha".into())),
        AspectValue::String(InternedString::Symbol(Symbol(17))),
        AspectValue::Uuid([7; 16]),
        AspectValue::Date(CanonicalDate {
            days_from_unix_epoch: 20_000,
        }),
        AspectValue::Time(CanonicalTime::new(1_000).unwrap()),
        AspectValue::Timestamp(CanonicalTimestamp {
            micros_since_unix_epoch: 123_456,
        }),
        AspectValue::TimestampTz(CanonicalTimestampTz {
            utc_micros_since_unix_epoch: 123_456,
            offset_minutes: -360,
        }),
        AspectValue::EntityRef(EntityId::new(PartitionId(9), 10, 11)),
        AspectValue::ContentRef(ContentRefId([
            42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ])),
    ]
}
