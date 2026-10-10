use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, AspectValue, CanonicalBigInt, CanonicalDate, CanonicalDecimal, CanonicalF32,
    CanonicalF64, CanonicalFieldPath, CanonicalRational, CanonicalTime, CanonicalTimestamp,
    CanonicalTimestampTz, ContentRefId, FieldDeclaration, FieldKey, FieldRequirement,
    InternedString, PartitionId, ScalarAspectType, StructAspectShape, StructAspectValue, Symbol,
};
use worth_query::facade::consumer_kit::{in_memory_test_runtime, WorthQueryTestBackendSchema};
use worth_query::facade::foundation::AspectFieldKey;
use worth_query::facade::mutation::{
    authoritative, declare, WorthQueryAspectTouch, WorthQueryMutationOutcome,
};
use worth_query::facade::runtime::WorthQueryUnrefinedLiveShape;

#[test]
fn ordinary_mutation_roundtrips_every_foundational_scalar_family() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let values = scalar_samples();
    let contracts = values
        .iter()
        .enumerate()
        .map(|(index, value)| scalar_contract(index, value.value_family()))
        .collect::<Vec<_>>();
    let mut schema = native_matrix_schema();
    for contract in &contracts {
        schema = schema
            .aspect_contract(contract.clone())
            .unwrap()
            .aspect(contract.key().as_str(), contract.key().as_str())
            .unwrap();
    }
    let mut workspace = in_memory_test_runtime()
        .with_schema(schema)
        .workspace(request_execution, "phase-26-native-scalar-matrix")
        .unwrap();
    let declaration = declare(|builder| {
        let builder = values.iter().enumerate().fold(
            builder.set_aspect(touch("identity", Some("id")), "matrix"),
            |builder, (index, value)| {
                builder.set_aspect(touch(&format!("native_{index}"), None), value.clone())
            },
        );
        builder.build_insert("NativeRecord")
    })
    .unwrap();

    let context = authoritative(request_execution, &workspace).unwrap();
    let outcome = declaration
        .using(context)
        .run(request_execution, &mut workspace);
    assert!(outcome.completed().is_some());
    let rows = read_rows(
        request_execution,
        &mut workspace,
        "phase-26-native-scalar-read",
    );
    assert_eq!(rows.len(), 1);
    for (index, expected) in values.iter().enumerate() {
        let key = AspectKey::new(format!("native_{index}")).unwrap();
        assert_eq!(rows[0].aspect_value(&key), Some(expected), "family {index}");
    }
}

#[test]
fn ordinary_mutation_preserves_struct_set_clear_null_and_denial_boundaries() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let profile = profile_contract();
    let schema = native_matrix_schema()
        .aspect_contract(profile.clone())
        .unwrap()
        .aspect("profile.title", "profile.title")
        .unwrap()
        .aspect("profile.null_value", "profile.null_value")
        .unwrap();
    let mut workspace = in_memory_test_runtime()
        .with_schema(schema)
        .workspace(request_execution, "phase-26-native-struct")
        .unwrap();
    let initial = StructAspectValue::new([
        (FieldKey::new("title").unwrap(), text("Draft")),
        (FieldKey::new("null_value").unwrap(), AspectValue::Null),
    ])
    .unwrap();
    let insert = declare(|builder| {
        builder
            .set_aspect(touch("identity", Some("id")), "struct")
            .set_aspect(touch("profile", None), initial.clone())
            .build_insert("NativeRecord")
    })
    .unwrap()
    .using(authoritative(request_execution, &workspace).unwrap())
    .run(request_execution, &mut workspace);
    let entity = insert
        .completed()
        .unwrap()
        .receipt()
        .target_entity_identity()
        .unwrap()
        .clone();
    assert_eq!(
        read_rows(
            request_execution,
            &mut workspace,
            "phase-26-native-struct-initial"
        )[0]
        .struct_aspect_value(profile.key()),
        Some(&initial)
    );

    let update = declare(|builder| {
        builder
            .clear(touch("profile", Some("title")))
            .set_aspect(touch("profile", Some("null_value")), AspectValue::Null)
            .build_update(entity.clone())
    })
    .unwrap()
    .using(authoritative(request_execution, &workspace).unwrap())
    .run(request_execution, &mut workspace);
    assert!(update.completed().is_some());
    let after_clear = read_rows(
        request_execution,
        &mut workspace,
        "phase-26-native-struct-cleared",
    );
    let profile_value = after_clear[0].struct_aspect_value(profile.key()).unwrap();
    assert_eq!(profile_value.get(&FieldKey::new("title").unwrap()), None);
    assert_eq!(
        profile_value.get(&FieldKey::new("null_value").unwrap()),
        Some(&AspectValue::Null)
    );

    let denied = declare(|builder| {
        builder
            .set_aspect(touch("profile", Some("title")), AspectValue::Bool(true))
            .build_update(entity)
    })
    .unwrap()
    .using(authoritative(request_execution, &workspace).unwrap())
    .run(request_execution, &mut workspace);
    let WorthQueryMutationOutcome::Stopped(stop) = denied else {
        panic!("wrong-family mutation must stop before lower-runtime execution");
    };
    assert_eq!(stop.counters().lower_runtime_execution_attempt_count(), 0);
    assert_eq!(stop.counters().lower_runtime_execution_completed_count(), 0);
    let after_denial = read_rows(
        request_execution,
        &mut workspace,
        "phase-26-native-struct-denied",
    );
    assert_eq!(
        after_denial[0].struct_aspect_value(profile.key()),
        Some(profile_value)
    );
}

#[test]
fn invalid_native_values_deny_before_lower_runtime_execution_without_residue() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let decimal = scalar_contract(700, ScalarAspectType::Decimal);
    let entity_ref = scalar_contract(701, ScalarAspectType::EntityRef);
    let profile = profile_contract();
    let schema = native_matrix_schema()
        .aspect_contract(decimal.clone())
        .unwrap()
        .aspect(decimal.key().as_str(), decimal.key().as_str())
        .unwrap()
        .aspect_contract(entity_ref.clone())
        .unwrap()
        .aspect(entity_ref.key().as_str(), entity_ref.key().as_str())
        .unwrap()
        .aspect_contract(profile.clone())
        .unwrap()
        .aspect("profile.title", "profile.title")
        .unwrap()
        .aspect("profile.null_value", "profile.null_value")
        .unwrap();
    let mut workspace = in_memory_test_runtime()
        .with_schema(schema)
        .workspace(request_execution, "phase-26-native-denials")
        .unwrap();
    let insert = declare(|builder| {
        builder
            .set_aspect(touch("identity", Some("id")), "denial-subject")
            .build_insert("NativeRecord")
    })
    .unwrap()
    .using(authoritative(request_execution, &workspace).unwrap())
    .run(request_execution, &mut workspace);
    let entity = insert
        .completed()
        .unwrap()
        .receipt()
        .target_entity_identity()
        .unwrap()
        .clone();

    for (touch, value) in [
        (
            touch(decimal.key().as_str(), None),
            AspectValue::Decimal(CanonicalDecimal::new("not-a-decimal")),
        ),
        (
            touch(entity_ref.key().as_str(), None),
            AspectValue::ContentRef(ContentRefId(902)),
        ),
        (
            touch(profile.key().as_str(), Some("undeclared")),
            text("not-declared"),
        ),
    ] {
        let denied = declare(|builder| {
            builder
                .set_aspect(touch, value)
                .build_update(entity.clone())
        })
        .unwrap()
        .using(authoritative(request_execution, &workspace).unwrap())
        .run(request_execution, &mut workspace);
        assert_pre_execution_denial(denied);
    }

    let rows = read_rows(
        request_execution,
        &mut workspace,
        "phase-26-native-denials-readback",
    );
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].aspect_value(decimal.key()), None);
    assert_eq!(rows[0].aspect_value(entity_ref.key()), None);
    assert_eq!(rows[0].struct_aspect_value(profile.key()), None);
}

fn assert_pre_execution_denial(outcome: WorthQueryMutationOutcome) {
    let WorthQueryMutationOutcome::Stopped(stop) = outcome else {
        panic!("invalid native mutation must stop before lower-runtime execution");
    };
    assert_eq!(stop.counters().lower_runtime_execution_attempt_count(), 0);
    assert_eq!(stop.counters().lower_runtime_execution_completed_count(), 0);
}

fn read_rows(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    name: &str,
) -> Vec<worth_query::facade::foundation::WorthQueryEntity> {
    let view = workspace
        .live_view::<WorthQueryUnrefinedLiveShape>(request_execution, name, |view| {
            view.from("NativeRecord")
                .select([AspectFieldKey::from_authoring_parts("identity", "id").unwrap()])
        })
        .unwrap();
    workspace.read(request_execution, &view)
}

fn native_matrix_schema() -> WorthQueryTestBackendSchema {
    WorthQueryTestBackendSchema::single_collection("NativeRecord")
        .aspect_contract(struct_contract(
            "identity",
            0x2610,
            [("id", ScalarAspectType::String)],
        ))
        .unwrap()
        .aspect("identity.id", "identity.id")
        .unwrap()
}

fn scalar_contract(index: usize, family: ScalarAspectType) -> AspectContract {
    AspectContract::scalar(
        AspectKey::new(format!("native_{index}")).unwrap(),
        AspectIdentity(0x2620 + index as u64),
        AspectContractRevision(1),
        family,
    )
}

fn profile_contract() -> AspectContract {
    struct_contract(
        "profile",
        0x2690,
        [
            ("title", ScalarAspectType::String),
            ("null_value", ScalarAspectType::Null),
        ],
    )
}

fn struct_contract<const N: usize>(
    aspect: &str,
    identity: u64,
    fields: [(&str, ScalarAspectType); N],
) -> AspectContract {
    let fields = fields.map(|(field, family)| {
        FieldDeclaration::new(
            FieldKey::new(field).unwrap(),
            family,
            FieldRequirement::Optional,
            AbsenceLaw::Optional,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .unwrap()
    });
    AspectContract::struct_aspect(
        AspectKey::new(aspect).unwrap(),
        AspectIdentity(identity),
        AspectContractRevision(1),
        StructAspectShape::new(fields).unwrap(),
    )
}

fn touch(aspect: &str, field: Option<&str>) -> WorthQueryAspectTouch {
    let aspect = AspectKey::new(aspect).unwrap();
    match field {
        Some(field) => WorthQueryAspectTouch::aspect_field_path(
            aspect,
            CanonicalFieldPath::single(FieldKey::new(field).unwrap()),
        ),
        None => WorthQueryAspectTouch::whole_aspect(aspect),
    }
}

fn text(value: &str) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.to_string()))
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
        AspectValue::String(InternedString::Raw("alpha".to_string())),
        AspectValue::String(InternedString::Symbol(Symbol(17))),
        AspectValue::Bytes(ContentRefId(41)),
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
        AspectValue::EntityRef(worth_foundational::facade::EntityId::new(
            PartitionId(9),
            10,
            11,
        )),
        AspectValue::ContentRef(ContentRefId(42)),
    ]
}
