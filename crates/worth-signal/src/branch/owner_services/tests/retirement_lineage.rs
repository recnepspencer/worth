use worth_proof::TransitionOutcome;

use crate::branch::{
    validate_signal_branch_name, SignalBranchForkOperationDenial, SignalBranchRetirementDenial,
    SignalBranchRetirementReason,
};
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalRuntime;

use super::super::SignalOwnerCancellationSource;
use super::runtime_root::runtime_with_two_branches;

#[test]
fn retirement_lineage_reservation_cannot_miss_a_concurrent_fork_commit() {
    let (mut runtime, _, source_branch, source_basis) = runtime_with_two_branches();
    let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = mutation.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("owner admits lifecycle setup");
    let retirement = owner
        .reserve_retirement(&admission, source_branch.id)
        .expect("lineage and receipt capacity reserve before cell work");
    let reservations_before = owner.reservation_count();

    let denial = owner.reserve_fork_destination(
        &admission,
        &source_basis,
        validate_signal_branch_name("fork-during-retirement").expect("fork identity validates"),
    );
    assert!(matches!(
        denial,
        Err(SignalBranchForkOperationDenial::RetirementInProgress { branch_id })
            if branch_id == source_branch.id
    ));
    assert_eq!(owner.reservation_count(), reservations_before);
    drop(retirement);

    let healthy = owner
        .reserve_fork_destination(
            &admission,
            &source_basis,
            validate_signal_branch_name("fork-after-retirement-cancel")
                .expect("healthy identity validates"),
        )
        .expect("no-effect retirement cancellation reopens lineage and registry membership");
    assert_eq!(healthy.branch().parent_branch_id, Some(source_branch.id));
}

#[test]
fn retirement_child_check_observes_a_fork_completed_before_reservation() {
    let (mut runtime, _, source_branch, source_basis) = runtime_with_two_branches();
    let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = mutation.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("fork and retirement admit");
    let source_cell = owner
        .lookup_cell(&admission, source_branch.id)
        .expect("the source cell is live");
    let (child, child_basis) = owner
        .reserve_fork_output(&admission, &source_cell)
        .expect("the completed-first output custody reserves")
        .fork(
            &source_basis,
            validate_signal_branch_name("completed-before-retirement")
                .expect("the destination identity validates"),
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("the completed-first destination installs")
        .into_destination_parts();
    assert_eq!(child_basis.owner_branch_id(), child.id);
    drop(child_basis);

    assert!(matches!(
        owner.reserve_retirement(&admission, source_branch.id),
        Err(SignalBranchRetirementDenial::LiveChildren {
            branch_id,
            child_branch_ids,
        }) if branch_id == source_branch.id && child_branch_ids == vec![child.id]
    ));
    assert!(
        owner.lookup_cell(&admission, source_branch.id).is_ok(),
        "denied retirement leaves the source live"
    );
}

#[test]
fn postseal_receipt_capacity_denies_the_next_live_target_pre_effect() {
    const MAXIMUM_RETAINED_RECEIPTS: usize = 4_096;
    const PRESEAL_RECEIPTS: usize = MAXIMUM_RETAINED_RECEIPTS - 5;

    let mut runtime = SignalRuntime::build_for::<()>(SignalGraph::new());
    let selected = runtime.current_branch();
    let selected_basis = runtime
        .observe_signal_branch_basis(selected)
        .expect("the sequential source basis admits");
    for index in 0..PRESEAL_RECEIPTS {
        let (branch, basis) = runtime
            .fork_signal_branch(format!("preseal-receipt-{index}"), &selected_basis)
            .expect("one preseal sibling forks")
            .into_parts();
        let plan = match runtime.plan_signal_branch_retirement(
            branch,
            basis,
            SignalBranchRetirementReason::Superseded,
        ) {
            TransitionOutcome::Success(plan) => plan,
            other => panic!("preseal receipt plans: {other:?}"),
        };
        assert!(matches!(
            runtime.retire_signal_branch(plan),
            TransitionOutcome::Success(_)
        ));
    }
    let mut planned = Vec::new();
    for index in 0..6 {
        let (branch, basis) = runtime
            .fork_signal_branch(format!("postseal-receipt-{index}"), &selected_basis)
            .expect("the bounded postseal target forks")
            .into_parts();
        let plan = match runtime.plan_signal_branch_retirement(
            branch.clone(),
            basis,
            SignalBranchRetirementReason::Superseded,
        ) {
            TransitionOutcome::Success(plan) => plan,
            other => panic!("postseal target plans: {other:?}"),
        };
        planned.push((branch, plan));
    }

    let (_, _, lifecycle) = runtime.owner_port_slots().expect("runtime seals");
    let owner = lifecycle.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("retirement execution admits");
    let cancellation = SignalOwnerCancellationSource::new();
    for (branch, plan) in planned.drain(..5) {
        let receipt = owner
            .reserve_retirement(&admission, branch.id)
            .expect("capacity remains for the performed receipt")
            .execute(plan, &cancellation.token())
            .expect("the reserved retirement performs");
        assert_eq!(receipt.retired_branch(), &branch);
    }
    let (denied_branch, denied_plan) = planned
        .pop()
        .expect("one live target remains after capacity fills");
    assert!(matches!(
        owner.reserve_retirement(&admission, denied_branch.id),
        Err(
            SignalBranchRetirementDenial::RetirementReceiptCapacityExhausted {
                maximum_retained_receipts: MAXIMUM_RETAINED_RECEIPTS,
            }
        )
    ));
    assert!(owner.lookup_cell(&admission, denied_branch.id).is_ok());
    assert_eq!(owner.live_count(), 2);
    drop(denied_plan);
}
