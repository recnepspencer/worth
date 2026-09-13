#[path = "retirement_planning/snapshots.rs"]
mod snapshots;

use std::sync::Arc;

use worth_proof::TransitionOutcome;

use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchRetirementDenial, SignalBranchRetirementReason,
};
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalRuntime;
use crate::state::SignalBranchHandle;

use super::super::{SignalOwner, SignalOwnerCancellationSource};
use super::retirement_receipt_oracle::expected_terminal_basis_digest;
use super::runtime_root::runtime_with_two_branches_from_graph;

type TestRuntime = SignalRuntime<(), (), (), (), ()>;
type TestOwner = Arc<SignalOwner<(), (), ()>>;

fn populated_runtime_with_two_branches() -> (
    TestRuntime,
    SignalBranchHandle,
    SignalBranchHandle,
    AdmittedSignalBranchBasis,
) {
    let mut graph = SignalGraph::new();
    let source = graph.create_node();
    let dependent = graph.create_node();
    graph
        .set_dependencies(dependent, [DependencyEdge::new(source, Aspect::new(0))])
        .expect("the planning fixture installs populated dependency state");
    runtime_with_two_branches_from_graph(graph)
}

fn seal_populated_target() -> (
    TestRuntime,
    TestOwner,
    SignalBranchHandle,
    SignalBranchHandle,
    AdmittedSignalBranchBasis,
) {
    let (mut runtime, selected, target, basis) = populated_runtime_with_two_branches();
    let (basis_port, _, _) = runtime
        .owner_port_slots()
        .expect("the populated runtime seals");
    let owner = basis_port
        .upgrade_owner()
        .expect("the sealed owner upgrades");
    (runtime, owner, selected, target, basis)
}

#[test]
fn owner_exact_retirement_plan_preserves_pre_effect_state_and_executes_real_handle() {
    let (_runtime, owner, selected, target, basis) = seal_populated_target();
    let expected_digest = expected_terminal_basis_digest(&target, basis.observation());
    let expected_observation = basis.observation().clone();
    let admission = owner.admit().expect("retirement planning admits once");
    let ledger_before = owner.retention_ledger_observation();
    let metadata_before = owner
        .metadata
        .retirement_contract_observation(&admission, target.id)
        .expect("planning metadata is admitted");
    let live_before = owner.live_count();

    let plan = match owner.plan_retirement_exact(
        &admission,
        basis,
        SignalBranchRetirementReason::DependencyCancellation,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("the exact owner plan should succeed: {other:?}"),
    };
    assert_eq!(plan.branch(), &target);
    assert_eq!(plan.admitted_basis().observation(), &expected_observation);
    assert_eq!(plan.terminal_basis_digest.as_str(), expected_digest);
    assert_eq!(plan.planned_child_membership_count(), 0);
    assert_eq!(owner.live_count(), live_before);
    assert_eq!(owner.retention_ledger_observation(), ledger_before);
    assert_eq!(
        owner
            .metadata
            .retirement_contract_observation(&admission, target.id)
            .expect("planning leaves metadata observable"),
        metadata_before
    );
    assert!(
        owner.observe_branch_exact(&admission, selected.id).is_ok(),
        "unrelated populated state remains observable after planning"
    );

    let reservation = owner
        .reserve_retirement(&admission, target.id)
        .expect("planning installed no competing retirement reservation");
    let cancellation = SignalOwnerCancellationSource::new();
    let receipt = reservation
        .execute(plan, &cancellation.token())
        .expect("the owner-produced plan executes through the canonical cell");
    assert_eq!(receipt.retired_branch(), &target);
    assert_eq!(receipt.terminal_basis_digest(), expected_digest);
}

#[test]
fn owner_retirement_planning_distinguishes_current_canonical_and_live_child() {
    let (mut current_runtime, current, _, target_basis) = populated_runtime_with_two_branches();
    let current_basis = current_runtime
        .observe_signal_branch_basis(current.clone())
        .expect("the selected populated branch admits");
    drop(target_basis);
    let (port, _, _) = current_runtime
        .owner_port_slots()
        .expect("the current-branch fixture seals");
    let current_owner = port.upgrade_owner().expect("the current owner upgrades");
    let current_admission = current_owner.admit().expect("current planning admits");
    assert!(matches!(
        current_owner.plan_retirement_exact(
            &current_admission,
            current_basis,
            SignalBranchRetirementReason::Rejected,
        ),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::CurrentBranch { branch_id })
            if branch_id == current.id
    ));

    let (mut canonical_runtime, canonical, selected, selected_basis) =
        populated_runtime_with_two_branches();
    let canonical_basis = canonical_runtime
        .observe_signal_branch_basis(canonical.clone())
        .expect("the canonical basis admits");
    canonical_runtime
        .switch_branch(selected)
        .expect("the child becomes the actual selected branch");
    drop(selected_basis);
    let (port, _, _) = canonical_runtime
        .owner_port_slots()
        .expect("the noncanonical selection seals");
    let canonical_owner = port.upgrade_owner().expect("the canonical owner upgrades");
    let canonical_admission = canonical_owner.admit().expect("canonical planning admits");
    assert!(matches!(
        canonical_owner.plan_retirement_exact(
            &canonical_admission,
            canonical_basis,
            SignalBranchRetirementReason::Rejected,
        ),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::CanonicalBranch { branch_id })
            if branch_id == canonical.id
    ));

    let (mut child_runtime, _, parent, parent_basis) = populated_runtime_with_two_branches();
    let (child, child_basis) = child_runtime
        .fork_signal_branch("retirement-planning-live-child", &parent_basis)
        .expect("the real parent forks a live child")
        .into_parts();
    drop(child_basis);
    let (port, _, _) = child_runtime
        .owner_port_slots()
        .expect("the live-child fixture seals");
    let child_owner = port.upgrade_owner().expect("the child owner upgrades");
    let child_admission = child_owner.admit().expect("child planning admits");
    assert!(matches!(
        child_owner.plan_retirement_exact(
            &child_admission,
            parent_basis,
            SignalBranchRetirementReason::Rejected,
        ),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::LiveChildren {
            branch_id,
            child_branch_ids,
        }) if branch_id == parent.id && child_branch_ids == vec![child.id]
    ));
}

#[test]
fn owner_retirement_planning_checks_complete_basis_and_owner_before_registry_contact() {
    let (mut stale_runtime, _, _target, stale_basis) = populated_runtime_with_two_branches();
    let (snapshot, refreshed_basis) = stale_runtime
        .capture_signal_branch_snapshot(&stale_basis)
        .expect("a real capture advances the target observation")
        .into_parts();
    drop(snapshot);
    drop(refreshed_basis);
    let (port, _, _) = stale_runtime
        .owner_port_slots()
        .expect("the stale-basis fixture seals");
    let stale_owner = port.upgrade_owner().expect("the stale owner upgrades");
    let stale_admission = stale_owner.admit().expect("stale planning admits");
    assert!(matches!(
        stale_owner.plan_retirement_exact(
            &stale_admission,
            stale_basis,
            SignalBranchRetirementReason::Rejected,
        ),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::CanonicalBasisMismatch)
    ));

    let (_runtime_a, owner_a, _, _, basis_a) = seal_populated_target();
    let (_runtime_b, owner_b, _, _, basis_b) = seal_populated_target();
    drop(basis_a);
    let admission_a = owner_a.admit().expect("the receiving owner admits");
    let before = owner_a.cost_snapshot();
    assert!(matches!(
        owner_a.plan_retirement_exact(
            &admission_a,
            basis_b,
            SignalBranchRetirementReason::Rejected,
        ),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::CanonicalBasisMismatch)
    ));
    assert_eq!(
        owner_a.cost_snapshot().branch_registry_lookups(),
        before.branch_registry_lookups(),
        "foreign admitted authority denies before registry contact"
    );
    drop(owner_b);
}

#[test]
fn owner_retirement_planning_preserves_distinct_retention_and_holder_denials() {
    let (_runtime, owner, _, target, basis) = seal_populated_target();
    let admission = owner.admit().expect("external-retention planning admits");
    let external = owner
        .acquire_external_retention(&admission, &basis)
        .expect("the real descriptor opens one exact external obligation");
    assert!(matches!(
        owner.plan_retirement_exact(
            &admission,
            basis,
            SignalBranchRetirementReason::Rejected,
        ),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::RetainedComponentBasis {
            branch_id,
            active_leases: 1,
        }) if branch_id == target.id
    ));
    drop(external);

    let (_runtime, owner, _, target, basis) = seal_populated_target();
    let admission = owner.admit().expect("admitted-retention planning admits");
    let extra = owner
        .acquire_admitted_retention(&admission, target.id)
        .expect("the exact branch gains one additional admitted lease");
    assert!(matches!(
        owner.plan_retirement_exact(
            &admission,
            basis,
            SignalBranchRetirementReason::Rejected,
        ),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::RetainedAdmittedBasis {
            branch_id,
            active_leases: 2,
        }) if branch_id == target.id
    ));
    drop(extra);

    let (_runtime, owner, _, target, basis) = seal_populated_target();
    let shared = basis.clone();
    let admission = owner.admit().expect("shared-holder planning admits");
    assert!(matches!(
        owner.plan_retirement_exact(
            &admission,
            basis,
            SignalBranchRetirementReason::Rejected,
        ),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::SharedAdmittedBasis {
            branch_id,
            shared_holders: 2,
        }) if branch_id == target.id
    ));
    drop(shared);
}

#[test]
fn owner_retirement_planning_preserves_reachable_merge_participant_denial() {
    let (mut runtime, selected, target, basis) = populated_runtime_with_two_branches();
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime.inject_merge_participation_unwind_for_owner_contract(selected.id, target.id);
    }));
    assert!(unwind.is_err(), "the controlled merge boundary unwinds");
    let (port, _, _) = runtime
        .owner_port_slots()
        .expect("the actual interrupted merge metadata transfers to the owner");
    let owner = port.upgrade_owner().expect("the merge owner upgrades");
    let admission = owner.admit().expect("merge-posture planning admits");
    assert!(matches!(
        owner.plan_retirement_exact(
            &admission,
            basis,
            SignalBranchRetirementReason::Rejected,
        ),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::MergeParticipant { branch_id })
            if branch_id == target.id
    ));
}
