use worth_foundational::{
    foundational_branch_local_state_definitions, FoundationalBranchId,
    FoundationalBranchIdConstructionDenial, FoundationalBranchLocalStateKind,
};

use super::fixtures::branch::{
    authority_first_candidate, projection_shaped_candidate, staged_candidate,
};

#[test]
fn branch_local_state_definitions_remain_non_authoritative() {
    let definitions = foundational_branch_local_state_definitions();

    assert_eq!(definitions.len(), 2);
    assert_eq!(
        definitions[0].kind(),
        FoundationalBranchLocalStateKind::Candidate
    );
    assert_eq!(
        definitions[1].kind(),
        FoundationalBranchLocalStateKind::Staged
    );
    assert!(definitions
        .iter()
        .all(|definition| definition.must_not_mean().contains("authority")));
}

#[test]
fn independent_branch_local_producers_preserve_candidate_meaning() {
    let authority_first = authority_first_candidate("mesh-update");
    let projection_shaped = projection_shaped_candidate("mesh-update");

    assert_eq!(authority_first.branch_id(), projection_shaped.branch_id());
    assert_eq!(
        authority_first.candidate_id(),
        projection_shaped.candidate_id()
    );
    assert_eq!(authority_first.fork_basis(), projection_shaped.fork_basis());
    assert_eq!(
        authority_first.observation_basis(),
        projection_shaped.observation_basis()
    );
    assert_eq!(
        authority_first.fork_observation_basis(),
        projection_shaped.fork_observation_basis()
    );
    assert_eq!(
        authority_first.comparison_basis(),
        projection_shaped.comparison_basis()
    );
    assert_eq!(authority_first.payload(), projection_shaped.payload());
    assert_eq!(
        authority_first.branch_local_state_kind(),
        FoundationalBranchLocalStateKind::Candidate
    );
}

#[test]
fn staged_branch_surface_remains_separate_from_candidate_surface() {
    let candidate = authority_first_candidate("mesh-update");
    let staged = staged_candidate("mesh-update");

    assert_eq!(candidate.branch_id(), staged.branch_id());
    assert_eq!(candidate.candidate_id(), staged.candidate_id());
    assert_eq!(candidate.fork_basis(), staged.fork_basis());
    assert_eq!(candidate.payload(), staged.payload());
    assert_eq!(
        staged.branch_local_state_kind(),
        FoundationalBranchLocalStateKind::Staged
    );
    assert_ne!(
        candidate.branch_local_state_definition().kind(),
        staged.branch_local_state_definition().kind()
    );
}

#[test]
fn blind_consumer_can_interpret_branch_local_basis_without_runtime_state() {
    let staged = staged_candidate("mesh-update");

    assert_eq!(staged.branch_id().as_str(), "feature/geometry");
    assert_eq!(staged.fork_basis().forked_from_branch().as_str(), "main");
    assert_eq!(staged.fork_basis().fork_epoch().get(), 4);
    assert_eq!(staged.observation_basis().basis_id().get(), 31);
    assert_eq!(staged.observation_basis().observed_epoch().get(), 5);

    let comparison_basis = staged
        .comparison_basis()
        .expect("comparison basis should remain explicit");
    assert_eq!(comparison_basis.basis_id().get(), 43);
    assert_eq!(comparison_basis.compared_against_branch().as_str(), "main");
}

#[test]
fn branch_id_rejects_empty_names() {
    let denial = FoundationalBranchId::new("  ").expect_err("empty branch ids fail closed");
    assert_eq!(denial, FoundationalBranchIdConstructionDenial::EmptyName);
}
