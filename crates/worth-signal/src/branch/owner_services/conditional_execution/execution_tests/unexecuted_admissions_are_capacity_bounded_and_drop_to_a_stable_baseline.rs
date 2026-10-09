//! Unexecuted admissions are capacity bounded and drop to a stable baseline.

use super::*;

#[test]
fn unexecuted_admissions_are_capacity_bounded_and_drop_to_a_stable_baseline() {
    let (mut runtime, claimant, [contract, _], source_owner) = runtime_with_contract();
    runtime.set_runtime_policy(
        SignalRuntimePolicy::development().with_conditional_evaluation_budget(
            SignalConditionalEvaluationBudget {
                maximum_retained_slots: 1,
                maximum_retained_bytes: 512 * 1024 * 1024,
                maximum_attempt_visits: 100_000,
            },
        ),
    );
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    runtime.owner_port_slots().unwrap();
    let service = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_owner.authority())
        .unwrap();
    let baseline = service._issuance_basis_custody.retention_usage();

    for ordinal in 0..32 {
        let admission = service
            .admit_evaluation(
                &contract,
                source(&source_owner, &format!("source-{ordinal}"), &mut 0),
            )
            .unwrap();
        let admitted = service._issuance_basis_custody.retention_usage();
        assert_eq!(admitted.0, 1);
        assert!(admitted.1 > baseline.1);
        assert!(matches!(
            service.admit_evaluation(&contract, source(&source_owner, "capacity-denied", &mut 0),),
            Err(Denial::AdmissionCapacityExhausted)
        ));
        assert_eq!(service._issuance_basis_custody.retention_usage(), admitted);
        drop(admission);
        assert_eq!(service._issuance_basis_custody.retention_usage(), baseline);
    }
}
