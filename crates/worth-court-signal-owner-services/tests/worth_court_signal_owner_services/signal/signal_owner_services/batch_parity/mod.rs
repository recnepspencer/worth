#[allow(unused_imports)]
use crate::signal_owner_services::ordinary_context::batch_parity::*;
pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_signal::facade::runtime::SignalBranchRetirementDenial;
pub(crate) use {
    worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome,
    worth_signal::facade::branch::SignalBranchRetirementBatchDenial,
};
pub(crate) use {
    worth_signal::facade::SignalBranchRetirementReason, worth_signal::facade::SignalGraph,
    worth_signal::facade::SignalRuntime,
};

#[test]
fn duplicate_batch_denial_is_identical_before_and_after_sealing() {
    let mut runtime = runtime();
    let main_basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("the bootstrap basis is admitted");
    let (child, child_basis) = runtime
        .fork_signal_branch("duplicate-batch-child", &main_basis)
        .expect("the owner creates a batch target")
        .into_parts();

    assert!(matches!(
        runtime.plan_signal_branch_retirement_batch(vec![
            (
                child.clone(),
                child_basis.clone(),
                worth_signal::facade::SignalBranchRetirementReason::Superseded,
            ),
            (
                child.clone(),
                child_basis.clone(),
                worth_signal::facade::SignalBranchRetirementReason::Merged,
            ),
        ]),
        worth_proof::TransitionOutcome::Denied(worth_signal::facade::branch::SignalBranchRetirementBatchDenial::DuplicateBranch {
            branch_id,
        }) if branch_id == child.id
    ));
    assert!(matches!(
        runtime.plan_signal_branch_retirement_batch_releasing_snapshots(vec![
            (
                child.clone(),
                child_basis.clone(),
                Vec::new(),
                worth_signal::facade::SignalBranchRetirementReason::Superseded,
            ),
            (
                child.clone(),
                child_basis.clone(),
                Vec::new(),
                worth_signal::facade::SignalBranchRetirementReason::Merged,
            ),
        ]),
        worth_proof::TransitionOutcome::Denied(worth_signal::facade::branch::SignalBranchRetirementBatchDenial::DuplicateBranch {
            branch_id,
        }) if branch_id == child.id
    ));

    let _services = runtime
        .owner_component_services()
        .expect("the canonical partition seals");

    assert!(matches!(
        runtime.plan_signal_branch_retirement_batch(vec![
            (
                child.clone(),
                child_basis.clone(),
                worth_signal::facade::SignalBranchRetirementReason::Superseded,
            ),
            (
                child.clone(),
                child_basis.clone(),
                worth_signal::facade::SignalBranchRetirementReason::Merged,
            ),
        ]),
        worth_proof::TransitionOutcome::Denied(worth_signal::facade::branch::SignalBranchRetirementBatchDenial::DuplicateBranch {
            branch_id,
        }) if branch_id == child.id
    ));
    assert!(matches!(
        runtime.plan_signal_branch_retirement_batch_releasing_snapshots(vec![
            (
                child.clone(),
                child_basis.clone(),
                Vec::new(),
                worth_signal::facade::SignalBranchRetirementReason::Superseded,
            ),
            (
                child.clone(),
                child_basis.clone(),
                Vec::new(),
                worth_signal::facade::SignalBranchRetirementReason::Merged,
            ),
        ]),
        worth_proof::TransitionOutcome::Denied(worth_signal::facade::branch::SignalBranchRetirementBatchDenial::DuplicateBranch {
            branch_id,
        }) if branch_id == child.id
    ));
    let observed = runtime
        .observe_signal_branch_basis(child.clone())
        .expect("duplicate planning denials leave the target unchanged");
    assert_eq!(observed.observation(), child_basis.observation());
}

#[test]
fn retained_component_batch_denial_is_identical_before_and_after_sealing() {
    let mut runtime = runtime();
    let main_basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("the bootstrap basis is admitted");
    let (child, child_basis) = runtime
        .fork_signal_branch("retained-batch-child", &main_basis)
        .expect("the owner creates a retained batch target")
        .into_parts();
    let lease = runtime
        .retain_signal_component_basis(&child_basis)
        .expect("the target component basis is retained");

    assert!(matches!(
        runtime.plan_signal_branch_retirement_batch(vec![(
            child.clone(),
            child_basis.clone(),
            worth_signal::facade::SignalBranchRetirementReason::Superseded,
        )]),
        worth_proof::TransitionOutcome::Denied(worth_signal::facade::branch::SignalBranchRetirementBatchDenial::Retirement {
            position: 0,
            denial: worth_signal::facade::runtime::SignalBranchRetirementDenial::RetainedComponentBasis {
                branch_id,
                active_leases: 1,
            },
        }) if branch_id == child.id
    ));
    assert!(matches!(
        runtime.plan_signal_branch_retirement_batch_releasing_snapshots(vec![(
            child.clone(),
            child_basis.clone(),
            Vec::new(),
            worth_signal::facade::SignalBranchRetirementReason::Superseded,
        )]),
        worth_proof::TransitionOutcome::Denied(worth_signal::facade::branch::SignalBranchRetirementBatchDenial::Retirement {
            position: 0,
            denial: worth_signal::facade::runtime::SignalBranchRetirementDenial::RetainedComponentBasis {
                branch_id,
                active_leases: 1,
            },
        }) if branch_id == child.id
    ));

    let _services = runtime
        .owner_component_services()
        .expect("the retained target moves into the sealed owner");

    assert!(matches!(
        runtime.plan_signal_branch_retirement_batch(vec![(
            child.clone(),
            child_basis.clone(),
            worth_signal::facade::SignalBranchRetirementReason::Superseded,
        )]),
        worth_proof::TransitionOutcome::Denied(worth_signal::facade::branch::SignalBranchRetirementBatchDenial::Retirement {
            position: 0,
            denial: worth_signal::facade::runtime::SignalBranchRetirementDenial::RetainedComponentBasis {
                branch_id,
                active_leases: 1,
            },
        }) if branch_id == child.id
    ));
    assert!(matches!(
        runtime.plan_signal_branch_retirement_batch_releasing_snapshots(vec![(
            child.clone(),
            child_basis.clone(),
            Vec::new(),
            worth_signal::facade::SignalBranchRetirementReason::Superseded,
        )]),
        worth_proof::TransitionOutcome::Denied(worth_signal::facade::branch::SignalBranchRetirementBatchDenial::Retirement {
            position: 0,
            denial: worth_signal::facade::runtime::SignalBranchRetirementDenial::RetainedComponentBasis {
                branch_id,
                active_leases: 1,
            },
        }) if branch_id == child.id
    ));
    assert!(matches!(
        runtime.release_signal_component_basis(lease),
        worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome::Released(_)
    ));
    let observed = runtime
        .observe_signal_branch_basis(child)
        .expect("retention planning denials leave the target unchanged");
    assert_eq!(observed.observation(), child_basis.observation());
}
