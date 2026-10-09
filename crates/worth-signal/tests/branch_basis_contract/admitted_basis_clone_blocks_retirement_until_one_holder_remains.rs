//! Admitted basis clone blocks retirement until one holder remains.

use super::*;

#[test]
fn admitted_basis_clone_blocks_retirement_until_one_holder_remains() {
    let mut runtime = runtime();
    let main_basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("owner observation should succeed");
    let (branch, basis) = runtime
        .fork_signal_branch("shared-admission", &main_basis)
        .expect("owner fork should succeed")
        .into_parts();
    let shared = basis.clone();

    let denied = runtime.plan_signal_branch_retirement(
        branch.clone(),
        basis,
        SignalBranchRetirementReason::Superseded,
    );
    assert!(matches!(
        denied,
        TransitionOutcome::Denied(SignalBranchRetirementDenial::SharedAdmittedBasis {
            shared_holders: 2,
            ..
        })
    ));

    let plan = runtime.plan_signal_branch_retirement(
        branch,
        shared,
        SignalBranchRetirementReason::Superseded,
    );
    let plan = match plan {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("one remaining admitted holder should become linear: {other:?}"),
    };
    assert!(matches!(
        runtime.retire_signal_branch(plan),
        TransitionOutcome::Success(_)
    ));
}
