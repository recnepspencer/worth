//! Public rebuild and invariant maintenance share actual Native cache material.
use super::super::refresh;
use super::*;
use crate::application_invariant_acceptance::managed_index::prefix_observation;
use worth_query_host::facade::runtime::{MapKernelStop, WorkCeilingDenial};
#[test]
fn public_rebuild_refills_canonical_source_and_cancelled_build_stays_cold() {
    lifecycle::isolated_allocation_case(
        run,
        concat!(
            module_path!(),
            "::public_rebuild_refills_canonical_source_and_cancelled_build_stays_cold"
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
        "both cold Source passes, canonical rows, nodes and all temporal owners release"
    );
}
fn journey() {
    let calls = Arc::new(provider::Calls::default());
    let config = Configuration::new(Arc::clone(&calls));
    let rules = Arc::clone(&config.rules);
    let prefix_calls = Arc::new(prefix_observation::Calls::default());
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
    let index = request
        .query(all("B"))
        .open_index::<provider::BoxIndex>()
        .unwrap();
    let original = request.query(all("B")).execute().unwrap();
    let expectation = original.result_set_observation().unwrap().clone();
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let builds = calls.builds.load(Ordering::SeqCst);
    let foreign = request.query(all("A")).rebuild_index(&index).unwrap_err();
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(denial) = foreign else {
        panic!("{foreign:?}")
    };
    assert!(matches!(
        denial.cause(),
        WorthQueryManagedIndexDenialCause::Refused(
            WorthQueryManagedIndexRefusal::ForeignAssociation
        )
    ));
    drop(denial);
    assert_eq!(calls.builds.load(Ordering::SeqCst), builds);
    request.query(all("B")).evict_index(&index).unwrap();
    cold::cold_read(&world, &principal, &scope, &calls);

    let cancellation = WorthQueryCancellationSource::new();
    let stopped_scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let reached = Arc::new(Mutex::new(false));
    let observed = Arc::clone(&reached);
    *calls.after_build_node_write.lock().unwrap() = Some(Box::new(move || {
        *observed.lock().unwrap() = true;
        cancellation.cancel();
    }));
    let stopped = world
        .runtime()
        .request(&principal, &stopped_scope)
        .on_branch(world.current_world())
        .query(all("B"))
        .rebuild_index(&index)
        .unwrap_err();
    assert!(
        matches!(stopped, WorthQueryApplicationRequestQueryDenial::ManagedIndex(ref denial)
        if matches!(denial.cause(), WorthQueryManagedIndexDenialCause::Work(
            WorthQueryManagedIndexWorkDenial::Controlled(WorthQueryManagedIndexStop::Request(
                WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)))))),
        "{stopped:?}"
    );
    drop(stopped);
    assert!(
        *reached.lock().unwrap(),
        "stopped after an actual provider node write"
    );
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Failed);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    cold::cold_read(&world, &principal, &scope, &calls);
    assert_eq!(calls.builds.load(Ordering::SeqCst), builds + 1);

    request.query(all("B")).rebuild_index(&index).unwrap();
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    assert_eq!(calls.builds.load(Ordering::SeqCst), builds + 2);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head,
        "cache reconstruction is not a Source publication"
    );
    assert_eq!(
        values(request.query(all("B")).execute().unwrap().rows()),
        expected("B", 10, all("B").probe)
    );
    // Consume the old retained Source condition through real Native preflight:
    // the rebuilt entry must preserve the original association, not copy IDs.
    let refreshed = request
        .advanced()
        .mutate(refresh::Refresh {
            probe: all("B").probe,
            directory: "B".into(),
            note: "after explicit rebuild".into(),
        })
        .expect_result_set(expectation)
        .idempotency(&0x919470u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            refreshed,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{refreshed:?}"
    );
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds + 2,
        "unchanged mutation consumes the public rebuilt canonical material"
    );
    rules.records.lock().unwrap().clear();
    commit(
        &world,
        &principal,
        &scope,
        Edit::Move {
            directory: "A".into(),
            vertex: "v1".into(),
            x: 20,
            y: 0,
        },
        0x919471,
    );
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds + 2,
        "later invariant maintenance updates the same resource without a second cold build"
    );
    assert!(rules
        .records
        .lock()
        .unwrap()
        .iter()
        .any(|record| values(&record.after) == expected("B", 20, rule::probe())));
    assert_eq!(values(original.rows()), expected("B", 10, all("B").probe));
    assert_eq!(
        values(request.query(all("B")).execute().unwrap().rows()),
        expected("B", 20, all("B").probe)
    );
    index.close();
}
