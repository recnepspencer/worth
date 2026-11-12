//! Actual original-request lowerer/readmitter and concrete owner controls.
use super::super::record_aspect_patch;
use super::*;
use crate::authority::mutation::{canonical_deltas, outcomes};
use crate::identity::data::{EntityId, KindId, PartitionId};
use crate::schema::data::{
    AspectBinding, AspectContractPlanCatalog, AspectContractPlanRevision,
    LoweredAspectContractBinding,
};
use crate::transactions::data::{planned_single_field_locator, RecordAspectPatchTarget};
use std::collections::BTreeMap;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::*;
mod legacy;
fn host(budget: &SerialMemoryBudget) -> SerialRequest {
    SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None)
}
mod fixture;
use fixture::*;
#[test]
fn controlled_field_lowering_preserves_original_winners_order_and_wire() {
    let source = fields();
    let plan = plan(false);
    let budget = SerialMemoryBudget::new(Some(64 * 1024));
    let host = host(&budget);
    let request = ExecutionRequest::serial(&host);
    for purpose in [
        PortablePatchReadmissionPurpose::RecordCreation,
        PortablePatchReadmissionPurpose::RecordMutation,
    ] {
        let actual = lower(
            &request,
            &source,
            purpose,
            Some(&plan),
            KindId(1),
            FieldAuthoringDomain::Entity,
        )
        .unwrap();
        let prior = legacy::lower(
            &source,
            purpose,
            Some(&plan),
            KindId(1),
            FieldAuthoringDomain::Entity,
        )
        .unwrap();
        assert_eq!(
            rmp_serde::to_vec_named(&actual.operations).unwrap(),
            rmp_serde::to_vec_named(prior.operations()).unwrap()
        );
        assert_eq!(
            rmp_serde::to_vec(&actual.operations).unwrap(),
            rmp_serde::to_vec(prior.operations()).unwrap()
        );
        assert_eq!(actual.operations[0].basis().key(), &key("z.scalar"));
    }
    let invalid = AspectFieldPatch::new(BTreeMap::from([
        (
            planned_single_field_locator(key("missing"), field("x")),
            text("first"),
        ),
        (
            AspectFieldLocator::new(
                LocatorAuthority::ReceiptBearing,
                key("z.scalar"),
                CanonicalFieldPath::new([field("x"), field("y")]).unwrap(),
            ),
            text("later"),
        ),
    ]));
    let CandidateDenial::Semantic(actual) = lower(
        &request,
        &invalid,
        PortablePatchReadmissionPurpose::RecordCreation,
        Some(&plan),
        KindId(1),
        FieldAuthoringDomain::Entity,
    )
    .err()
    .unwrap() else {
        panic!("actual semantic refusal")
    };
    assert_eq!(
        actual,
        legacy::lower(
            &invalid,
            PortablePatchReadmissionPurpose::RecordCreation,
            Some(&plan),
            KindId(1),
            FieldAuthoringDomain::Entity
        )
        .unwrap_err()
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
}
#[test]
fn controlled_field_lowering_refuses_before_copy_and_preserves_typed_causes() {
    let source = fields();
    let plan = plan(false);
    let zero = SerialMemoryBudget::new(Some(0));
    let host = host(&zero);
    let request = ExecutionRequest::serial(&host);
    assert!(matches!(
        lower(
            &request,
            &source,
            PortablePatchReadmissionPurpose::RecordCreation,
            Some(&plan),
            KindId(1),
            FieldAuthoringDomain::Entity
        ),
        Err(CandidateDenial::Request(_))
    ));
    assert_eq!(zero.charged_memory_bytes(), 0);
    let cancellation = CancellationSource::new();
    cancellation.cancel();
    let stopped =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    assert!(matches!(
        lower(
            &ExecutionRequest::serial(&stopped),
            &source,
            PortablePatchReadmissionPurpose::RecordCreation,
            Some(&plan),
            KindId(1),
            FieldAuthoringDomain::Entity
        ),
        Err(CandidateDenial::Request(Cause::Cancelled))
    ));
}
#[test]
fn mutation_patch_retains_moved_values_through_real_consumer_and_refunds() {
    let budget = SerialMemoryBudget::new(Some(64 * 1024));
    let host = host(&budget);
    let request = ExecutionRequest::serial(&host);
    let plan = plan(false);
    let patch = record_aspect_patch::readmit_field_authoring(
        &request,
        &fields(),
        PortablePatchReadmissionPurpose::RecordCreation,
        Some(&plan),
        target(),
        FieldAuthoringDomain::Entity,
    )
    .unwrap();
    let bill = budget.charged_memory_bytes();
    assert!(bill > 0);
    let change = outcomes::RecordMutation::EntityCreated {
        entity_id: EntityId::new(PartitionId::main(), 0, 1),
        kind_id: KindId(1),
        authoritative_patch: Some(patch),
    };
    let mut catalog = AspectContractPlanCatalog::empty();
    catalog.entity_plans.insert(KindId(1), plan);
    let delta = canonical_deltas::canonical_delta_for_mutation(&change, &catalog).unwrap();
    assert_eq!(budget.charged_memory_bytes(), bill);
    let outcomes::RecordMutation::EntityCreated {
        authoritative_patch: Some(patch),
        ..
    } = change
    else {
        panic!("actual produced owner")
    };
    let independent =
        canonical_deltas::authoritative_patch_with_delta_supplements(&delta, patch).unwrap();
    assert!(!independent.is_empty());
    assert_eq!(budget.charged_memory_bytes(),0,"the real consumer destroyed its original input; independent projection copies have their separate contract");
    eprintln!(
        "target layouts FoundationPatch={} MutationAspectPatch={} RecordMutation={}",
        std::mem::size_of::<AuthoritativeRecordAspectPatch>(),
        std::mem::size_of::<outcomes::MutationAspectPatch>(),
        std::mem::size_of::<outcomes::RecordMutation>()
    );
}
#[test]
fn field_lowering_and_portable_copy_preserve_surviving_values() {
    let mut spare = String::with_capacity(64 * 1024);
    spare.push_str("ok");
    let source = AspectFieldPatch::new(BTreeMap::from([
        (
            AspectFieldLocator::new(
                LocatorAuthority::Authoritative,
                key("z.scalar"),
                CanonicalFieldPath::single(field("value")),
            ),
            text(&"discarded".repeat(8192)),
        ),
        (
            planned_single_field_locator(key("z.scalar"), field("value")),
            AspectValue::String(InternedString::Raw(spare)),
        ),
    ]));
    let plan = plan(false);
    let budget = SerialMemoryBudget::new(None);
    let host = host(&budget);
    let request = ExecutionRequest::serial(&host);
    let patch = record_aspect_patch::readmit_field_authoring(
        &request,
        &source,
        PortablePatchReadmissionPurpose::RecordCreation,
        Some(&plan),
        target(),
        FieldAuthoringDomain::Entity,
    )
    .unwrap();
    let (_, value) = patch.as_patch().whole_aspect_sets().next().unwrap();
    assert_eq!(
        value.view(),
        ContractValidatedAspectValueView::Scalar(&text("ok"))
    );
    drop(patch);
    let struct_fields = AspectFieldPatch::from_locator(
        planned_single_field_locator(key("a.struct"), field("title")),
        text("ok"),
    );
    let output = record_aspect_patch::readmit_field_authoring(
        &request,
        &struct_fields,
        PortablePatchReadmissionPurpose::RecordCreation,
        Some(&plan),
        target(),
        FieldAuthoringDomain::Entity,
    )
    .unwrap();
    let (_, value) = output.as_patch().whole_aspect_sets().next().unwrap();
    let ContractValidatedAspectValueView::Struct(value) = value.view() else {
        panic!("the actual struct value survives lowering");
    };
    assert_eq!(value.get(&field("title")), Some(&text("ok")));
    drop(output);
    let mut operations = Vec::with_capacity(1024);
    operations.push(PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(&plan.executable_bindings[0].contract),
        value: ContractValidationInput::Scalar(text("ok")),
    });
    let original = PortableRecordAspectPatch::new(operations);
    let copied = FieldCandidate::copy_portable(&request, &original).unwrap();
    assert_eq!(
        rmp_serde::to_vec(&copied.operations).unwrap(),
        rmp_serde::to_vec(original.operations()).unwrap()
    );
}
