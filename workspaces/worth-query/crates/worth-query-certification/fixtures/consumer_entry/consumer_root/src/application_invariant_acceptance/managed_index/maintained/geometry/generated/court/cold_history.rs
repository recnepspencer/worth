//! Cold generated Source construction and current-security historical admission.
use super::*;
use std::sync::atomic::AtomicBool;

fn read_source(directory: &str) -> query::ReadSource {
    query::ReadSource {
        directory: directory.into(),
    }
}

#[test]
fn cold_generated_source_rebuilds_before_rules_and_historical_reads_keep_current_security() {
    lifecycle::isolated_allocation_case(run, concat!(module_path!(), "::cold_generated_source_rebuilds_before_rules_and_historical_reads_keep_current_security"));
}
fn run() {
    journey();
    let region = stats_alloc::Region::new(lifecycle::ALLOCATOR);
    journey();
    let released = region.change();
    assert_eq!(released.bytes_allocated, released.bytes_deallocated, "all cold staging, actual historical observations, endpoint payload and final reader owners close");
}

fn journey() {
    let calls = Arc::new(provider::Calls::default());
    let config = Configuration::new(calls.clone());
    let rules = config.rules.clone();
    let prefix = Arc::new(prefix::Calls::default());
    let prefix_rules = Arc::new(prefix_rule::Calls::new(prefix.clone()));
    let world = host::install_geometry(prefix, prefix_rules.clone(), config);
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    population::native_b(&world, &principal, &scope);
    population::produce(&world, &principal, &scope);
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
    let original = request.query(all("A")).execute().unwrap();
    assert_eq!(values(original.rows()), expected("A", 10));
    let observation = request.retain_current_read().unwrap();
    let pinned = request.at(&observation);
    for directory in ["A", "B"] {
        assert_eq!(
            values(
                pinned
                    .query(read_source(directory))
                    .execute()
                    .unwrap()
                    .rows()
            ),
            expected(directory, 10)
        );
    }
    let directory_source = request
        .query(source::ReadDirectory {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    let generated_source = directory_source.observed_sources()[0].clone();
    let a_scope = directory_source.rows()[0].coordinate;
    let b_scope = request
        .query(source::ReadDirectory {
            directory: "B".into(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .coordinate;
    let scopes = [(a_scope, "A"), (b_scope, "B")];
    assert_ne!(a_scope, b_scope);
    rules.capture_complete_source.store(true, Ordering::SeqCst);
    request.query(all("B")).evict_index(&b).unwrap();
    assert_eq!(b.lifecycle(), WorthQueryManagedIndexLifecycle::Evicted);
    cold_read(&world, &principal, &scope, &calls);
    assert_eq!(
        values(request.query(read_source("B")).execute().unwrap().rows()),
        expected("B", 10),
        "cold derived material does not remove available Native Source rows"
    );
    b.close();
    assert_eq!(b.lifecycle(), WorthQueryManagedIndexLifecycle::Disposed);
    cold_read(&world, &principal, &scope, &calls);

    let position = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    rules.records.lock().unwrap().clear();
    prefix_rules.records.lock().unwrap().clear();
    let builds = calls.builds.load(Ordering::SeqCst);
    cancel_cold_build(&world, &calls, generated_source.clone());
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds + 1,
        "one genuine B cold construction reached its first node"
    );
    assert!(
        rules.records.lock().unwrap().is_empty(),
        "all cold preparation precedes every geometry rule"
    );
    assert!(
        prefix_rules.records.lock().unwrap().is_empty(),
        "no sibling rule runs after stopped Source preparation"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &position
    );
    cold_read(&world, &principal, &scope, &calls);
    assert_eq!(
        values(request.query(all("A")).execute().unwrap().rows()),
        expected("A", 10)
    );
    assert_eq!(
        values(pinned.query(read_source("B")).execute().unwrap().rows()),
        expected("B", 10)
    );

    let builds = calls.builds.load(Ordering::SeqCst);
    let selected = world
        .with_host_advancement(|phase| {
            world
                .on_branch(world.current_world())
                .select(&phase)
                .unwrap()
        })
        .unwrap();
    let suspended = selected
        .suspend_current_generated_output::<producer::Producer>(&scope, generated_source)
        .unwrap();
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds + 1,
        "retry rebuilds exactly cold B from actual before-Source"
    );
    assert_rules(&rules, scopes, true, 10);
    assert!(
        rules
            .records
            .lock()
            .unwrap()
            .iter()
            .all(|row| row.builds_before_callback == builds + 1),
        "both actual scopes see complete cold preparation before the first callback"
    );
    assert_suspended(request.query(all("A")).execute());
    assert_suspended(request.query(read_source("A")).execute());
    assert_suspended(pinned.query(read_source("A")).execute());
    assert_suspended(pinned.query(read_source("B")).execute());
    assert_eq!(
        values(original.rows()),
        expected("A", 10),
        "decoded result retention is a separate immutable-result property"
    );

    rules.records.lock().unwrap().clear();
    let builds = calls.builds.load(Ordering::SeqCst);
    let completed = reconstruction::complete(&world, suspended, 20);
    world
        .restore_generated_output(completed, &scope)
        .unwrap_or_else(|failure| {
            panic!(
                "lawful changed endpoint reconstruction restores: {:?}",
                failure.cause()
            )
        });
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds + 2,
        "changed actual endpoint declines correspondence and constructs both scopes"
    );
    assert_rules(&rules, scopes, false, 20);
    for directory in ["A", "B"] {
        assert_eq!(
            values(request.query(all(directory)).execute().unwrap().rows()),
            expected(directory, 20)
        );
        assert_eq!(
            values(
                request
                    .query(read_source(directory))
                    .execute()
                    .unwrap()
                    .rows()
            ),
            expected(directory, 20)
        );
        assert_eq!(
            values(
                pinned
                    .query(read_source(directory))
                    .execute()
                    .unwrap()
                    .rows()
            ),
            expected(directory, 10),
            "fresh retained execution reads its genuine older endpoint, not current fallback"
        );
    }
    let queries = calls.probe_validations.load(Ordering::SeqCst);
    let denial = pinned
        .query(all("A"))
        .execute()
        .err()
        .expect("retained managed Parent mode remains unsupported");
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(denial) = denial else {
        panic!("current security admits before the retained Parent mode receiver")
    };
    assert!(matches!(
        denial.cause(),
        WorthQueryManagedIndexDenialCause::Refused(WorthQueryManagedIndexRefusal::UnsupportedMode)
    ));
    assert_eq!(
        calls.probe_validations.load(Ordering::SeqCst),
        queries,
        "unsupported retained managed mode refuses before provider query"
    );
    assert_eq!(values(original.rows()), expected("A", 10));
    a.close();
}

fn cold_read(
    world: &host::Host,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Catalog>,
    scope: &WorthQueryRequestScope,
    calls: &provider::Calls,
) {
    let queries = calls.probe_validations.load(Ordering::SeqCst);
    let result = world
        .runtime()
        .request(principal, scope)
        .on_branch(world.current_world())
        .query(all("B"))
        .execute();
    let Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(denial)) = result else {
        panic!("selected B is genuinely cold")
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
        calls.probe_validations.load(Ordering::SeqCst),
        queries,
        "cold refusal precedes provider query"
    );
}

fn cancel_cold_build(
    world: &host::Host,
    calls: &Arc<provider::Calls>,
    source: WorthQueryObservedSource<source::Query>,
) {
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let reached = Arc::new(AtomicBool::new(false));
    let observed = reached.clone();
    *calls.after_build_node_write.lock().unwrap() = Some(Box::new(move || {
        observed.store(true, Ordering::SeqCst);
        cancellation.cancel();
    }));
    let selected = world
        .with_host_advancement(|phase| {
            world
                .on_branch(world.current_world())
                .select(&phase)
                .unwrap()
        })
        .unwrap();
    let failure =
        match selected.suspend_current_generated_output::<producer::Producer>(&scope, source) {
            Err(failure) => failure,
            Ok(_) => panic!("actual cold construction cancellation must refuse before publication"),
        };
    assert!(
        reached.load(Ordering::SeqCst),
        "a real cold node write preceded original-request cancellation"
    );
    assert_eq!(
        failure.execution_control_stop(),
        Some(WorthQueryProviderSessionControlStopKind::Cancelled),
        "original typed cause: {failure:?}"
    );
}
