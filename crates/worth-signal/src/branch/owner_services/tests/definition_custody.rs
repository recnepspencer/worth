use crate::branch::validate_signal_branch_name;
use crate::data::aspect::SignalAspectLoweringOwner;
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalRuntime;
use crate::state::SignalBranchId;

use super::super::{SignalOwner, SignalOwnerCancellationSource};
use super::runtime_root::runtime_with_two_branches_from_graph;

fn source_authority() -> worth_proof::ConditionalSourceObservationAuthority {
    worth_proof::ConditionalSourceObservationOwner::fresh().authority()
}

fn assert_claimant(
    owner: &std::sync::Arc<SignalOwner<(), (), ()>>,
    branch: SignalBranchId,
    claimant: &SignalAspectLoweringOwner,
    expected: Option<bool>,
) {
    let admission = owner.admit().unwrap();
    let cell = owner.lookup_cell(&admission, branch).unwrap();
    let actual = cell
        .with_state(&admission, |state, _| {
            state
                .state()
                .installed_definition()
                .map(|binding| binding.is_claimed_by(claimant))
        })
        .unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn sealing_preserves_each_actual_claimant_and_owner_fork_carries_its_source() {
    let claimant_a = SignalAspectLoweringOwner::fresh();
    let claimant_b = SignalAspectLoweringOwner::fresh();
    let mut graph = SignalGraph::new();
    graph.claim_aspect_lowering_owner(&claimant_a).unwrap();
    let (mut runtime, branch_a, branch_b, _) = runtime_with_two_branches_from_graph(graph);
    runtime.switch_branch(branch_b.clone()).unwrap();
    runtime
        .graph_mut()
        .claim_aspect_lowering_owner(&claimant_b)
        .unwrap();
    let basis_b = runtime
        .observe_signal_branch_basis(branch_b.clone())
        .unwrap();
    let basis_a = runtime
        .observe_signal_branch_basis(branch_a.clone())
        .unwrap();
    let (_, mutation, _) = runtime.owner_port_slots().unwrap();
    let owner = mutation.upgrade_owner().unwrap();

    assert_claimant(&owner, branch_a.id, &claimant_a, Some(true));
    assert_claimant(&owner, branch_a.id, &claimant_b, Some(false));
    assert_claimant(&owner, branch_b.id, &claimant_a, Some(false));
    assert_claimant(&owner, branch_b.id, &claimant_b, Some(true));
    assert!(matches!(
        runtime.issue_conditional_execution_service(&basis_b, &claimant_a, &source_authority()),
        Err(super::super::SignalConditionalServiceIssuanceDenial::ClaimantMismatch),
    ));
    runtime
        .issue_conditional_execution_service(&basis_a, &claimant_a, &source_authority())
        .unwrap();
    runtime
        .issue_conditional_execution_service(&basis_b, &claimant_b, &source_authority())
        .unwrap();

    let cancellation = SignalOwnerCancellationSource::new();
    let child = mutation
        .fork_exact(
            validate_signal_branch_name("claimed-child").unwrap(),
            &basis_b,
            &cancellation.token(),
        )
        .unwrap();
    assert_claimant(&owner, child.created_branch().id, &claimant_b, Some(true));
    assert_claimant(&owner, child.created_branch().id, &claimant_a, Some(false));
}

#[test]
fn snapshot_restore_preserves_installed_custody_and_exact_absence() {
    let claimant = SignalAspectLoweringOwner::fresh();
    let mut graph = SignalGraph::new();
    graph.claim_aspect_lowering_owner(&claimant).unwrap();
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    let branch = runtime.current_branch();
    let initial = runtime.observe_signal_branch_basis(branch.clone()).unwrap();
    let (preseal_snapshot, preseal_basis) = runtime
        .capture_signal_branch_snapshot(&initial)
        .unwrap()
        .into_parts();
    let (_, mutation, _) = runtime.owner_port_slots().unwrap();
    let owner = mutation.upgrade_owner().unwrap();
    let cancellation = SignalOwnerCancellationSource::new();
    assert_claimant(&owner, branch.id, &claimant, Some(true));

    let (installed_snapshot, installed_basis) = mutation
        .capture_exact(&preseal_basis, &cancellation.token())
        .unwrap()
        .into_parts();
    assert_claimant(&owner, branch.id, &claimant, Some(true));
    let restored = mutation
        .restore_exact(&installed_basis, &preseal_snapshot, &cancellation.token())
        .unwrap();
    assert_claimant(&owner, branch.id, &claimant, None);
    assert!(matches!(
        runtime.issue_conditional_execution_service(&restored, &claimant, &source_authority()),
        Err(super::super::SignalConditionalServiceIssuanceDenial::DefinitionReadmissionRequired),
    ));
    mutation
        .restore_exact(&restored, &installed_snapshot, &cancellation.token())
        .unwrap();
    assert_claimant(&owner, branch.id, &claimant, Some(true));
}
