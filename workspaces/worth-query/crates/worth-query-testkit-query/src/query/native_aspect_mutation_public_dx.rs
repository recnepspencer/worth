pub use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, AspectValue, CanonicalBigInt, CanonicalDate, CanonicalDecimal, CanonicalF32,
    CanonicalF64, CanonicalFieldPath, CanonicalRational, CanonicalTime, CanonicalTimestamp,
    CanonicalTimestampTz, ContentRefId, FieldDeclaration, FieldKey, FieldRequirement,
    InternedString, PartitionId, ScalarAspectType, StructAspectShape, StructAspectValue, Symbol,
};
pub use worth_query::facade::consumer_kit::{in_memory_test_runtime, WorthQueryTestBackendSchema};
pub use worth_query::facade::foundation::AspectFieldKey;
pub use worth_query::facade::mutation::{
    authoritative, declare, WorthQueryAspectTouch, WorthQueryMutationOutcome,
};
pub use worth_query::facade::runtime::WorthQueryUnrefinedLiveShape;

pub fn assert_pre_execution_denial(outcome: WorthQueryMutationOutcome) {
    let WorthQueryMutationOutcome::Stopped(stop) = outcome else {
        panic!("invalid native mutation must stop before lower-runtime execution");
    };
    assert_eq!(stop.counters().lower_runtime_execution_attempt_count(), 0);
    assert_eq!(stop.counters().lower_runtime_execution_completed_count(), 0);
}

pub fn read_rows(
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

pub fn native_matrix_schema() -> WorthQueryTestBackendSchema {
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

pub fn scalar_contract(index: usize, family: ScalarAspectType) -> AspectContract {
    AspectContract::scalar(
        AspectKey::new(format!("native_{index}")).unwrap(),
        AspectIdentity(0x2620 + index as u64),
        AspectContractRevision(1),
        family,
    )
}

pub fn profile_contract() -> AspectContract {
    struct_contract(
        "profile",
        0x2690,
        [
            ("title", ScalarAspectType::String),
            ("null_value", ScalarAspectType::Null),
        ],
    )
}

pub fn struct_contract<const N: usize>(
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

pub fn touch(aspect: &str, field: Option<&str>) -> WorthQueryAspectTouch {
    let aspect = AspectKey::new(aspect).unwrap();
    match field {
        Some(field) => WorthQueryAspectTouch::aspect_field_path(
            aspect,
            CanonicalFieldPath::single(FieldKey::new(field).unwrap()),
        ),
        None => WorthQueryAspectTouch::whole_aspect(aspect),
    }
}

pub fn text(value: &str) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.to_string()))
}

pub fn scalar_samples() -> Vec<AspectValue> {
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
