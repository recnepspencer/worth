//! Exact pair admission and the reference locks used by actual retirement.
use super::tests::{create_reused_branch, reuse_intent, setup};
use crate::branch::ProductBranchCurrentnessFailure;
use crate::lifecycle::{
    RuntimeWorldBranchService, RuntimeWorldObservationPort, RuntimeWorldObservationService,
};
use std::sync::Arc;

type TestOwner = crate::lifecycle::RuntimeWorldOwnerRoot<(), (), (), (), ()>;

fn port(owner: &Arc<TestOwner>) -> RuntimeWorldObservationPort {
    let service: Arc<dyn RuntimeWorldObservationService + Send + Sync> = owner.clone();
    RuntimeWorldObservationPort::new(Arc::downgrade(&service))
}

#[test]
fn pair_currentness_returns_identical_custody_when_candidate_retires_after_admission() {
    let (_fixture, owner, source) = setup(3);
    let candidate = create_reused_branch(&owner, &source, reuse_intent("candidate"));
    let first = owner.admit_product_branch_currentness(&source).unwrap();
    let second = owner.admit_product_branch_currentness(&candidate).unwrap();
    let retirement = owner.retire_product_branch(&candidate).unwrap();
    assert!(retirement.owner_retirement_work().is_empty());
    let custody = Box::new(43);
    let address = &*custody as *const i32;
    let stopped = first
        .while_both_current(second, custody, |_, _, _| {
            panic!("candidate retirement after admission must prevent installation")
        })
        .unwrap_err();
    let ProductBranchCurrentnessFailure::ExpectedHeadUnavailable(custody) = stopped else {
        panic!("the admitted candidate lost its exact reference head")
    };
    assert_eq!(&*custody as *const i32, address);
    assert_eq!(*custody, 43);
    assert_eq!(owner.state.operation.active(), 0);
    assert_eq!(
        owner
            .observe_product_branch(source.branch_identity())
            .unwrap(),
        source
    );
}

#[test]
fn pair_currentness_locks_both_retirement_cells_only_through_installation() {
    let (_fixture, owner, source) = setup(3);
    let candidate = create_reused_branch(&owner, &source, reuse_intent("candidate"));
    let source_cell = owner
        .state
        .branches
        .branch_cell(source.branch_identity())
        .unwrap();
    let candidate_cell = owner
        .state
        .branches
        .branch_cell(candidate.branch_identity())
        .unwrap();
    let owner = Arc::new(owner);
    let port = port(&owner);
    // Cover both acquisition orders and duplicate-cell handling without racing
    // a scheduler: retirement itself requires these exact cells' write locks.
    for (first, second) in [
        (&source, &candidate),
        (&candidate, &source),
        (&source, &source),
    ] {
        assert!(!source_cell.writers_are_locked_out_for_test());
        assert!(!candidate_cell.writers_are_locked_out_for_test());
        let result = port
            .while_product_branches_current(first, second, 44, |custody, a, b| {
                assert_eq!(a.observation(), first);
                assert_eq!(b.observation(), second);
                assert!(source_cell.writers_are_locked_out_for_test());
                if first != second {
                    assert!(candidate_cell.writers_are_locked_out_for_test());
                }
                custody + 1
            })
            .unwrap();
        assert_eq!(result, 45);
        assert!(!source_cell.writers_are_locked_out_for_test());
        assert!(!candidate_cell.writers_are_locked_out_for_test());
        assert_eq!(owner.state.operation.active(), 0);
    }
    let retirement = owner.retire_product_branch(&candidate).unwrap();
    assert!(retirement.owner_retirement_work().is_empty());
    let retirement = owner.retire_product_branch(&source).unwrap();
    assert!(retirement.owner_retirement_work().is_empty());
    assert_eq!(owner.state.operation.active(), 0);
}
