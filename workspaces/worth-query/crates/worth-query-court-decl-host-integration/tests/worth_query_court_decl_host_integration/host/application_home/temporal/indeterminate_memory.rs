//! Memory-backed live conditional fate courts: effect-bearing post-Native
//! release and colliding host idempotency relations across distinct intents.
use super::indeterminate_live::try_cancel_intent_independently;
use super::*;
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation as temporal;

#[test]
fn released_post_native_original_on_memory_reports_released_and_never_no_effect() {
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        temporal::directory::memory_request().open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("memory temporal Home open: {refused:?}"));
    {
        let app = home.runtime();
        let installed = app.conditional::<TemporalConditional>().unwrap();
        let clock_handle = &installed.as_ref().as_ref().unwrap().clock;
        let selected = app
            .with_host_advancement(|phase| {
                app.on_branch(app.current_world()).select(&phase).unwrap()
            })
            .unwrap();
        let mut clock = selected.conditional_clock(&execution, clock_handle).unwrap();
        app.runtime()
            .world_operation_control_for_test()
            .panic_before_product_compare_once_for_test();
        app.runtime().lose_next_commit_answer_for_test();
        let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(first) =
            clock.observe(&execution)
        else {
            panic!("initial due intent observes");
        };
        assert_eq!(first.indeterminate_operation_count(), 1);
        drop(first);
        let page = app
            .runtime()
            .product_publication_recovery_page(None, std::num::NonZeroUsize::MIN)
            .unwrap();
        assert_eq!(page.rows().len(), 1);
        let recovery = app
            .runtime()
            .readmit_product_publication_recovery(page.rows()[0].handle())
            .unwrap();
        drop(page);
        app.runtime()
            .release_product_publication_recovery(recovery, 0)
            .unwrap();
        assert!(matches!(
            retry_original_intent_on_branch(&home, app.current_world(), &execution),
            primary_graph::WorthQueryConditionalOriginalResolution::Released
        ));
        let original_relation =
            temporal::domain::WorthQueryTemporalIntentIdempotencyRelation::declare(&(
                "intent-1",
                1_u64,
                "payload",
            ))
            .unwrap();
        assert!(matches!(
            clock
                .retry_original_temporal_intent(
                    &execution,
                    "intent-1",
                    2,
                    original_relation.as_str(),
                    "other-payload",
                )
                .unwrap(),
            primary_graph::WorthQueryConditionalOriginalResolution::OtherIntent
        ));
        // Because Native already moved ahead of World on the released original,
        // no ordinary Native edit can rewrite the candidate on this branch.
        assert!(matches!(
            try_cancel_intent_independently(&home, &execution),
            primary_graph::WorthQueryApplicationCommitOutcome::Denied(denial)
                if denial.kind() == primary_graph::WorthQueryApplicationCommitDenialKind::ProductBasisStale
        ));
        // Both live reentry and candidate-retirement share the single
        // `outcome_for_resolved_key_fate` owner, which maps `Released` to
        // `AlreadyCommitted` without inventing a World receipt or `NoEffect`.
        let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(retired) =
            clock.observe(&execution)
        else {
            panic!("released-original live re-observation observes");
        };
        assert_eq!(retired.committed_operation_count(), 0);
        assert_eq!(retired.already_committed_operation_count(), 1);
        assert_eq!(retired.execution_provenance().len(), 1);
        assert_eq!(
            retired.execution_provenance()[0].terminal(),
            primary_graph::WorthQueryConditionalExecutionTerminal::AlreadyCommitted,
            "effect-bearing live release must never report NoEffect on retirement"
        );
        assert!(retired.execution_provenance()[0].published_commit().is_none());
        assert_eq!(installed.as_ref().as_ref().unwrap().contacts.snapshot().3, 1);
    }
    drop(home);
}

#[test]
fn colliding_host_relation_refuses_foreign_intent_before_authored_invocation() {
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        temporal::directory::colliding_memory_request().open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("colliding memory temporal Home open: {refused:?}"));
    {
        let app = home.runtime();
        let installed = app.conditional::<TemporalConditional>().unwrap();
        let entry = installed.as_ref().as_ref().unwrap();
        let clock_handle = &entry.clock;
        let selected = app
            .with_host_advancement(|phase| {
                app.on_branch(app.current_world()).select(&phase).unwrap()
            })
            .unwrap();
        let mut clock = selected.conditional_clock(&execution, clock_handle).unwrap();

        // 1. At t=10 only intent-1 is due; post-Native unwind retains live
        // unpublished custody on the shared key.
        app.runtime()
            .world_operation_control_for_test()
            .panic_before_product_compare_once_for_test();
        app.runtime().lose_next_commit_answer_for_test();
        let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(first) =
            clock.observe(&execution)
        else {
            panic!("first due intent observes");
        };
        assert_eq!(first.indeterminate_operation_count(), 1);
        let (_, pre1, proj1, apply1) = entry.contacts.snapshot();
        assert_eq!((pre1, proj1, apply1), (1, 1, 1));
        drop(first);

        // 2. At t=20 intent-2 shares the host relation with a distinct root.
        // Pre-invocation fencing must refuse intent-2 as IdempotencyIntentDrift
        // without calling authored preconditions, projection, or apply.
        entry.clock_control.push(2, 20);
        let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(second) =
            clock.observe(&execution)
        else {
            panic!("colliding live unpublished observation runs");
        };
        let (_, pre2, proj2, apply2) = entry.contacts.snapshot();
        assert_eq!(
            (pre2, proj2, apply2),
            (1, 1, 1),
            "foreign intent against live unpublished custody must not invoke authored code"
        );
        let intent2 = second
            .execution_provenance()
            .iter()
            .find(|p| p.intent_identity() == "intent-2")
            .expect("intent-2 provenance recorded");
        assert_eq!(
            intent2.terminal(),
            primary_graph::WorthQueryConditionalExecutionTerminal::Failed
        );
        assert_eq!(
            intent2.cause(),
            Some(primary_graph::WorthQueryConditionalExecutionCause::TerminalFailure)
        );
        drop(second);

        // 3. Release intent-1's post-Native original. Explicit retry distinguishes
        // Released (intent-1) from OtherIntent (intent-2).
        let page = app
            .runtime()
            .product_publication_recovery_page(None, std::num::NonZeroUsize::MIN)
            .unwrap();
        assert_eq!(page.rows().len(), 1);
        let recovery = app
            .runtime()
            .readmit_product_publication_recovery(page.rows()[0].handle())
            .unwrap();
        drop(page);
        app.runtime()
            .release_product_publication_recovery(recovery, 0)
            .unwrap();
        let shared =
            temporal::domain::WorthQueryTemporalIntentIdempotencyRelation::declare(&"shared-relation")
                .unwrap();
        let retry_shared = |id: &str, input: &str| {
            let sel = app
                .with_host_advancement(|p| app.on_branch(app.current_world()).select(&p).unwrap())
                .unwrap();
            sel.conditional_clock(&execution, clock_handle)
                .unwrap()
                .retry_original_temporal_intent(&execution, id, 1, shared.as_str(), input)
                .unwrap()
        };
        assert!(matches!(
            retry_shared("intent-1", "shared:payload"),
            primary_graph::WorthQueryConditionalOriginalResolution::Released
        ));
        assert!(matches!(
            retry_shared("intent-2", "shared:other-payload"),
            primary_graph::WorthQueryConditionalOriginalResolution::OtherIntent
        ));

        // 4. At t=30 intent-3 collides against the live effect-bearing Released
        // entry: intent-1 resolves AlreadyCommitted (never NoEffect) and intent-3
        // is refused pre-invocation without calling authored code.
        entry.clock_control.push(3, 30);
        let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(third) =
            clock.observe(&execution)
        else {
            panic!("colliding live released observation runs");
        };
        let (_, pre3, proj3, apply3) = entry.contacts.snapshot();
        assert_eq!(
            (pre3, proj3, apply3),
            (1, 1, 1),
            "foreign intent against live released custody must not invoke authored code"
        );
        let intent1 = third
            .execution_provenance()
            .iter()
            .find(|p| p.intent_identity() == "intent-1")
            .expect("intent-1 provenance recorded");
        assert_eq!(
            intent1.terminal(),
            primary_graph::WorthQueryConditionalExecutionTerminal::AlreadyCommitted
        );
        assert!(intent1.published_commit().is_none());
        let intent3 = third
            .execution_provenance()
            .iter()
            .find(|p| p.intent_identity() == "intent-3")
            .expect("intent-3 provenance recorded");
        assert_eq!(
            intent3.terminal(),
            primary_graph::WorthQueryConditionalExecutionTerminal::Failed
        );
        assert_eq!(
            intent3.cause(),
            Some(primary_graph::WorthQueryConditionalExecutionCause::TerminalFailure)
        );
    }
    drop(home);
}
