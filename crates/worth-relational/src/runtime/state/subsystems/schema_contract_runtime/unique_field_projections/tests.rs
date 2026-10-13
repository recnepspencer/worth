//! Installed projection proofs are shared; record values remain borrowed.
use super::*;
use crate::facade::runtime::RelationalRuntimeApi;
use crate::runtime::RuntimeSubsystem;
use crate::tests::support::{entity_field_aspect, AspectSchemaFixture};
use crate::validation::data::InvariantCatalog;
use std::sync::Arc;
use worth_foundational::facade::{
    admit_authoritative_record_aspect_state, validate_aspect_value, InternedString,
};
use worth_proof::TransitionOutcome;

#[test]
fn installed_unique_projection_reuses_long_key_proofs_within_its_schema_snapshot() {
    let aspect = AspectKey::new("aspect".repeat(16_384)).unwrap();
    let field = FieldKey::new("field".repeat(16_384)).unwrap();
    let locator =
        crate::transactions::data::planned_single_field_locator(aspect.clone(), field.clone());
    let build = |declared_field: FieldKey| {
        RelationalRuntimeApi::builder()
            .schema_registry(
                AspectSchemaFixture {
                    entity_aspects: vec![entity_field_aspect(aspect.clone(), declared_field)],
                    ..AspectSchemaFixture::default()
                }
                .build_registry(),
            )
            .invariant_catalog(InvariantCatalog {
                registrations: vec![InvariantRegistration::commit_boundary_blocking(
                    InvariantRule::unique_entity_aspect_field(aspect.clone(), field.clone()),
                )],
            })
            .build()
    };
    let runtime = build(field.clone());
    let installed = runtime.schema_contract_runtime.clone();
    let catalog = &installed.unique_field_projections;
    let proof = catalog
        .entries
        .get(&KindId(1))
        .unwrap()
        .get(&locator)
        .unwrap();
    assert!(proof
        .projection_scope()
        .requirements()
        .iter()
        .all(|requirement| requirement.mask_basis().is_some()));
    let contract = installed
        .aspect_contract_plans
        .entity_plans
        .get(&KindId(1))
        .unwrap()
        .contract_for(&aspect)
        .unwrap();
    let TransitionOutcome::Success(value) = validate_aspect_value(
        &contract,
        AspectValue::String(InternedString::Raw("small selected value".to_owned())).into(),
    ) else {
        panic!("fixture value must validate against its installed contract")
    };
    let TransitionOutcome::Success(state) = admit_authoritative_record_aspect_state([value]) else {
        panic!("fixture state must have authentic admission")
    };
    let (state, _, _) = state.into_parts().into_parts();
    let ContractValidatedAspectValueView::Scalar(expected) = state.get(&aspect).unwrap().view()
    else {
        panic!("fixture uses a scalar aspect")
    };
    for _ in 0..16 {
        let actual = catalog.value(KindId(1), &locator, || Some(&state)).unwrap();
        assert!(
            std::ptr::eq(actual, expected),
            "selection borrows the authoritative value"
        );
        assert!(
            std::ptr::eq(
                proof,
                catalog
                    .entries
                    .get(&KindId(1))
                    .unwrap()
                    .get(&locator)
                    .unwrap()
            ),
            "the same admitted metadata serves each record read"
        );
    }
    let fork = installed.fork();
    assert!(
        Arc::ptr_eq(catalog, &fork.unique_field_projections),
        "forking cannot deep-copy the installed projection metadata"
    );
    let changed = build(FieldKey::new("different").unwrap());
    assert!(changed
        .schema_contract_runtime
        .unique_field_projections
        .value(KindId(1), &locator, || panic!(
            "an unbound field cannot contact record state"
        ))
        .is_none());
    assert!(
        std::ptr::eq(
            catalog.value(KindId(1), &locator, || Some(&state)).unwrap(),
            expected
        ),
        "another schema snapshot cannot replace the earlier admitted proof"
    );
    assert!(catalog
        .value(KindId(9), &locator, || panic!(
            "an unrelated kind cannot contact record state"
        ))
        .is_none());
}
