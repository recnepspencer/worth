//! Real changed Source values must requalify summaries even without a Node write.
use super::*;

#[test]
fn checked_summary_rejects_unchanged_root_for_changed_source_then_commits_lawful_twin() {
    lifecycle::run(journey);
}
fn journey() {
    let calls = Arc::new(provider::Calls::default());
    let config = Configuration::new(calls.clone());
    let rules = config.rules.clone();
    let prefixes = Arc::new(prefix::Calls::default());
    let world = host::install_geometry(
        prefixes.clone(),
        Arc::new(prefix_rule::Calls::new(prefixes)),
        config,
    );
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    setup(&world, &principal, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let index = request
        .query(all("A"))
        .open_index::<provider::BoxIndex>()
        .unwrap();
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let before = values(request.query(all("A")).execute().unwrap().rows());
    rules.records.lock().unwrap().clear();
    calls.preserve_root.store(true, Ordering::SeqCst);
    let memory = accounted_request();
    let baseline = memory.charged_memory_bytes();
    let result = request
        .advanced()
        .mutate(Edit::Move {
            directory: "A".into(),
            vertex: "v1".into(),
            x: 30,
            y: 0,
        })
        .idempotency(&0x919601u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::Execution(&memory))
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = result
    else {
        panic!("an unchanged root cannot represent changed Source: {result:?}");
    };
    let native = denial
        .invariant_execution_failure()
        .unwrap()
        .managed_source_denial()
        .unwrap();
    let mut cause: &(dyn std::error::Error + 'static) = native.provider_failure().unwrap().cause();
    loop {
        if let Some(cause) = cause.downcast_ref::<WorthQueryManagedIndexWorkDenial>() {
            assert!(
                matches!(
                    cause,
                    WorthQueryManagedIndexWorkDenial::Controlled(
                        WorthQueryManagedIndexStop::SummaryMismatch
                    )
                ),
                "{cause:?}"
            );
            break;
        }
        cause = cause
            .source()
            .expect("retain the actual summary qualification cause");
    }
    drop(denial);
    assert!(rules.records.lock().unwrap().is_empty());
    assert_eq!(memory.charged_memory_bytes(), baseline);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(
        values(request.query(all("A")).execute().unwrap().rows()),
        before
    );
    calls.preserve_root.store(false, Ordering::SeqCst);
    let fresh_scope = authentication::request_scope();
    commit(
        &world,
        &principal,
        &fresh_scope,
        Edit::Move {
            directory: "A".into(),
            vertex: "v1".into(),
            x: 30,
            y: 0,
        },
        0x919602,
    );
    let fresh = world
        .runtime()
        .request(&principal, &fresh_scope)
        .on_branch(world.current_world());
    for directory in ["A", "B"] {
        for x in [15, 30, 50, 80] {
            let probe = find(directory, x);
            assert_eq!(
                values(fresh.query(probe.clone()).execute().unwrap().rows()),
                expected(directory, 30, probe.probe)
            );
        }
    }
    index.close();
}
