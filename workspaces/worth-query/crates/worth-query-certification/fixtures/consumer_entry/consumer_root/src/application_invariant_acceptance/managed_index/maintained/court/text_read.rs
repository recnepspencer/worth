//! Actual original-payer cleanup after populated text reuse and semantic/cancelled denial.
use super::*;
#[test]
fn decoded_text_reuses_nested_loans_refuses_nominal_hits_and_releases_original_payer() {
    lifecycle::run(|| {
        journey(false);
        journey(true);
    });
}
fn journey(cancel: bool) {
    let provider = Arc::new(prefix::Calls::default());
    let rules = Arc::new(rule::Calls::new(provider.clone()));
    let world = host::install(provider.clone(), rules.clone());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let a = request
        .query(find("A", "sh"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    let b = request
        .query(find("B", ""))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    let source = request.query(find("A", "sh")).execute().unwrap();
    let observation = source.into_result_set_observation().unwrap();
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let builds = provider.builds.load(Ordering::SeqCst);
    rules.records.lock().unwrap().clear();
    let memory = accounted_request();
    let baseline = memory.charged_memory_bytes();
    let charged = Arc::new(Mutex::new(None));
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    provider.inspect_text.store(true, Ordering::SeqCst);
    if cancel {
        let seen = charged.clone();
        let original = memory.clone();
        *provider.after_text.lock().unwrap() = Some(Box::new(move || {
            *seen.lock().unwrap() = Some(original.charged_memory_bytes());
            cancellation.cancel();
        }));
    } else {
        provider.wrong_text_binding.store(true, Ordering::SeqCst);
    }
    let original = world
        .runtime()
        .request(&principal, &stopped)
        .on_branch(world.current_world());
    let outcome = original
        .advanced()
        .mutate(rename("sh", "free"))
        .expect_result_set(observation.clone())
        .idempotency(&0x919711u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::Execution(&memory))
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("{outcome:?}");
    };
    if cancel {
        assert_eq!(
            denial.invariant_execution_failure().unwrap().kind(),
            WorthQueryInvariantExecutionDenialKind::RequestInterrupted(
                RelationalOperationInterruption::Cancelled
            )
        );
        assert!(charged
            .lock()
            .unwrap()
            .is_some_and(|bytes| bytes > baseline));
    } else {
        assert!(
            denial
                .invariant_execution_failure()
                .unwrap()
                .managed_source_denial()
                .is_some(),
            "{denial:?}"
        );
    }
    drop(denial);
    assert!(
        provider.text_reads.load(Ordering::SeqCst) > 0,
        "real text populated and nested reuse observed before refusal"
    );
    assert!(rules.records.lock().unwrap().is_empty());
    assert_eq!(
        memory.charged_memory_bytes(),
        baseline,
        "memo, backing header, inline array and original parse reservations release on denial"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(
        values(request.query(find("B", "")).execute().unwrap().rows()),
        expected("B", "shared", "")
    );
    provider.wrong_text_binding.store(false, Ordering::SeqCst);
    let fresh_scope = authentication::request_scope();
    let fresh = world
        .runtime()
        .request(&principal, &fresh_scope)
        .on_branch(world.current_world());
    let paid = accounted_request();
    let result = fresh
        .advanced()
        .mutate(rename("sh", "free"))
        .expect_result_set(observation)
        .idempotency(&0x919711u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::Execution(&paid))
        .unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
    for key in ["A", "B"] {
        assert_eq!(
            values(fresh.query(find(key, "")).execute().unwrap().rows()),
            expected(key, "free", "")
        );
    }
    assert_eq!(
        provider.builds.load(Ordering::SeqCst),
        builds,
        "same-slot before/after rename is maintained"
    );
    a.close();
    b.close();
}
