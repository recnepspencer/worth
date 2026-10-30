//! Current canonical cache eviction, real cold reconstruction, and unchanged Source custody.
use super::super::refresh;
use super::*;
mod cancellation;
#[test]
fn cold_scope_reconstructs_before_rules_and_preserves_unchanged_source_expectations() {
    lifecycle::isolated_allocation_case(
        run,
        concat!(
            module_path!(),
            "::cold_scope_reconstructs_before_rules_and_preserves_unchanged_source_expectations"
        ),
    );
}
fn run() {
    journey();
    let region = Region::new(lifecycle::ALLOCATOR);
    journey();
    let balance = region.change();
    assert_eq!(
        balance.bytes_allocated, balance.bytes_deallocated,
        "all relevant Native/World images, observations and request owners close"
    );
}
pub(super) fn cold_read(
    world: &host::Host,
    principal: &Principal,
    scope: &WorthQueryRequestScope,
    calls: &provider::Calls,
) {
    let queries = calls.queries.load(Ordering::SeqCst);
    let result = world
        .runtime()
        .request(principal, scope)
        .on_branch(world.current_world())
        .query(all("B"))
        .execute();
    let Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(denial)) = result else {
        panic!("cold current material must refuse, never attach an older image or return empty")
    };
    assert!(
        matches!(
            denial.cause(),
            WorthQueryManagedIndexDenialCause::Refused(
                WorthQueryManagedIndexRefusal::RootUnavailable
            )
        ),
        "{denial:?}"
    );
    assert_eq!(
        calls.queries.load(Ordering::SeqCst),
        queries,
        "cold denial precedes provider query"
    );
}
fn journey() {
    let calls = Arc::new(provider::Calls::default());
    let config = Configuration::new(Arc::clone(&calls));
    let rules = Arc::clone(&config.rules);
    let prefix_calls = Arc::new(prefix::Calls::default());
    let prefix_rules = Arc::new(prefix_rule::Calls::new(Arc::clone(&prefix_calls)));
    let world = host::install_geometry(prefix_calls, prefix_rules, config);
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
    let a = request
        .query(all("A"))
        .open_index::<provider::BoxIndex>()
        .unwrap();
    let b = request
        .query(all("B"))
        .open_index::<provider::BoxIndex>()
        .unwrap();
    let old_b = request.query(all("B")).execute().unwrap();
    assert_eq!(values(old_b.rows()), expected("B", 10, all("B").probe));
    let original = old_b.result_set_observation().unwrap().clone();
    let foreign = request.query(all("A")).evict_index(&b).unwrap_err();
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(denial) = foreign else {
        panic!("foreign scope cleanup must refuse before effects: {foreign:?}")
    };
    assert!(matches!(
        denial.cause(),
        WorthQueryManagedIndexDenialCause::Refused(
            WorthQueryManagedIndexRefusal::ForeignAssociation
        )
    ));
    drop(denial);
    assert_eq!(
        values(request.query(all("B")).execute().unwrap().rows()),
        expected("B", 10, all("B").probe)
    );

    let builds = calls.builds.load(Ordering::SeqCst);
    request.query(all("B")).evict_index(&b).unwrap();
    assert_eq!(b.lifecycle(), WorthQueryManagedIndexLifecycle::Evicted);
    cold_read(&world, &principal, &scope, &calls);
    // A real Directory field changes; membership and every Source row remain
    // unchanged. The old complete Source condition is consumed under Native,
    // first while cold, then again AFTER its newly built material publishes.
    for (key, note) in [(0x919460u64, "first"), (0x919461, "second")] {
        let result = request
            .advanced()
            .mutate(refresh::Refresh {
                directory: "B".into(),
                note: note.into(),
            })
            .expect_result_set(original.clone())
            .idempotency(&key)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                result,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "unchanged Source custody survives eviction/refill: {result:?}"
        );
    }
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds + 1,
        "only the first cold committed Source is reconstructed"
    );
    assert_eq!(
        values(request.query(all("B")).execute().unwrap().rows()),
        expected("B", 10, all("B").probe)
    );
    let b = request
        .query(all("B"))
        .open_index::<provider::BoxIndex>()
        .unwrap();
    request.query(all("B")).evict_index(&b).unwrap();
    rules.records.lock().unwrap().clear();
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let builds = calls.builds.load(Ordering::SeqCst);
    let memory = accounted_request();
    let refused = request
        .advanced()
        .mutate(Edit::Move {
            directory: "A".into(),
            vertex: "v1".into(),
            x: 50,
            y: 0,
        })
        .idempotency(&0x919462u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::Execution(&memory))
        .unwrap();
    require_violation(refused);
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds + 1,
        "B really reconstructed before its refusing rule"
    );
    assert!(rules
        .records
        .lock()
        .unwrap()
        .iter()
        .any(|record| values(&record.after) == expected("B", 50, rule::probe())));
    assert_eq!(
        memory.charged_memory_bytes(),
        0,
        "refused cold staging releases its original funding immediately"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    cold_read(&world, &principal, &scope, &calls);
    cancellation::run(&world, &principal, &scope, &calls, &rules);
    assert_eq!(
        values(old_b.rows()),
        expected("B", 10, all("B").probe),
        "retained old material remains immutable"
    );
    assert_eq!(
        values(request.query(all("B")).execute().unwrap().rows()),
        expected("B", 20, all("B").probe)
    );
    a.close();
    b.close();
}
