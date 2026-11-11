//! The actual reconstruction output releases roots discarded by optional apply.
use super::*;

#[test]
fn clearing_reconstructed_optional_aspect_preserves_original_value() {
    let plan = plan(
        aspects()
            .contract()
            .for_key(key())
            .identified_by(AspectIdentity(1))
            .at_revision(AspectContractRevision(1))
            .scalar_with(
                ScalarAspectType::String,
                AspectMaskContract::scalar(),
                AbsenceLaw::Optional,
                AspectEquivalenceBasis::ExactCanonicalValue,
                AspectEvolutionPolicy::ExplicitBreakRequired,
            ),
    );
    let current = state(&plan, text("original"));
    let source = PublishedAuthoritativePatch::new(vec![Op::WholeAspectClear {
        aspect_key: key(),
        aspect_identity: AspectIdentity(1),
        contract_revision: AspectContractRevision(1),
        binding: plan.executable_bindings[0].target.clone(),
    }]);
    let budget = SerialMemoryBudget::new(None);
    let output = run(&budget, Some(&current), &source, &plan, false).unwrap();
    assert!(output.state.is_none());
    assert_eq!(
        current.get(&key()).unwrap().view(),
        ContractValidatedAspectValueView::Scalar(&text("original"))
    );
    drop(output);
}
