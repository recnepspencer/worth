use super::*;
use crate::identity::data::KindId;
use crate::schema::data::{
    AspectBinding, AspectContractPlanRevision, LoweredAspectContractBinding,
    LoweredAspectContractPlan,
};
use smallvec::smallvec;
use worth_foundational::facade::{
    AspectContract, AspectContractRevision, AspectIdentity, AspectKey, FieldKey, ScalarAspectType,
};

#[test]
fn projection_contract_admission_resolves_only_the_installed_scalar_field() {
    let aspect_key = AspectKey::new("counter").unwrap();
    let field_key = FieldKey::new("value").unwrap();
    let projection_scope = ProjectionAspectScope::fields(aspect_key.clone(), [field_key.clone()]);
    let plan = LoweredAspectContractPlan {
        kind_id: KindId(1),
        plan_revision: AspectContractPlanRevision(1),
        executable_bindings: smallvec![LoweredAspectContractBinding {
            contract: AspectContract::scalar(
                aspect_key.clone(),
                AspectIdentity(1),
                AspectContractRevision(1),
                ScalarAspectType::Int64,
            ),
            target: AspectBinding::EntityField { field: field_key },
        }],
    };

    assert_eq!(
        plan.executable_bindings[0]
            .contract
            .admits_projection_mask(projection_scope.requirements()[0].mask()),
        Err(worth_foundational::facade::MaskAdmissibilityDenial::FieldMaskRequiresStruct),
        "the foundational Scalar contract does not invent a field-mask interpretation"
    );
    admit_declared_projection_aspects(
        &projection_scope,
        Some(&plan),
        ProjectionRecordDomain::Entity,
        KindId(1),
    )
    .expect("only the installed physical binding lowers the field to its scalar value");
    let wrong = ProjectionAspectScope::fields(aspect_key, [FieldKey::new("counter").unwrap()]);
    let denial = admit_declared_projection_aspects(
        &wrong,
        Some(&plan),
        ProjectionRecordDomain::Entity,
        KindId(1),
    )
    .unwrap_err();
    assert_eq!(
        denial,
        RelationalProjectionContractDenial {
            domain: ProjectionRecordDomain::Entity,
            kind_id: KindId(1),
            requirement: Some(0),
            refusal: ProjectionContractRefusal::MaskRejected,
        }
    );
}

#[test]
fn field_projection_scope_carries_locator_and_canonical_basis() {
    let scope = ProjectionAspectScope::fields(
        AspectKey::new("summary").unwrap(),
        [FieldKey::new("title").unwrap()],
    );
    let requirement = &scope.requirements()[0];

    assert!(requirement.mask_basis().is_some());
    assert_eq!(requirement.locator().aspect_key().as_str(), "summary");
}
