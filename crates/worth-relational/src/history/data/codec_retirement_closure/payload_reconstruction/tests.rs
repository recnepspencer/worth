//! Finite original serial funding for pure payload reconstruction owners.
use super::*;
use crate::publication::patch::data::{
    PublishedAuthoritativeFieldSet, PublishedAuthoritativePatchOperation as Op,
    PublishedAuthoritativePatchValue as Value,
};
use crate::schema::data::{AspectContractPlanRevision, LoweredAspectContractBinding};
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};

#[test]
fn reconstruction_funds_portable_admission_and_output_until_drop() {
    let plan = scalar_plan();
    let source = scalar_patch(&plan, text("original payload"));
    let budget = SerialMemoryBudget::new(Some(16_000));
    let output = run(&budget, None, &source, &plan, true).unwrap();
    let state = output.state.as_ref().unwrap();
    assert_eq!(
        state.get(&key()).unwrap().view(),
        ContractValidatedAspectValueView::Scalar(&text("original payload"))
    );
    assert_eq!(
        budget.charged_memory_bytes(),
        state.owned_allocation_capacity_bytes() as u64
    );
    assert!(budget.charged_memory_bytes() > 0);
    drop(output);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn reconstruction_refuses_zero_and_midphase_budget_without_changing_prior_state() {
    let plan = scalar_plan();
    let current = state(&plan, text(&"existing".repeat(512)));
    let zero = SerialMemoryBudget::new(Some(0));
    assert!(matches!(
        run(
            &zero,
            Some(&current),
            &scalar_patch(&plan, text("new")),
            &plan,
            false
        ),
        Err(Stop::Request(_))
    ));
    assert_eq!(zero.charged_memory_bytes(), 0);
    // Empty portable/admitted owners fit, but copying the actual current
    // output does not. Removing pre-application admission must fail this case.
    let small = SerialMemoryBudget::new(Some(1024));
    let (result, entered) = run_observed(
        &small,
        Some(&current),
        &PublishedAuthoritativePatch::empty(),
        &plan,
        false,
    );
    assert!(
        entered,
        "the finite scope admitted the reconstruction itself"
    );
    assert!(matches!(result, Err(Stop::Request(_))));
    assert_eq!(small.charged_memory_bytes(), 0);
    assert_eq!(
        current.get(&key()).unwrap().view(),
        ContractValidatedAspectValueView::Scalar(&text(&"existing".repeat(512)))
    );
}

#[test]
fn reconstruction_releases_large_replaced_payload_before_retaining_small_output() {
    let plan = scalar_plan();
    let current = state(&plan, text(&"discarded".repeat(1024)));
    let source = scalar_patch(&plan, text("ok"));
    // Existing caller input is borrowed, never eight newly allocated copies.
    assert!(current.owned_allocation_capacity_bytes() > 4096);
    let budget = SerialMemoryBudget::new(Some(4096));
    let output = run(&budget, Some(&current), &source, &plan, false).unwrap();
    let actual = output.state.as_ref().unwrap();
    assert_eq!(
        actual.get(&key()).unwrap().view(),
        ContractValidatedAspectValueView::Scalar(&text("ok"))
    );
    assert_eq!(
        budget.charged_memory_bytes(),
        actual.owned_allocation_capacity_bytes() as u64
    );
    assert!(budget.charged_memory_bytes() < 512);
    drop(output);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn reconstruction_birth_canonicalizes_unique_fields_and_preserves_duplicate_refusal() {
    let plan = struct_plan();
    let budget = SerialMemoryBudget::new(Some(16_000));
    let source = fields_patch(&plan, &["b", "a"]);
    let output = run(&budget, None, &source, &plan, true).unwrap();
    let ContractValidatedAspectValueView::Struct(actual) =
        output.state.as_ref().unwrap().get(&key()).unwrap().view()
    else {
        panic!("struct birth")
    };
    assert_eq!(
        actual
            .fields()
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );
    assert_eq!(
        actual,
        &StructAspectValue::new([(field("b"), text("b")), (field("a"), text("a"))]).unwrap()
    );
    let changed = PublishedAuthoritativePatch::new(vec![Op::FieldLevelPatch {
        aspect_key: key(),
        aspect_identity: AspectIdentity(1),
        contract_revision: AspectContractRevision(1),
        binding: plan.executable_bindings[0].target.clone(),
        field_sets: vec![PublishedAuthoritativeFieldSet {
            field: field("a"),
            value: text("replacement"),
        }],
        field_clears: vec![field("a"), field("b")],
    }]);
    let updated = run(&budget, output.state.as_ref(), &changed, &plan, false).unwrap();
    drop(output);
    let ContractValidatedAspectValueView::Struct(value) =
        updated.state.as_ref().unwrap().get(&key()).unwrap().view()
    else {
        panic!("struct update")
    };
    assert_eq!(value.get(&field("a")), Some(&text("replacement")));
    assert!(value.get(&field("b")).is_none());
    assert_eq!(
        budget.charged_memory_bytes(),
        updated
            .state
            .as_ref()
            .unwrap()
            .owned_allocation_capacity_bytes() as u64
    );
    drop(updated);
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert!(
        matches!(run(&budget, None, &fields_patch(&plan, &["a", "a"]), &plan, true), Err(Stop::Payload(detail)) if detail == "native birth repeats a struct field")
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
}

fn run(
    budget: &SerialMemoryBudget,
    current: Option<&AuthoritativeRecordAspectState>,
    patch: &PublishedAuthoritativePatch,
    plan: &LoweredAspectContractPlan,
    creation: bool,
) -> Result<ReconstructedPayload, Stop> {
    run_observed(budget, current, patch, plan, creation).0
}
fn run_observed(
    budget: &SerialMemoryBudget,
    current: Option<&AuthoritativeRecordAspectState>,
    patch: &PublishedAuthoritativePatch,
    plan: &LoweredAspectContractPlan,
    creation: bool,
) -> (Result<ReconstructedPayload, Stop>, bool) {
    let mut entered = false;
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    let result = request
        .in_scope(|lease| {
            ExecutionWorkCeiling::new(request.work_ceiling()).run_with_checkpoint(
                lease,
                None,
                |context| {
                    entered = true;
                    apply(current, patch, plan, creation, lease, context)
                },
            )
        })
        .map_err(super::super::request)
        .and_then(|outcome| outcome.map_err(super::super::request))
        .and_then(|(result, _)| result);
    (result, entered)
}
fn key() -> AspectKey {
    AspectKey::new("data").unwrap()
}
fn field(name: &str) -> FieldKey {
    FieldKey::new(name).unwrap()
}
fn text(value: &str) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.into()))
}
fn scalar_plan() -> LoweredAspectContractPlan {
    plan(AspectContract::scalar(
        key(),
        AspectIdentity(1),
        AspectContractRevision(1),
        ScalarAspectType::String,
    ))
}
fn struct_plan() -> LoweredAspectContractPlan {
    let shape = StructAspectShape::new(["a", "b"].map(|name| {
        FieldDeclaration::new(
            field(name),
            ScalarAspectType::String,
            FieldRequirement::Optional,
            AbsenceLaw::Optional,
            AspectEvolutionPolicy::AdditiveFieldsAllowed,
        )
        .unwrap()
    }))
    .unwrap();
    plan(AspectContract::struct_aspect(
        key(),
        AspectIdentity(1),
        AspectContractRevision(1),
        shape,
    ))
}
fn plan(contract: AspectContract) -> LoweredAspectContractPlan {
    LoweredAspectContractPlan {
        kind_id: crate::identity::data::KindId(1),
        plan_revision: AspectContractPlanRevision(1),
        executable_bindings: std::iter::once(LoweredAspectContractBinding {
            contract,
            target: AspectBinding::EntityField {
                field: field("data"),
            },
        })
        .collect(),
    }
}
fn scalar_patch(
    plan: &LoweredAspectContractPlan,
    value: AspectValue,
) -> PublishedAuthoritativePatch {
    PublishedAuthoritativePatch::new(vec![Op::WholeAspectSet {
        aspect_key: key(),
        aspect_identity: AspectIdentity(1),
        contract_revision: AspectContractRevision(1),
        binding: plan.executable_bindings[0].target.clone(),
        value: Value::Scalar(value),
    }])
}
fn fields_patch(plan: &LoweredAspectContractPlan, names: &[&str]) -> PublishedAuthoritativePatch {
    PublishedAuthoritativePatch::new(vec![Op::FieldLevelPatch {
        aspect_key: key(),
        aspect_identity: AspectIdentity(1),
        contract_revision: AspectContractRevision(1),
        binding: plan.executable_bindings[0].target.clone(),
        field_sets: names
            .iter()
            .map(|name| PublishedAuthoritativeFieldSet {
                field: field(name),
                value: text(name),
            })
            .collect(),
        field_clears: vec![],
    }])
}
fn state(plan: &LoweredAspectContractPlan, value: AspectValue) -> AuthoritativeRecordAspectState {
    let worth_proof::TransitionOutcome::Success(value) = validate_aspect_value(
        &plan.executable_bindings[0].contract,
        ContractValidationInput::Scalar(value),
    ) else {
        panic!("pure admitted scalar")
    };
    let worth_proof::TransitionOutcome::Success(state) =
        admit_authoritative_record_aspect_state([value])
    else {
        panic!("pure admitted state")
    };
    state.into_parts().into_parts().0
}
