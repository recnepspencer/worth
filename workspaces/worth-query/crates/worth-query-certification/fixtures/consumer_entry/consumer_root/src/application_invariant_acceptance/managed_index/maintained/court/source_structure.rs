//! A real warm Source update must preserve structure before any invariant callback.
use super::*;
use prefix::structure::Fault;
#[test]
fn malformed_warm_structure_preserves_predecessor_and_stable_slot_retry_commits() {
    lifecycle::run(journey);
}
fn journey() {
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
    let index = request
        .query(find("A", "sh"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    let other = request
        .query(find("B", ""))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    let builds = provider.builds.load(Ordering::SeqCst);
    let selected = request.query(find("A", "sh")).execute().unwrap();
    let before_rows = selected.rows().to_vec();
    let observation = selected.into_result_set_observation().unwrap();
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    rules.records.lock().unwrap().clear();
    let memory = accounted_request();
    let baseline = memory.charged_memory_bytes();
    *provider.structure_fault.lock().unwrap() = Fault::OmitSource;
    let denied = request
        .advanced()
        .mutate(rename("sh", "free"))
        .expect_result_set(observation.clone())
        .idempotency(&0x919401u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::Execution(&memory))
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = denied
    else {
        panic!("bad prepared structure cannot publish: {denied:?}");
    };
    let native = denial
        .invariant_execution_failure()
        .unwrap()
        .managed_source_denial()
        .unwrap();
    let mut cause: &(dyn std::error::Error + 'static) = native.provider_failure().unwrap().cause();
    loop {
        if let Some(cause) = cause.downcast_ref::<WorthQueryManagedIndexWorkDenial>() {
            assert!(matches!(
                cause,
                WorthQueryManagedIndexWorkDenial::Controlled(
                    WorthQueryManagedIndexStop::SourceStructureIncomplete
                )
            ));
            break;
        }
        cause = cause
            .source()
            .expect("retain the actual typed structure refusal");
    }
    drop(denial);
    assert!(rules.records.lock().unwrap().is_empty());
    assert_eq!(memory.charged_memory_bytes(), baseline);
    assert_eq!(provider.builds.load(Ordering::SeqCst), builds);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(
        values(request.query(find("A", "")).execute().unwrap().rows()),
        values(&before_rows)
    );
    *provider.structure_fault.lock().unwrap() = Fault::None;
    let fresh_scope = authentication::request_scope();
    let fresh = world
        .runtime()
        .request(&principal, &fresh_scope)
        .on_branch(world.current_world());
    let outcome = fresh
        .advanced()
        .mutate(rename("sh", "free"))
        .expect_result_set(observation)
        .idempotency(&0x919402u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
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
        "warm structure never scans a rebuild"
    );
    index.close();
    other.close();
}
