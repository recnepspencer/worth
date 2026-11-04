//! Query readmission reconstructs cold ordered generations; old cursors stay stale.
use std::num::NonZeroUsize;
use std::time::Duration;

use worth_relational::facade::indexes::BoundedIndexParityMode;
use worth_relational::facade::indexes::BoundedRelatedEntityOrderedLookupDenialKind;
use worth_relational::facade::indexes::BoundedRelatedEntityOrderedLookupRequest;
use worth_relational::facade::indexes::DerivedIndexDiscardRequest;

use crate::domain_computation::primary_graph::application_query::continuation::tests::support::ContinuationTestContext;
use crate::domain_computation::primary_graph::application_query::continuation::tests::support::TestContinuation;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationContinuationDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryResumeControls;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::live_account_query::live_account_parameters;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Activity;
use worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls;

#[test]
fn discarded_ordered_generation_rebuilds_without_reviving_old_query_cursors() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let context = ContinuationTestContext::new(Duration::from_secs(60));
    let old_cursor = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            context.issue(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let cold_cursor = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            context.issue(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!((*old_cursor.index_generation_for_test()), (*cold_cursor.index_generation_for_test()));
    let index_id = (*old_cursor.index_id_for_test());
    let basis = (*old_cursor.product_for_test()).relational_basis_for_test().clone();
    let graph = &(*(*context.world.application.primary_provider_for_test()).graph_for_test());
    let child_kind = (*(*context
        .world
        .application
        .runtime_for_test())
        .primary_graph()
        .unwrap()
        .layout_for_test())
        .entity_kind_for_test(Activity::reference().name())
        .unwrap();
    let original = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            graph.with_runtime(request_execution, |runtime| {
                let original = runtime
                    .index_access()
                    .read_published_generation_for_observation(
                        request_execution,
                        index_id,
                        &basis.observation(),
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(original.generation_id, (*old_cursor.index_generation_for_test()));
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
                    (*old_cursor.scope_entity_id_for_test()),
                    child_kind,
                    None,
                    1,
                )
                .unwrap()
                .expect_generation((*old_cursor.index_generation_for_test()));
                let denied = runtime
                    .index_access()
                    .execute_bounded_related_entity_ordered_lookup(
                        lookup,
                        BoundedIndexParityMode::Production,
                        &host_custody.borrowed_request_for_test(),
                    )
                    .unwrap_err();
                assert_eq!(
                    denied.kind(),
                    BoundedRelatedEntityOrderedLookupDenialKind::ExactGenerationUnavailable
                );
                runtime.snapshots().release_snapshot(&snapshot).unwrap();
                original
            })
        })
        .expect("the fixture read owns its installed host call");
    context.assert_resource_baseline();
    context
        .world
        .application
        .with_host_advancement(|phase| {
            assert_resume_denial(
                &phase,
                &context,
                cold_cursor,
                WorthQueryApplicationContinuationDenialKind::ContinuationGenerationChanged,
            )
        })
        .expect("the fixture read owns its installed host call");

    // Ordinary Query admission owns reconstruction at the retained exact basis.
    // No test-issued build may stand in for that downstream handoff.
    let generation = graph.with_runtime(request_execution, |runtime| {
        runtime
            .index_access()
            .read_published_generation_for_observation(
                request_execution,
                index_id,
                &basis.observation(),
            )
            .unwrap()
            .expect("ordinary readmission rebuilt the discarded ordered generation")
    });
    assert_ne!(generation.generation_id, original.generation_id);
    assert_eq!(generation.source_commit_id, original.source_commit_id);
    assert_eq!(generation.applicability, original.applicability);
    assert_eq!(generation.entries, original.entries);
    context
        .world
        .application
        .with_host_advancement(|phase| {
            assert_resume_denial(
                &phase,
                &context,
                old_cursor,
                WorthQueryApplicationContinuationDenialKind::ContinuationGenerationChanged,
            )
        })
        .expect("the fixture read owns its installed host call");
    graph.with_runtime(request_execution, |runtime| {
        assert_eq!(
            runtime
                .index_access()
                .read_published_generation_for_observation(
                    request_execution,
                    index_id,
                    &basis.observation()
                )
                .unwrap()
                .unwrap()
                .generation_id,
            generation.generation_id,
            "a subsequent readmission reuses the current rebuilt generation"
        );
    });
    context
        .world
        .application
        .with_host_advancement(|phase| assert_fresh_pages(&phase, &context))
        .expect("the fixture read owns its installed host call");
    context.assert_resource_baseline();
}

pub fn assert_resume_denial(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
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
            phase_custody,
            &context.query,
            &access,
            live_account_parameters("account-1"),
            cursor,
            WorthQueryApplicationQueryResumeControls::new(one(), &request),
        )
        .unwrap();
    let denied = context
        .world
        .application
        .execute_application_query_continuation_page(phase_custody, plan)
        .err()
        .expect("no page may be returned from the missing or replaced generation");
    assert_eq!(denied.kind(), expected);
    context.assert_resource_baseline();
}

pub fn assert_fresh_pages(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    context: &ContinuationTestContext,
) {
    let request = live_scope();
    let access = WorthQueryApplicationQueryAccessContext::new(&context.principal, &context.account);
    let plan = context
        .world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .admit_application_query_continuation(
            phase_custody,
            &context.query,
            &access,
            live_account_parameters("account-1"),
            WorthQueryProductQueryControls::new(one(), &request),
        )
        .unwrap();
    let first = context
        .world
        .application
        .execute_application_query_continuation_page(phase_custody, plan)
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
            phase_custody,
            &context.query,
            &access,
            live_account_parameters("account-1"),
            cursor.expect("one ordered activity remains"),
            WorthQueryApplicationQueryResumeControls::new(one(), &request),
        )
        .unwrap();
    let last = context
        .world
        .application
        .execute_application_query_continuation_page(phase_custody, plan)
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

pub fn one() -> NonZeroUsize {
    NonZeroUsize::new(1).unwrap()
}
