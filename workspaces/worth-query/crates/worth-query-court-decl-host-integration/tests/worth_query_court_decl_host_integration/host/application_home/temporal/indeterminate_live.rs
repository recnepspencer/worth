//! C3.3 / R9 live uncertainty preservation: a durably retained
//! `Indeterminate(Unclassified)` conditional decision is never overwritten by
//! a retryable failure or finished by a candidate change while World has not
//! resolved the original key.
use super::*;
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation as temporal;

#[test]
fn indeterminate_decision_survives_live_retryable_failure_until_world_resolves() {
    let location = fixture::location();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(&location, &execution);
    let runtime = home.runtime().runtime();
    let rejection = runtime
        .reject_next_commit_before_transaction_for_test()
        .unwrap();
    runtime.lose_next_commit_answer_for_test();
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(first) =
        observe(&home, &execution)
    else {
        panic!("initial due intent executes");
    };
    assert_eq!(first.indeterminate_operation_count(), 1);
    drop((first, rejection));
    let original = home.runtime().conditional_decision_obligations();
    let [row] = original.as_slice() else {
        panic!("one retained indeterminate decision: {original:?}");
    };
    assert_eq!(
        row.posture,
        primary_graph::QueryConditionalPosture::Indeterminate(
            primary_graph::QueryConditionalOriginal::Unclassified
        )
    );

    // Live retry fails with a retryable precondition error: uncertainty stays.
    {
        let installed = home
            .runtime()
            .conditional::<TemporalConditional>()
            .unwrap();
        installed
            .as_ref()
            .as_ref()
            .unwrap()
            .preconditions_panic
            .as_ref()
            .unwrap()
            .set(true);
    }
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(retry) =
        observe(&home, &execution)
    else {
        panic!("live retry observes");
    };
    assert_eq!(retry.failed_operation_count(), 1);
    drop(retry);
    assert_eq!(
        home.runtime().conditional_decision_obligations(),
        original,
        "a live retryable failure never overwrites Indeterminate(Unclassified)"
    );
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("Home closes with indeterminate row: {refused:?}"))
        .reclaim();

    // Reopen lets World classify the still-intact indeterminate row as Absent
    // (proven no-effect), then executes once cleanly.
    let home = open(&location, &execution);
    assert!(
        home.runtime().conditional_decision_obligations().is_empty(),
        "cold classification finished the proven no-effect row"
    );
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(resumed) =
        observe(&home, &execution)
    else {
        panic!("reopened clock observes");
    };
    assert_eq!(resumed.committed_operation_count(), 1);
    assert!(resumed.execution_provenance()[0].published_commit().is_some());
    drop(resumed);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("settled Home closes: {refused:?}"))
        .reclaim();
}

#[test]
fn indeterminate_decision_resolves_through_world_on_unpublished_and_candidate_retirement() {
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    // 1. Live unpublished original: unwind before product compare + lost answer
    // retains Indeterminate(Unclassified) across live retries, joins World on
    // reopen, and continues the original occurrence once.
    let location_unpub = fixture::location();
    let home = open(&location_unpub, &execution);
    let runtime = home.runtime().runtime();
    runtime
        .world_operation_control_for_test()
        .panic_before_product_compare_once_for_test();
    runtime.lose_next_commit_answer_for_test();
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(first) =
        observe(&home, &execution)
    else {
        panic!("initial due intent observes");
    };
    assert_eq!(first.indeterminate_operation_count(), 1);
    drop(first);
    let original = home.runtime().conditional_decision_obligations();
    assert_eq!(original.len(), 1);
    assert_eq!(
        original[0].posture,
        primary_graph::QueryConditionalPosture::Indeterminate(
            primary_graph::QueryConditionalOriginal::Unclassified
        )
    );
    let contacts_after_first = home
        .runtime()
        .conditional::<TemporalConditional>()
        .unwrap()
        .as_ref()
        .as_ref()
        .unwrap()
        .contacts
        .snapshot()
        .3;
    assert_eq!(contacts_after_first, 1);
    assert!(matches!(
        retry_original_intent_on_branch(&home, home.runtime().current_world(), &execution),
        primary_graph::WorthQueryConditionalOriginalResolution::Unresolved
    ));
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(live_retry) =
        observe(&home, &execution)
    else {
        panic!("live retry observes");
    };
    assert_eq!(live_retry.indeterminate_operation_count(), 2);
    drop(live_retry);
    assert_eq!(
        home.runtime()
            .conditional::<TemporalConditional>()
            .unwrap()
            .as_ref()
            .as_ref()
            .unwrap()
            .contacts
            .snapshot()
            .3,
        contacts_after_first,
        "live unpublished original fences fresh authored invocation"
    );
    assert_eq!(
        home.runtime().conditional_decision_obligations(),
        original,
        "live retry never erases Indeterminate while an unpublished original is live"
    );
    drop(home);
    let home = open(&location_unpub, &execution);
    let reopened = home.runtime().conditional_decision_obligations();
    assert_eq!(reopened.len(), 1);
    assert!(matches!(
        reopened[0].posture,
        primary_graph::QueryConditionalPosture::Indeterminate(
            primary_graph::QueryConditionalOriginal::Recovered(_)
        )
    ));
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(continued) =
        observe(&home, &execution)
    else {
        panic!("reopened clock continues recovered original");
    };
    assert_eq!(continued.already_committed_operation_count(), 1);
    assert!(continued.execution_provenance()[0].published_commit().is_some());
    drop(continued);
    assert!(home.runtime().conditional_decision_obligations().is_empty());
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("continued Home closes: {refused:?}"))
        .reclaim();

    // 2. Published original whose answer was lost: live re-observation on the
    // retained clock binding sees the intent already advanced, consults World,
    // finishes the durable Indeterminate row, and returns the exact original
    // published occurrence (matching retry_original_intent_on_branch).
    let location_pub = fixture::location();
    let home = open(&location_pub, &execution);
    let published_commit = {
        let app = home.runtime();
        let installed = app.conditional::<TemporalConditional>().unwrap();
        let clock_handle = &installed.as_ref().as_ref().unwrap().clock;
        let selected = app
            .with_host_advancement(|phase| {
                app.on_branch(app.current_world()).select(&phase).unwrap()
            })
            .unwrap();
        let mut clock = selected.conditional_clock(&execution, clock_handle).unwrap();
        app.runtime().lose_next_commit_answer_for_test();
        let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(lost) =
            clock.observe(&execution)
        else {
            panic!("published lost-answer intent observes");
        };
        assert_eq!(lost.indeterminate_operation_count(), 1);
        drop(lost);
        assert_eq!(app.conditional_decision_obligations().len(), 1);
        let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(resolved) =
            clock.observe(&execution)
        else {
            panic!("live re-observation resolves published original");
        };
        assert_eq!(resolved.already_committed_operation_count(), 1);
        let commit = *resolved.execution_provenance()[0]
            .published_commit()
            .expect("candidate retirement after publication must return World's original occurrence");
        drop(resolved);
        assert!(app.conditional_decision_obligations().is_empty());
        commit
    };
    let primary_graph::WorthQueryConditionalOriginalResolution::Published(receipt) =
        retry_original_intent_on_branch(&home, home.runtime().current_world(), &execution)
    else {
        panic!("original intent retry must return the published original receipt");
    };
    assert_eq!(
        receipt.new_product_head().durable().unwrap().selected_commit(),
        published_commit,
    );
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("published resolved Home closes: {refused:?}"))
        .reclaim();

    // 3. No-effect indeterminate whose candidate is independently cancelled:
    // live re-observation on the retained clock binding consults World
    // (Fate::Absent), finishes the obligation, and never executes fresh.
    let location_cancel = fixture::location();
    let home = open(&location_cancel, &execution);
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
        let rejection = app
            .runtime()
            .reject_next_commit_before_transaction_for_test()
            .unwrap();
        app.runtime().lose_next_commit_answer_for_test();
        let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(no_effect) =
            clock.observe(&execution)
        else {
            panic!("no-effect indeterminate observes");
        };
        assert_eq!(no_effect.indeterminate_operation_count(), 1);
        drop((no_effect, rejection));
        cancel_intent_independently(&home, &execution);
        let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(retired) =
            clock.observe(&execution)
        else {
            panic!("retired candidate re-observes");
        };
        assert_eq!(retired.committed_operation_count(), 0);
        assert_eq!(retired.already_committed_operation_count(), 0);
        assert_eq!(retired.execution_provenance().len(), 1);
        assert_eq!(
            retired.execution_provenance()[0].terminal(),
            primary_graph::WorthQueryConditionalExecutionTerminal::NoEffect,
            "proved no-effect candidate retirement must not report AlreadyCommitted"
        );
        assert!(retired.execution_provenance()[0].published_commit().is_none());
        drop(retired);
        assert!(app.conditional_decision_obligations().is_empty());
    }
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("cancelled Home closes: {refused:?}"))
        .reclaim();
}

fn cancel_intent_independently(
    home: &Home,
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    use primary_graph::WorthQueryPrincipalResolutionMode::Ordinary;
    use temporal::*;
    use worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation;
    let app = home.runtime().runtime();
    let invariant = Arc::clone(&home.runtime().conditional::<TemporalConditional>().unwrap().as_ref().as_ref().unwrap().invariant);
    let schema = app.installed_schema();
    let request = request_scope();
    let external = block_on(admit_identity_adapter(schema).authenticate((), &request)).unwrap();
    let branch = app.current_world();
    let (intent, mut effects) = app.with_host_advancement(|p| {
        let selected = app.on_branch(branch).select(&p).unwrap();
        let principal = selected.resolve_authenticated_principal(&p, &schema.principal_binding(TemporalPrincipalBinding::reference()).unwrap(), &external, &request, Ordinary).unwrap();
        let intent = selected.resolve_entity(&p, IntentIdentityField::at("IntentIdentityField"), "intent-1".to_string(), &request, Ordinary).unwrap();
        let op = schema.installed_operation(AmendTemporal::reference()).unwrap();
        let admission = selected.authorize_operation(&p, &principal, &intent, &op, Default::default(), &request).unwrap();
        let (_, projection, _) = invariant.project_admitted_operation(&p, &admission, |reader, scope| {
            reader.decision_field(scope, IntentRevisionField::at("IntentRevisionField")).unwrap();
            reader.decision_field(scope, IntentLifecycleField::at("IntentLifecycleField")).unwrap();
            reader.decision_field(scope, IntentGateField::at("IntentGateField")).unwrap();
        }, SystemAllocation).unwrap().into_parts();
        let effects = app.begin_projected_application_read_attempt(&p, admission, projection, SystemAllocation).unwrap()
            .complete_projected_dependencies(&p, SystemAllocation).unwrap().begin_effect_program();
        (intent, effects)
    }).unwrap();
    let target = effects.existing_entity(&intent).unwrap();
    effects.write_field(&target, IntentRevisionField::at("IntentRevisionField"), 2_u64).unwrap();
    effects.write_field(&target, IntentLifecycleField::at("IntentLifecycleField"), "cancelled".to_string()).unwrap();
    let idempotency = primary_graph::WorthQueryApplicationIdempotencyBinding::for_host_commit::<TemporalHostSchema, AmendTemporal, _, _>(
        &("independent-cancel", 1_u8), &("independent-cancel", 1_u8),
    ).unwrap();
    let outcome = app.on_branch(branch).transaction()
        .apply(product::WorthQueryAdmittedChange::new(effects.finish().unwrap(), idempotency))
        .commit_for_program(request_execution, home.runtime().admit_program_operation::<AmendTemporal>().unwrap())
        .unwrap();
    assert!(matches!(outcome, primary_graph::WorthQueryApplicationCommitOutcome::Committed(_)), "independent cancellation commits: {outcome:?}");
}
