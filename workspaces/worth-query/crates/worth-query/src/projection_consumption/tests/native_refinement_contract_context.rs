use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, AspectValue, AspectValuePosture, FieldDeclaration, FieldKey, FieldRequirement,
    ScalarAspectType, StructAspectShape, StructAspectValue,
};

use crate::projection_consumption::{
    bind_materialized_projection_contract, declare_projection_consumption,
    evaluate_projection_consumption_eligibility, ConsumedFieldValueFact, ConsumedNativeValue,
    DeclaredNativeAspectContractBasis, DeclaredNativeFactContract, MaterializedProjectionContract,
    ProjectMaterializedFacts, ProjectionConsumptionBindingContext,
    ProjectionConsumptionEligibility, ProjectionConsumptionSource, ProjectionSourceFamily,
};

const CONTRACT_IDENTITY: AspectIdentity = AspectIdentity(0x9150_00c0);
const CONTRACT_REVISION: AspectContractRevision = AspectContractRevision(7);

#[test]
fn wrong_scalar_family_retains_declared_foundational_contract_context() {
    let (materialized, declaration) = native_string_field_contract(AbsenceLaw::Optional);
    let fact = declared_fact(
        &materialized,
        &declaration,
        ConsumedNativeValue::scalar(AspectValue::UInt64(17)),
    );

    let denial = fact.as_interned_string().unwrap_err();

    assert_eq!(
        denial.expected(),
        AspectValuePosture::Scalar(ScalarAspectType::String)
    );
    assert_eq!(
        denial.actual(),
        AspectValuePosture::Scalar(ScalarAspectType::UInt64)
    );
    assert_contract_context(&denial);
}

#[test]
fn struct_instead_of_scalar_retains_declared_foundational_contract_context() {
    let (materialized, declaration) = native_string_field_contract(AbsenceLaw::Optional);
    let structured = StructAspectValue::new([(
        FieldKey::new("nested").unwrap(),
        AspectValue::String("unexpected".into()),
    )])
    .unwrap();
    let fact = declared_fact(
        &materialized,
        &declaration,
        ConsumedNativeValue::struct_value(structured),
    );

    let denial = fact.as_interned_string().unwrap_err();

    assert_eq!(denial.actual(), AspectValuePosture::Struct);
    assert_contract_context(&denial);
}

#[test]
fn absent_instead_of_null_retains_absence_and_foundational_contract_context() {
    let (materialized, declaration) = native_string_field_contract(AbsenceLaw::Optional);
    let fact = declared_fact(
        &materialized,
        &declaration,
        ConsumedNativeValue::absent(AbsenceLaw::Optional),
    );

    let denial = fact.as_null().unwrap_err();

    assert_eq!(
        denial.expected(),
        AspectValuePosture::Scalar(ScalarAspectType::Null)
    );
    assert_eq!(
        denial.actual(),
        AspectValuePosture::Absent(AbsenceLaw::Optional)
    );
    assert_contract_context(&denial);
}

fn declared_fact(
    materialized: &MaterializedProjectionContract,
    declaration: &DeclaredNativeFactContract,
    value: ConsumedNativeValue,
) -> ConsumedFieldValueFact {
    ConsumedFieldValueFact::new_declared_native(
        materialized,
        "row:native-refinement",
        declaration,
        value,
    )
}

fn native_string_field_contract(
    absence: AbsenceLaw,
) -> (MaterializedProjectionContract, DeclaredNativeFactContract) {
    let aspect_key = AspectKey::new("profile.native").unwrap();
    let field_key = FieldKey::new("label").unwrap();
    let contract = AspectContract::struct_aspect(
        aspect_key.clone(),
        CONTRACT_IDENTITY,
        CONTRACT_REVISION,
        StructAspectShape::new([FieldDeclaration::new(
            field_key.clone(),
            ScalarAspectType::String,
            field_requirement(absence),
            absence,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .unwrap()])
        .unwrap(),
    );
    let declaration = DeclaredNativeFactContract::field(
        DeclaredNativeAspectContractBasis::new(contract),
        &[],
        true,
        &field_key,
    )
    .unwrap();
    let requested = ProjectMaterializedFacts::declare()
        .display_native(declaration.clone())
        .unwrap();
    let source = ProjectionConsumptionSource::test_only(
        ProjectionSourceFamily::QueryReadReceipt,
        Some("query:test"),
        Some("basis:native-refinement"),
        Some("result:native-refinement"),
        Some("result-shape:test"),
        "source:native-refinement",
    );
    let binding = ProjectionConsumptionBindingContext::test_only(
        "result-shape:test",
        "projection:native-refinement",
        vec![
            crate::authorized_projection::AuthorizedProjectionFieldPath::from_native_keys(
                aspect_key, field_key,
            ),
        ],
    );
    let admitted = match evaluate_projection_consumption_eligibility(
        &declare_projection_consumption(source, binding, requested).unwrap(),
    ) {
        ProjectionConsumptionEligibility::Admitted(admitted) => admitted,
        outcome => panic!("native refinement fixture did not admit: {outcome:?}"),
    };
    (
        bind_materialized_projection_contract(&admitted).expect("fixture digest must fit"),
        declaration,
    )
}

fn field_requirement(absence: AbsenceLaw) -> FieldRequirement {
    match absence {
        AbsenceLaw::Required => FieldRequirement::Required,
        AbsenceLaw::Optional => FieldRequirement::Optional,
        AbsenceLaw::Defaulted => FieldRequirement::Defaulted,
    }
}

fn assert_contract_context(denial: &crate::projection_consumption::ConsumedNativeRefinementDenial) {
    assert_eq!(
        denial.contract_key(),
        AspectKey::new("profile.native").as_ref()
    );
    assert_eq!(denial.contract_identity(), Some(CONTRACT_IDENTITY));
    assert_eq!(denial.contract_revision(), Some(CONTRACT_REVISION));
    assert_eq!(
        denial.field_path().native_aspect_key(),
        AspectKey::new("profile.native").as_ref()
    );
    assert_eq!(
        denial.field_path().native_field_key(),
        FieldKey::new("label").as_ref()
    );
    assert_eq!(
        denial.source_family(),
        ProjectionSourceFamily::QueryReadReceipt
    );
    assert_eq!(denial.source_identity(), "source:native-refinement");
    assert_eq!(denial.source_row_identity(), "row:native-refinement");
    assert!(!denial.projection_authority().is_empty());
}
