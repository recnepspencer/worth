//! Query readmission reconstructs cold ordered generations; old cursors stay stale.
use std::{num::NonZeroUsize, time::Duration};

use worth_relational::facade::indexes::{
    BoundedIndexParityMode, BoundedRelatedEntityOrderedLookupDenialKind,
    BoundedRelatedEntityOrderedLookupRequest, DerivedIndexDiscardRequest,
};

use super::support::{ContinuationTestContext, TestContinuation};
use crate::domain_computation::primary_graph::{
    application_query::{
        WorthQueryApplicationContinuationDenialKind, WorthQueryApplicationQueryAccessContext,
        WorthQueryApplicationQueryResumeControls,
    },
    tests::fixture::{live_account_parameters, live_scope, Activity},
    WorthQueryProductQueryControls,
};

#[test]
fn discarded_ordered_generation_rebuilds_without_reviving_old_query_cursors() {
    let context = ContinuationTestContext::new(Duration::from_secs(60));
    let old_cursor = context.issue();
    let cold_cursor = context.issue();
    assert_eq!(old_cursor.index_generation, cold_cursor.index_generation);
    let index_id = old_cursor.index_id;
    let basis = old_cursor.product.relational_basis().clone();
    let graph = &context.world.application.primary_provider.graph;
    let child_kind = context
        .world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout
        .entity_kind(Activity::reference().name())
        .unwrap();
    let original = graph.with_runtime(|runtime| {
        let original = runtime
            .index_access()
            .published_generation_for_observation(index_id, &basis.observation())
            .unwrap();
        assert_eq!(original.generation_id, old_cursor.index_generation);
        let definitions = runtime
            .index_access()
            .definition_lookup_snapshot()
            .definition_count();
        let discarded = runtime
            .index_authority()
            .discard_generations(DerivedIndexDiscardRequest::all_bases(index_id))
            .unwrap();
        assert_eq!(discarded.index_id(), index_id);
        assert!(discarded.removed_generation_count() > 0);
        assert_eq!(
            runtime
                .index_access()
                .definition_lookup_snapshot()
                .definition_count(),
            definitions
        );
        // Check the native cold boundary before any Query readmission can warm indexes.
        let snapshot = runtime
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
        let lookup = BoundedRelatedEntityOrderedLookupRequest::new(
            snapshot.clone(),
            index_id,
            old_cursor.scope_entity_id,
            child_kind,
            None,
            1,
        )
        .unwrap()
        .expect_generation(old_cursor.index_generation);
        let denied = runtime
            .index_access()
            .execute_bounded_related_entity_ordered_lookup(
                lookup,
                BoundedIndexParityMode::Production,
            )
            .unwrap_err();
        assert_eq!(
            denied.kind(),
            BoundedRelatedEntityOrderedLookupDenialKind::ExactGenerationUnavailable
        );
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
        original
    });
    context.assert_resource_baseline();
    assert_resume_denial(
        &context,
        cold_cursor,
        WorthQueryApplicationContinuationDenialKind::ContinuationGenerationChanged,
    );

    // Ordinary Query admission owns reconstruction at the retained exact basis.
    // No test-issued build may stand in for that downstream handoff.
    let generation = graph.with_runtime(|runtime| {
        runtime
            .index_access()
            .published_generation_for_observation(index_id, &basis.observation())
            .expect("ordinary readmission rebuilt the discarded ordered generation")
    });
    assert_ne!(generation.generation_id, original.generation_id);
    assert_eq!(generation.source_commit_id, original.source_commit_id);
    assert_eq!(generation.applicability, original.applicability);
    assert_eq!(generation.entries, original.entries);
    assert_resume_denial(
        &context,
        old_cursor,
        WorthQueryApplicationContinuationDenialKind::ContinuationGenerationChanged,
    );
    graph.with_runtime(|runtime| {
        assert_eq!(
            runtime
                .index_access()
                .published_generation_for_observation(index_id, &basis.observation())
                .unwrap()
                .generation_id,
            generation.generation_id,
            "a subsequent readmission reuses the current rebuilt generation"
        );
    });
    assert_fresh_pages(&context);
    context.assert_resource_baseline();
}

fn assert_resume_denial(
    context: &ContinuationTestContext,
    cursor: TestContinuation,
    expected: WorthQueryApplicationContinuationDenialKind,
) {
    let request = live_scope();
    let access = WorthQueryApplicationQueryAccessContext::new(&context.principal, &context.account);
    let plan = context
        .world
        .application
        .readmit_application_query_continuation(
            &context.query,
            &access,
            live_account_parameters("account-1"),
            cursor,
            WorthQueryApplicationQueryResumeControls::new(one(), work(), &request),
        )
        .unwrap();
    let denied = context
        .world
        .application
        .execute_application_query_continuation_page(plan)
        .err()
        .expect("no page may be returned from the missing or replaced generation");
    assert_eq!(denied.kind(), expected);
    context.assert_resource_baseline();
}

fn assert_fresh_pages(context: &ContinuationTestContext) {
    let request = live_scope();
    let access = WorthQueryApplicationQueryAccessContext::new(&context.principal, &context.account);
    let plan = context
        .world
        .selected_product()
        .admit_application_query_continuation(
            &context.query,
            &access,
            live_account_parameters("account-1"),
            WorthQueryProductQueryControls::new(one(), work(), &request),
        )
        .unwrap();
    let first = context
        .world
        .application
        .execute_application_query_continuation_page(plan)
        .unwrap();
    let (rows, cursor, receipt) = first.into_parts();
    assert!(receipt.basis_released());
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].account(), "account-1");
    assert_eq!(rows[0].activities(), &[("activity-primary".to_owned(), 11)]);
    let plan = context
        .world
        .application
        .readmit_application_query_continuation(
            &context.query,
            &access,
            live_account_parameters("account-1"),
            cursor.expect("one ordered activity remains"),
            WorthQueryApplicationQueryResumeControls::new(one(), work(), &request),
        )
        .unwrap();
    let last = context
        .world
        .application
        .execute_application_query_continuation_page(plan)
        .unwrap();
    assert!(last.receipt().basis_released());
    assert!(last.continuation().is_none());
    assert_eq!(last.rows().len(), 1);
    assert_eq!(last.rows()[0].account(), "account-1");
    assert_eq!(
        last.rows()[0].activities(),
        &[("activity-secondary".to_owned(), 22)]
    );
}

fn one() -> NonZeroUsize {
    NonZeroUsize::new(1).unwrap()
}

fn work() -> NonZeroUsize {
    NonZeroUsize::new(10_000).unwrap()
}
