//! Inherited source selection uses real fork parentage without a branch generation.
use super::*;
use crate::branch::ProductBranchAncestorSelectionDenial;
use crate::history::CompositeAncestorSelectionDenial;
use std::num::NonZeroUsize;

#[test]
fn inherited_source_selection_crosses_real_fork_roots_and_refuses_a_sibling() {
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let (_fixture, owner, source) = setup_with_relational_source(4);
    let fork = create_forked_branch(
        &owner,
        &source,
        fork_intent(
            "ancestor-first-fork",
            relational_fork("ancestor-first-relational"),
            SignalBranchCreationPlan::ReuseExact,
        ),
    );
    let target = create_forked_branch(
        &owner,
        &fork,
        fork_intent(
            "ancestor-nested-fork",
            relational_fork("ancestor-nested-relational"),
            SignalBranchCreationPlan::ReuseExact,
        ),
    );
    let sibling = create_forked_branch(
        &owner,
        &source,
        fork_intent(
            "ancestor-sibling",
            relational_fork("ancestor-sibling-relational"),
            SignalBranchCreationPlan::ReuseExact,
        ),
    );
    assert_eq!(target.reference_generation().get(), 0);
    let branch_history = owner
        .trace_product_branch_ancestry(&execution, target.lifecycle_incarnation(), None)
        .unwrap();
    assert_eq!(branch_history.visited_count(), 1);
    assert!(branch_history.is_complete());
    let selected = owner
        .select_product_branch_ancestor(&execution, &target, source.selected_commit(), None)
        .expect("a real source before both fork roots remains selectable");
    assert_eq!(selected.source().identity(), source.selected_commit());
    assert_eq!(selected.source().basis(), source.basis());
    assert_eq!(selected.target(), &target);
    assert!(matches!(
        owner.select_product_branch_ancestor(&execution, &target, sibling.selected_commit(), None,),
        Err(ProductBranchAncestorSelectionDenial::History(
            CompositeAncestorSelectionDenial::NotAncestor,
        ))
    ));
    assert!(matches!(
        owner.select_product_branch_ancestor(
            &execution,
            &target,
            source.selected_commit(),
            NonZeroUsize::new(1),
        ),
        Err(ProductBranchAncestorSelectionDenial::History(
            CompositeAncestorSelectionDenial::SelectionLimitReached,
        ))
    ));
    assert_eq!(
        owner
            .observe_product_branch(target.branch_identity())
            .unwrap(),
        target
    );

    let (_foreign_fixture, foreign_owner, foreign) = setup_with_relational_source(1);
    assert!(matches!(
        foreign_owner.select_product_branch_ancestor(
            &execution,
            &target,
            foreign.selected_commit(),
            None,
        ),
        Err(ProductBranchAncestorSelectionDenial::Branch(
            RuntimeWorldBranchAdmissionDenial::ForeignOwner,
        ))
    ));
}
