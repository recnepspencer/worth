use worth_foundational::{
    prepare_canonical_comparison, prepare_locator_for_canonical_basis, CanonicalBasisDomain,
    CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisReadyArtifact,
    CanonicalBasisValue, CanonicalComparisonOutcome, CanonicalEquivalenceBasis,
    CanonicalIntegerWidth, CanonicalLocatorInput, CanonicalizationRuleVersion,
    FoundationalBoundaryEvidenceAuthorityPath, FoundationalBoundaryEvidenceReceiptBoundary,
    FoundationalDiagnosticLocator, FoundationalMergeConflictLocator,
    FoundationalSelectedAspectScopeLocator, FoundationalSelectedNodeScopeLocator,
    FoundationalTransitionLocator,
};
use worth_proof::TransitionOutcome;

use super::fixtures::branch::branch_id;
use super::fixtures::merge::{authority_first_merge_candidate, conflict_locus};
use super::fixtures::scoped_merge::{selected_aspect, selected_node};

#[test]
fn selected_node_scope_locator_lowers_to_transition_locator_basis() {
    let entries = locator_entries(FoundationalTransitionLocator::SelectedNodeScope(
        FoundationalSelectedNodeScopeLocator::new(
            branch_id("feature/geometry"),
            branch_id("main"),
            selected_node("gear"),
        ),
    ));

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
    let entries = locator_entries(FoundationalTransitionLocator::SelectedAspectScope(
        FoundationalSelectedAspectScopeLocator::new(
            branch_id("feature/geometry"),
            branch_id("main"),
            selected_aspect("gear", "teeth"),
        ),
    ));

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
    let conflict = ready_locator(FoundationalTransitionLocator::MergeConflict(
        FoundationalMergeConflictLocator::new(
            merge_candidate.source_branch().clone(),
            merge_candidate.target_branch().clone(),
            conflict_locus(),
        ),
    ));
    let selected_node = ready_locator(FoundationalTransitionLocator::SelectedNodeScope(
        FoundationalSelectedNodeScopeLocator::new(
            branch_id("feature/geometry"),
            branch_id("main"),
            selected_node("geometry-face"),
        ),
    ));
    let selected_aspect = ready_locator(FoundationalTransitionLocator::SelectedAspectScope(
        FoundationalSelectedAspectScopeLocator::new(
            branch_id("feature/geometry"),
            branch_id("main"),
            selected_aspect("geometry-face", "teeth"),
        ),
    ));

    assert_mismatched(conflict.clone(), selected_node.clone());
    assert_mismatched(conflict, selected_aspect.clone());
    assert_mismatched(selected_node, selected_aspect);
}

#[test]
fn selected_scope_diagnostic_locator_fragments_are_separator_safe() {
    let selected_node_left = FoundationalDiagnosticLocator::Transition(
        FoundationalTransitionLocator::SelectedNodeScope(
            FoundationalSelectedNodeScopeLocator::new(
                branch_id("feature:geometry"),
                branch_id("main"),
                selected_node("gear"),
            ),
        ),
    );
    let selected_node_right = FoundationalDiagnosticLocator::Transition(
        FoundationalTransitionLocator::SelectedNodeScope(
            FoundationalSelectedNodeScopeLocator::new(
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

    let selected_aspect_left = FoundationalDiagnosticLocator::Transition(
        FoundationalTransitionLocator::SelectedAspectScope(
            FoundationalSelectedAspectScopeLocator::new(
                branch_id("feature"),
                branch_id("main"),
                selected_aspect("gear:teeth", "profile"),
            ),
        ),
    );
    let selected_aspect_right = FoundationalDiagnosticLocator::Transition(
        FoundationalTransitionLocator::SelectedAspectScope(
            FoundationalSelectedAspectScopeLocator::new(
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
    let selected_node_locator = FoundationalTransitionLocator::SelectedNodeScope(
        FoundationalSelectedNodeScopeLocator::new(
            branch_id("feature/geometry"),
            branch_id("main"),
            selected_node("gear"),
        ),
    );
    let selected_aspect_locator = FoundationalTransitionLocator::SelectedAspectScope(
        FoundationalSelectedAspectScopeLocator::new(
            branch_id("feature/geometry"),
            branch_id("main"),
            selected_aspect("gear", "teeth"),
        ),
    );

    let authority_path =
        FoundationalBoundaryEvidenceAuthorityPath::transition(selected_node_locator.clone());
    let receipt_boundary =
        FoundationalBoundaryEvidenceReceiptBoundary::transition(selected_aspect_locator.clone());

    assert_eq!(authority_path.locator(), &selected_node_locator);
    assert_eq!(receipt_boundary.locator(), &selected_aspect_locator);
}

fn locator_entries(locator: FoundationalTransitionLocator) -> Vec<CanonicalBasisEntry> {
    ready_locator(locator).payload().entries().to_vec()
}

fn ready_locator(locator: FoundationalTransitionLocator) -> CanonicalBasisReadyArtifact {
    match prepare_locator_for_canonical_basis(version(), CanonicalLocatorInput::Transition(locator))
    {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready scoped transition locator"),
    }
}

fn assert_mismatched(left: CanonicalBasisReadyArtifact, right: CanonicalBasisReadyArtifact) {
    let ready = match prepare_canonical_comparison(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected comparison readiness"),
    };
    assert!(matches!(
        worth_foundational::compare_canonical_basis(&ready),
        CanonicalComparisonOutcome::Mismatched(_)
    ));
}

fn transition_locator_text_entry(locus: &str, value: &str) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Locator,
        CanonicalBasisLocus::Named(locus.into()),
        CanonicalBasisEntryKind::TransitionLocator,
        CanonicalBasisValue::ExactText(value.into()),
    )
}

#[allow(dead_code)]
fn transition_locator_integer_entry(locus: &str, value: u64) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Locator,
        CanonicalBasisLocus::Named(locus.into()),
        CanonicalBasisEntryKind::TransitionLocator,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: u128::from(value),
        },
    )
}

fn version() -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new("milestone-9-phase-12").expect("version")
}
