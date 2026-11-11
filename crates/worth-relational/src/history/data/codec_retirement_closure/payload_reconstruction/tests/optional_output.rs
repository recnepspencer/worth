//! The actual reconstruction output releases roots discarded by optional apply.
use super::*;

#[test]
fn clearing_reconstructed_last_aspect_retains_no_state_bill() {
    let plan = scalar_plan();
    let current = state(&plan, text("original"));
    let source = PublishedAuthoritativePatch::new(vec![Op::WholeAspectClear {
        aspect_key: key(),
        aspect_identity: AspectIdentity(1),
        contract_revision: AspectContractRevision(1),
        binding: plan.executable_bindings[0].target.clone(),
    }]);
    let footprint = storage_footprint::quote(Some(&current), &source, &plan, false);
    assert_eq!(footprint.retained, 0);
    let budget = SerialMemoryBudget::new(None);
    let output = run(&budget, Some(&current), &source, &plan, false).unwrap();
    assert!(output.state.is_none());
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(
        current.get(&key()).unwrap().view(),
        ContractValidatedAspectValueView::Scalar(&text("original"))
    );
    drop(output);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
