use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, FieldDeclaration, FieldKey, FieldRequirement, ScalarAspectType, StructAspectShape,
};

use crate::authorized_projection::AuthorizedProjectionFieldPath;
use crate::projection_consumption::{
    declare_projection_consumption, evaluate_projection_consumption_eligibility,
    DeclaredNativeAspectContractBasis, DeclaredNativeFactContract, ProjectMaterializedFacts,
    ProjectionConsumptionBindingContext, ProjectionConsumptionDenialReason,
    ProjectionConsumptionEligibility, ProjectionConsumptionSource, ProjectionSourceFamily,
};

#[test]
fn whole_struct_is_visible_only_when_every_foundational_field_is_authorized() {
    let (contract, aspect, label, rank) = native_struct_contract();
    let declaration =
        DeclaredNativeFactContract::whole(DeclaredNativeAspectContractBasis::new(contract), true)
            .unwrap();

    let admitted = eligibility(
        declaration.clone(),
        vec![
            AuthorizedProjectionFieldPath::from_native_keys(aspect.clone(), label.clone()),
            AuthorizedProjectionFieldPath::from_native_keys(aspect.clone(), rank.clone()),
        ],
    );
    assert!(matches!(
        admitted,
        ProjectionConsumptionEligibility::Admitted(_)
    ));

    let denied = eligibility(
        declaration,
        vec![AuthorizedProjectionFieldPath::from_native_keys(
            aspect.clone(),
            label,
        )],
    );
    let ProjectionConsumptionEligibility::Denied(denied) = denied else {
        panic!("a partial structured projection must not authorize whole-aspect access");
    };
    assert_eq!(
        denied.reason(),
        &ProjectionConsumptionDenialReason::FactFamilyNotVisible {
            field_key: aspect.as_str().to_string(),
        }
    );
}

fn eligibility(
    declaration: DeclaredNativeFactContract,
    visible: Vec<AuthorizedProjectionFieldPath>,
) -> ProjectionConsumptionEligibility {
    let requested = ProjectMaterializedFacts::declare()
        .display_native(declaration)
        .unwrap();
    let source = ProjectionConsumptionSource::test_only(
        ProjectionSourceFamily::QueryReadReceipt,
        Some("query:native-whole-visibility"),
        Some("basis:native-whole-visibility"),
        Some("result:native-whole-visibility"),
        Some("result-shape:native-whole-visibility"),
        "source:native-whole-visibility",
    );
    let binding = ProjectionConsumptionBindingContext::test_only_with_projection_metadata(
        "result-shape:native-whole-visibility",
        "query:native-whole-visibility",
        "result-shape:native-whole-visibility",
        "projection:native-whole-visibility",
        "narrowed:native-whole-visibility",
        "policy:native-whole-visibility",
        "schema:native-whole-visibility",
        visible,
    );
    let declaration = declare_projection_consumption(source, binding, requested).unwrap();
    evaluate_projection_consumption_eligibility(&declaration).expect("fixture digest must fit")
}

fn native_struct_contract() -> (AspectContract, AspectKey, FieldKey, FieldKey) {
    let aspect = AspectKey::new("native.matrix").unwrap();
    let label = FieldKey::new("label").unwrap();
    let rank = FieldKey::new("rank").unwrap();
    let shape = StructAspectShape::new([
        field(label.clone(), ScalarAspectType::String),
        field(rank.clone(), ScalarAspectType::UInt64),
    ])
    .unwrap();
    (
        AspectContract::struct_aspect(
            aspect.clone(),
            AspectIdentity(0x9150_00d0),
            AspectContractRevision(3),
            shape,
        ),
        aspect,
        label,
        rank,
    )
}

fn field(key: FieldKey, value_type: ScalarAspectType) -> FieldDeclaration {
    FieldDeclaration::new(
        key,
        value_type,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .unwrap()
}
