#[allow(unused_imports)]
use crate::upfront_contract_vocabulary_certification::ordinary_context::*;

use worth_proof::contracts::*;

worth_proof::binding_axes! {
    pub struct RecoveryAxes {
        pub runtime: u64 => Runtime,
        pub branch: &'static str => Branch,
        pub principal: u32 => Principal,
    }
    drift pub enum RecoveryAxesDrift;
}

worth_proof::binding_axis_drift_certification! {
    binding: RecoveryAxes,
    drift: RecoveryAxesDrift,
    base: RecoveryAxes { runtime: 7, branch: "main", principal: 11 },
    twins: {
        runtime => Runtime = 8,
        branch => Branch = "other",
        principal => Principal = 12,
    }
}

#[test]
fn brand_linear_freshness_and_causality_compose_on_the_public_lane() {
    let same_instance = with_brand(|brand| {
        let left = brand.bind(2_u8);
        let right = brand.bind(3_u8);
        left.into_value() + right.into_value()
    });
    assert_eq!(same_instance, 5);

    let resource = LinearResource::<_, RecoveryTerminal, owner::ContractAuthority>::mint(
        17_u64,
        &crate::upfront_contract_vocabulary_certification::ordinary_context::owner::witness(),
    );
    assert_eq!(
        resource.terminate(crate::upfront_contract_vocabulary_certification::ordinary_context::RecoveryTerminal::Resolved).label(),
        "resolved"
    );

    let source = Generations(std::cell::Cell::new(4));
    let current = evaluate_freshness(5_u64, &source, &DeadlinePolicy)
        .expect("generation sampling is infallible");
    assert!(matches!(current, EvaluatedFreshness::Current(_)));
    source.0.set(5);
    let expired = evaluate_freshness(5_u64, &source, &DeadlinePolicy)
        .expect("generation sampling is infallible");
    assert!(matches!(
        expired,
        EvaluatedFreshness::AuthorityRevalidationRequired(_)
    ));

    let undo = Performed::<Undo, owner::ContractAuthority>::record(
        &crate::upfront_contract_vocabulary_certification::ordinary_context::owner::witness(),
        (),
    );
    assert_eq!(redo(prove_inversion::<Undo, Commit, _, _>(&undo)), "redone");
    assert_eq!(publish(prove_derivation(&undo)), "published");
}

#[test]
fn every_terminal_names_itself_and_a_resource_reaches_exactly_one() {
    let disposed = LinearResource::<_, RecoveryTerminal, owner::ContractAuthority>::mint(
        23_u64,
        &crate::upfront_contract_vocabulary_certification::ordinary_context::owner::witness(),
    )
    .terminate(crate::upfront_contract_vocabulary_certification::ordinary_context::RecoveryTerminal::Disposed);

    assert_eq!(disposed.id(), &23);
    assert_eq!(disposed.label(), "disposed");
    // The `Resolved` twin is exercised by the composition test above. A second
    // `terminate` on either resource does not typecheck; see
    // `tests/ui/contracts/compile_fail/linear_resource_cannot_terminate_twice.rs`.
}
