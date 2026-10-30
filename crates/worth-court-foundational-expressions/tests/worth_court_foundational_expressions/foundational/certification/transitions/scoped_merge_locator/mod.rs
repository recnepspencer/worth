#[allow(unused_imports)]
use crate::certification::ordinary_context::transitions::scoped_merge_locator::*;

use crate::certification::ordinary_context::transitions::fixtures::branch::branch_id;
use {
    crate::certification::ordinary_context::transitions::fixtures::merge::authority_first_merge_candidate,
    crate::certification::ordinary_context::transitions::fixtures::merge::conflict_locus,
};
use {
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_aspect,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_node,
};

#[test]
fn selected_node_scope_locator_lowers_to_transition_locator_basis() {
    let entries = locator_entries(
        worth_foundational::FoundationalTransitionLocator::SelectedNodeScope(
            worth_foundational::FoundationalSelectedNodeScopeLocator::new(
                branch_id("feature/geometry"),
                branch_id("main"),
                selected_node("gear"),
            ),
        ),
    );

    assert_eq!(
        entries,
        vec![
            transition_locator_text_entry(
                "transition.selected_node_scope.kind",
                "selected-node-scope"
            ),
            transition_locator_text_entry("transition.selected_node_scope.node", "gear"),
            transition_locator_text_entry(
                "transition.selected_node_scope.source_branch",
                "feature/geometry",
            ),
            transition_locator_text_entry("transition.selected_node_scope.target_branch", "main"),
        ]
    );
}

#[test]
fn selected_aspect_scope_locator_lowers_node_and_aspect_without_collapse() {
    let entries = locator_entries(
        worth_foundational::FoundationalTransitionLocator::SelectedAspectScope(
            worth_foundational::FoundationalSelectedAspectScopeLocator::new(
                branch_id("feature/geometry"),
                branch_id("main"),
                selected_aspect("gear", "teeth"),
            ),
        ),
    );

    assert_eq!(
        entries,
        vec![
            transition_locator_text_entry("transition.selected_aspect_scope.aspect", "teeth"),
            transition_locator_text_entry(
                "transition.selected_aspect_scope.kind",
                "selected-aspect-scope",
            ),
            transition_locator_text_entry("transition.selected_aspect_scope.node", "gear"),
            transition_locator_text_entry(
                "transition.selected_aspect_scope.source_branch",
                "feature/geometry",
            ),
            transition_locator_text_entry("transition.selected_aspect_scope.target_branch", "main"),
        ]
    );
}

#[test]
fn scoped_locators_remain_distinct_from_merge_conflict_locators() {
    let merge_candidate = authority_first_merge_candidate("mesh-update");
    let conflict = ready_locator(
        worth_foundational::FoundationalTransitionLocator::MergeConflict(
            worth_foundational::FoundationalMergeConflictLocator::new(
                merge_candidate.source_branch().clone(),
                merge_candidate.target_branch().clone(),
                conflict_locus(),
            ),
        ),
    );
    let selected_node = ready_locator(
        worth_foundational::FoundationalTransitionLocator::SelectedNodeScope(
            worth_foundational::FoundationalSelectedNodeScopeLocator::new(
                branch_id("feature/geometry"),
                branch_id("main"),
                selected_node("geometry-face"),
            ),
        ),
    );
    let selected_aspect = ready_locator(
        worth_foundational::FoundationalTransitionLocator::SelectedAspectScope(
            worth_foundational::FoundationalSelectedAspectScopeLocator::new(
                branch_id("feature/geometry"),
                branch_id("main"),
                selected_aspect("geometry-face", "teeth"),
            ),
        ),
    );

    assert_mismatched(conflict.clone(), selected_node.clone());
    assert_mismatched(conflict, selected_aspect.clone());
    assert_mismatched(selected_node, selected_aspect);
}

#[test]
fn selected_scope_diagnostic_locator_fragments_are_separator_safe() {
    let selected_node_left = worth_foundational::FoundationalDiagnosticLocator::Transition(
        worth_foundational::FoundationalTransitionLocator::SelectedNodeScope(
            worth_foundational::FoundationalSelectedNodeScopeLocator::new(
                branch_id("feature:geometry"),
                branch_id("main"),
                selected_node("gear"),
            ),
        ),
    );
    let selected_node_right = worth_foundational::FoundationalDiagnosticLocator::Transition(
        worth_foundational::FoundationalTransitionLocator::SelectedNodeScope(
            worth_foundational::FoundationalSelectedNodeScopeLocator::new(
                branch_id("feature"),
                branch_id("geometry:main"),
                selected_node("gear"),
            ),
        ),
    );
    assert_ne!(
        selected_node_left.canonical_key_fragment(),
        selected_node_right.canonical_key_fragment()
    );

    let selected_aspect_left = worth_foundational::FoundationalDiagnosticLocator::Transition(
        worth_foundational::FoundationalTransitionLocator::SelectedAspectScope(
            worth_foundational::FoundationalSelectedAspectScopeLocator::new(
                branch_id("feature"),
                branch_id("main"),
                selected_aspect("gear:teeth", "profile"),
            ),
        ),
    );
    let selected_aspect_right = worth_foundational::FoundationalDiagnosticLocator::Transition(
        worth_foundational::FoundationalTransitionLocator::SelectedAspectScope(
            worth_foundational::FoundationalSelectedAspectScopeLocator::new(
                branch_id("feature"),
                branch_id("main"),
                selected_aspect("gear", "teeth:profile"),
            ),
        ),
    );
    assert_ne!(
        selected_aspect_left.canonical_key_fragment(),
        selected_aspect_right.canonical_key_fragment()
    );
}

#[test]
fn selected_scope_locators_can_address_provenance_and_receipt_boundaries() {
    let selected_node_locator =
        worth_foundational::FoundationalTransitionLocator::SelectedNodeScope(
            worth_foundational::FoundationalSelectedNodeScopeLocator::new(
                branch_id("feature/geometry"),
                branch_id("main"),
                selected_node("gear"),
            ),
        );
    let selected_aspect_locator =
        worth_foundational::FoundationalTransitionLocator::SelectedAspectScope(
            worth_foundational::FoundationalSelectedAspectScopeLocator::new(
                branch_id("feature/geometry"),
                branch_id("main"),
                selected_aspect("gear", "teeth"),
            ),
        );

    let authority_path = worth_foundational::FoundationalBoundaryEvidenceAuthorityPath::transition(
        selected_node_locator.clone(),
    );
    let receipt_boundary =
        worth_foundational::FoundationalBoundaryEvidenceReceiptBoundary::transition(
            selected_aspect_locator.clone(),
        );

    assert_eq!(authority_path.locator(), &selected_node_locator);
    assert_eq!(receipt_boundary.locator(), &selected_aspect_locator);
}
