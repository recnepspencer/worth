use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::application_query::observed_source::source_identity::membership::tests::TestsWorthQueryObservedSourceSelectionForTest;

use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::application_query::observed_source::source_identity::membership::tests::entity;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::application_query::observed_source::source_identity::membership::tests::footprint;

use worth_foundational::facade::CanonicalDigestId;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::membership::WorthQueryObservedSourceSelection;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::membership::same_footprint_membership;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::membership::same_text;
#[test]
fn selected_membership_ignores_revisions_but_not_selected_endpoints() {
    let before = footprint(vec![entity(2)], 4);
    let same_selection = footprint(vec![entity(2)], 9);
    let changed_selection = footprint(vec![entity(3)], 9);
    assert!(same_footprint_membership(&before, &same_selection));
    assert!(!same_footprint_membership(&before, &changed_selection,));
}
#[test]
fn retained_selection_compares_a_fresh_meaning_without_revision_identity() {
    let authority = worth_query_execution::facade::domain_computation::WorthQueryRuntimeAuthorityIdentity::mint_for_test();
    let registry = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry::new(authority);
    let selection = worth_query_execution::facade::domain_computation::WorthQueryApplicationBasisSelectionIdentity::Relational;
    let meaning = |endpoints, revision| {
        registry
            .intern_for_test(
                &[7; 32],
                &[8; 32],
                footprint(endpoints, revision),
                &selection,
            )
            .unwrap()
    };
    let binding = ApplicationSchemaBindingIdentity::from_installed_parts(
        1,
        2,
        CanonicalDigestId::new([3; 32]),
        CanonicalDigestId::new([4; 32]),
    );
    let carrier = |meaning| {
        WorthQueryObservedSourceSelection::new_for_test(
            authority.as_u64(),
            binding.clone(),
            meaning,
            None,
        )
    };
    let before = carrier(meaning(vec![entity(2)], 4));
    let revision_only = carrier(meaning(vec![entity(2)], 9));
    let different = carrier(meaning(vec![entity(3)], 9));
    assert!(before
        .same_selected_membership_as_for_test(&revision_only, &mut |_| Ok(()))
        .unwrap());
    assert!(!before
        .same_selected_membership_as_for_test(&different, &mut |_| Ok(()))
        .unwrap());
}
#[test]
fn selection_comparison_requires_complete_membership() {
    let selected = footprint(vec![entity(2), entity(3)], 4);
    assert!(same_footprint_membership(&selected, &selected));
    let mut incomplete = selected.clone();
    incomplete.root_selection = None;
    assert!(!same_footprint_membership(&selected, &incomplete));
}
#[test]
fn text_comparison_reads_initialized_utf8_not_spare_capacity() {
    let mut spare = String::with_capacity(4096);
    spare.push_str("open");
    // Stale bytes past the initialized length stay in the spare capacity.
    let mut truncated = String::from("openXYZ");
    truncated.truncate("open".len());
    for left in [spare.as_str(), truncated.as_str()] {
        for right in ["open", "openXYZ", "opal", "closed", ""] {
            assert_eq!(same_text(left, right), left == right, "{left:?} {right:?}");
        }
    }
    assert!(same_text(&spare, &truncated));
}
#[test]
fn selected_membership_propagates_owned_admission_and_final_poll_stops() {
    let selected = WorthQueryObservedSourceSelection::selecting_for_test(1);
    let mut calls = 0;
    let stopped = selected.same_selected_membership_as_for_test(&selected, &mut |_| {
        calls += 1;
        Err(Cause::Cancelled)
    });
    assert_eq!(stopped, Err(Cause::Cancelled));
    assert_eq!(calls, 1);
    let last = selected.same_selected_membership_as_for_test(&selected, &mut |work| {
        if work == 0 {
            Err(Cause::DeadlineElapsed)
        } else {
            Ok(())
        }
    });
    assert_eq!(last, Err(Cause::DeadlineElapsed));
}
