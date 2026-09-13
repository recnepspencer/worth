use worth_proof::TransitionOutcome;

use crate::branch::{SignalBranchRetentionAcquisitionDenial, SignalBranchRetirementReason};
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalRuntime;
use crate::state::{SignalBranchHandle, SignalBranchId, SignalSnapshotId};

use super::super::super::SignalOwnerCancellationSource;
use super::super::runtime_root::runtime_with_two_branches;
use super::owner_admitted_exact_target;

#[test]
fn retention_preflight_denies_foreign_definition_unknown_and_unavailable_before_ledger_contact() {
    let mut shared_graph = SignalGraph::new();
    let (same_graph_other_owner, _) = shared_graph.fork_persistent();
    let mut runtime_a = SignalRuntime::<(), (), (), (), ()>::build_for::<()>(shared_graph);
    let mut runtime_b =
        SignalRuntime::<(), (), (), (), ()>::build_for::<()>(same_graph_other_owner);
    let branch_b = runtime_b.current_branch();
    let foreign_basis = runtime_b
        .observe_signal_branch_basis(branch_b)
        .expect("foreign same-graph owner issues a real basis");
    let (port_a, _, _) = runtime_a.owner_port_slots().expect("owner A seals");
    let (port_b, _, _) = runtime_b.owner_port_slots().expect("owner B seals");
    let owner_a = port_a.upgrade_owner().expect("owner A remains live");
    let _owner_b = port_b.upgrade_owner().expect("owner B remains live");
    assert_eq!(
        foreign_basis
            .observation()
            .target()
            .as_basis()
            .expect("foreign basis target is exact")
            .graph_instance_id(),
        owner_a.runtime_instance_id().to_string()
    );
    let admission_a = owner_a.admit().expect("owner A admits");
    assert_preflight_denial_without_retention_contact(
        &owner_a,
        &admission_a,
        &foreign_basis,
        SignalBranchRetentionAcquisitionDenial::ForeignBasis,
    );

    let (mut runtime, sibling, branch, basis) = runtime_with_two_branches();
    let (_, mutation, _) = runtime.owner_port_slots().expect("hostile owner seals");
    let owner = mutation
        .upgrade_owner()
        .expect("hostile owner remains live");
    let admission = owner.admit().expect("hostile preflight admits");
    let wrong_definition = owner_admitted_exact_target(
        &owner,
        &admission,
        &branch,
        branch.id,
        owner.definition_basis() + 1,
        None,
    );
    assert_preflight_denial_without_retention_contact(
        &owner,
        &admission,
        &wrong_definition,
        SignalBranchRetentionAcquisitionDenial::DefinitionMismatch {
            basis_definition_basis: owner.definition_basis() + 1,
            runtime_definition_basis: owner.definition_basis(),
        },
    );

    let unknown = SignalBranchHandle {
        id: SignalBranchId(91_701),
        name: "owner-preflight-unknown".to_owned(),
        parent_branch_id: Some(sibling.id),
        head_snapshot_id: None,
    };
    let unknown_basis = owner_admitted_exact_target(
        &owner,
        &admission,
        &unknown,
        sibling.id,
        owner.definition_basis(),
        None,
    );
    assert_preflight_denial_without_retention_contact(
        &owner,
        &admission,
        &unknown_basis,
        SignalBranchRetentionAcquisitionDenial::UnknownBranch {
            branch_id: unknown.id,
        },
    );

    let unavailable = owner_admitted_exact_target(
        &owner,
        &admission,
        &branch,
        branch.id,
        owner.definition_basis(),
        Some(SignalSnapshotId(91_702)),
    );
    assert_preflight_denial_without_retention_contact(
        &owner,
        &admission,
        &unavailable,
        SignalBranchRetentionAcquisitionDenial::UnavailableTarget {
            branch_id: branch.id,
            snapshot_id: SignalSnapshotId(91_702),
        },
    );
    drop(basis);
}

#[test]
fn retention_preflight_distinguishes_a_real_retired_membership_before_ledger_contact() {
    let (mut runtime, sibling, branch, basis) = runtime_with_two_branches();
    let plan = match runtime.plan_signal_branch_retirement(
        branch.clone(),
        basis,
        SignalBranchRetirementReason::Rejected,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("retired preflight fixture plans: {other:?}"),
    };
    let (_, _, lifecycle) = runtime.owner_port_slots().expect("retired owner seals");
    let owner = lifecycle
        .upgrade_owner()
        .expect("retired owner remains live");
    let admission = owner.admit().expect("retirement admits");
    owner
        .reserve_retirement(&admission, branch.id)
        .expect("retirement reserves")
        .execute(plan, &SignalOwnerCancellationSource::new().token())
        .expect("retirement performs and records a receipt");
    let retired_basis = owner_admitted_exact_target(
        &owner,
        &admission,
        &branch,
        sibling.id,
        owner.definition_basis(),
        branch.head_snapshot_id,
    );
    assert_preflight_denial_without_retention_contact(
        &owner,
        &admission,
        &retired_basis,
        SignalBranchRetentionAcquisitionDenial::RetiredBranch {
            branch_id: branch.id,
        },
    );
}

fn assert_preflight_denial_without_retention_contact(
    owner: &super::TestOwner,
    admission: &super::super::super::SignalOwnerOperationAdmission<'_>,
    basis: &crate::branch::AdmittedSignalBranchBasis,
    expected: SignalBranchRetentionAcquisitionDenial,
) {
    let ledger_before = owner.retention_ledger_observation();
    let contacts_before = owner.cost_snapshot().retention_registry_contacts();
    let denial = owner
        .acquire_external_retention(admission, basis)
        .expect_err("preflight must deny before retention contact");
    assert_eq!(denial, expected);
    assert_eq!(owner.retention_ledger_observation(), ledger_before);
    assert_eq!(
        owner.cost_snapshot().retention_registry_contacts(),
        contacts_before
    );
}
