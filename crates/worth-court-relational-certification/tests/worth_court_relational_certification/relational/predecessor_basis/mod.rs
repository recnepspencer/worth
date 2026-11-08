//! Public Native predecessor truth, custody and cold-unavailability courts.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis, history::CommitId,
    mvcc::RelationalPredecessorBasisDenial as Denial, runtime::RelationalRuntime,
};
mod fixture;
use fixture::{commit, World};
fn request() -> worth_execution::SerialRequest {
    worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request(
    )
}
#[test]
fn first_publication_has_no_predecessor_basis() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution);
    assert_eq!(
        world
            .runtime
            .predecessor_publication_basis(execution, &world.basis(), None, None),
        Err(Denial::PredecessorBasisUnavailable)
    );
}
#[test]
fn exact_publication_basis_keeps_old_truth_after_source_edits() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution);
    let original = world.basis().descriptor().clone();
    let original_value = world
        .read(execution, &world.basis())
        .authoritative_aspect_state;
    let prior = world.birth(execution);
    world.change_source(execution, "new-input");
    let selected = world.basis();
    let old = world
        .runtime
        .predecessor_publication_basis(execution, &selected, Some(&prior), Some(commit(&prior)))
        .unwrap();
    assert_eq!(old.descriptor(), &original);
    assert_eq!(
        world
            .runtime
            .readmit_branch_basis(old.descriptor())
            .unwrap(),
        old
    );
    assert_ne!(old.descriptor(), selected.descriptor());
    assert_eq!(
        world.read(execution, &old).authoritative_aspect_state,
        original_value
    );
    assert_ne!(
        world.read(execution, &selected).authoritative_aspect_state,
        original_value
    );
}
#[test]
fn retained_selection_survives_unrelated_publication_but_cannot_select_a_future_predecessor() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution);
    let before_publication = world.basis();
    let prior = world.birth(execution);
    let selected = world.basis();
    world.add_unrelated_publication(execution);
    let old = world
        .runtime
        .predecessor_publication_basis(execution, &selected, Some(&prior), Some(commit(&prior)))
        .unwrap();
    assert_eq!(old.descriptor(), before_publication.descriptor());
    assert_eq!(
        world.runtime.predecessor_publication_basis(
            execution,
            &before_publication,
            Some(&prior),
            Some(commit(&prior)),
        ),
        Err(Denial::PredecessorMismatch)
    );
}
#[test]
fn forged_or_succeeded_predecessor_coordinates_refuse_mismatch() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution);
    let prior = world.birth(execution);
    assert_eq!(
        world.runtime.predecessor_publication_basis(
            execution,
            &world.basis(),
            Some(&prior),
            Some(world.source_commit)
        ),
        Err(Denial::PredecessorMismatch)
    );
    assert_eq!(
        world
            .runtime
            .predecessor_publication_basis(execution, &world.basis(), Some(&prior), None),
        Err(Denial::PredecessorMismatch)
    );
    let _next = world.preserve(execution, prior.clone());
    assert_eq!(
        world.runtime.predecessor_publication_basis(
            execution,
            &world.basis(),
            Some(&prior),
            Some(commit(&prior))
        ),
        Err(Denial::PredecessorMismatch)
    );
}
#[test]
fn cold_checkpoint_keeps_publication_but_refuses_unretained_decision_root() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution);
    let prior = world.birth(execution);
    world.change_source(execution, "cold-current-input");
    let (cold, restored) = world.reopen(execution, &prior);
    assert_eq!(commit(&restored), commit(&prior));
    assert_eq!(
        cold.runtime.predecessor_publication_basis(
            execution,
            &cold.basis(),
            Some(&restored),
            Some(commit(&restored))
        ),
        Err(Denial::PredecessorBasisUnavailable)
    );
}
mod cost {
    use super::*;
    #[test]
    fn live_retention_stays_bounded_across_succession_and_releases_at_retirement() {
        let host = request();
        let execution = &ExecutionRequest::serial(&host);
        let world = World::new(execution);
        let baseline = world.active_holds();
        let mut prior = world.birth(execution);
        let live = world.active_holds();
        assert!(
            live > baseline,
            "a live publication must retain its decision basis"
        );
        for _ in 0..24 {
            prior = world.preserve(execution, prior);
            assert_eq!(
                world.active_holds(),
                live,
                "retention must not grow with succession history"
            );
        }
        world.retire(execution, &prior);
        assert!(
            world.active_holds() < live,
            "retirement releases the publication hold"
        );
        assert_eq!(world.active_holds(), baseline);
    }
}

mod coupled_inputs;
mod edit_admission;
