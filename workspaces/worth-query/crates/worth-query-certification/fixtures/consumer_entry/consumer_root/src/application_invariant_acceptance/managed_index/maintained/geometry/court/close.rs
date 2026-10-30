//! An actual ReuseExact alias keeps its selected Source after Native owner close.
use super::*;

#[test]
fn closing_native_owner_keeps_retained_alias_read_eviction_and_refill_valid() {
    lifecycle::isolated_allocation_case(
        run,
        concat!(
            module_path!(),
            "::closing_native_owner_keeps_retained_alias_read_eviction_and_refill_valid"
        ),
    );
}
fn run() {
    journey();
    let region = Region::new(lifecycle::ALLOCATOR);
    journey();
    let released = region.change();
    assert_eq!(
        released.bytes_allocated, released.bytes_deallocated,
        "closed lookups, actual Native/World shares and all reader owners release"
    );
}
fn journey() {
    let calls = Arc::new(provider::Calls::default());
    let configuration = Configuration::new(Arc::clone(&calls));
    let prefix_calls = Arc::new(prefix::Calls::default());
    let prefix_rules = Arc::new(prefix_rule::Calls::new(Arc::clone(&prefix_calls)));
    let world = host::install_geometry(prefix_calls, prefix_rules, configuration);
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    setup(&world, &principal, &scope);
    let branch = world
        .runtime()
        .with_host_advancement(|phase| {
            world
                .runtime()
                .branches()
                .fork(world.current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let branch_request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch);
    let changed = branch_request
        .advanced()
        .mutate(Edit::Move {
            directory: "A".into(),
            vertex: "v1".into(),
            x: 20,
            y: 0,
        })
        .idempotency(&0x919490_u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            changed,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{changed:?}"
    );
    drop(changed);
    let original = branch_request.query(all("A")).execute().unwrap();
    let alias = world
        .runtime()
        .with_host_advancement(|phase| {
            world
                .runtime()
                .branches()
                .fork(branch)
                .components(|components| {
                    components
                        .reuse_exact_relational_basis()
                        .reuse_exact_signal_basis()
                })
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let request = world.runtime().request(&principal, &scope).on_branch(alias);
    let index = request
        .query(all("A"))
        .open_index::<provider::BoxIndex>()
        .unwrap();
    let builds = calls.builds.load(Ordering::SeqCst);
    assert_eq!(
        values(request.query(all("A")).execute().unwrap().rows()),
        expected("A", 20, all("A").probe)
    );
    let closed = world
        .runtime()
        .with_host_advancement(|phase| world.runtime().on_branch(branch).close(&phase).unwrap())
        .unwrap();
    assert!(closed.is_complete());
    assert_eq!(
        closed.retired_component_count(),
        1,
        "the real owned Native component was deleted"
    );
    assert_eq!(
        values(request.query(all("A")).execute().unwrap().rows()),
        expected("A", 20, all("A").probe)
    );
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds,
        "retained read borrows its actual selected image"
    );
    request.query(all("A")).evict_index(&index).unwrap();
    let queries = calls.queries.load(Ordering::SeqCst);
    let Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(denial)) =
        request.query(all("A")).execute()
    else {
        panic!("eviction makes the alias's genuine selected Source cold");
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
    assert_eq!(calls.queries.load(Ordering::SeqCst), queries);
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    request.query(all("A")).rebuild_index(&index).unwrap();
    assert_eq!(calls.builds.load(Ordering::SeqCst), builds + 1);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head,
        "refill creates no Source/World publication"
    );
    assert_eq!(
        values(request.query(all("A")).execute().unwrap().rows()),
        expected("A", 20, all("A").probe)
    );
    let main = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    assert_eq!(
        values(main.query(all("A")).execute().unwrap().rows()),
        expected("A", 10, all("A").probe)
    );
    assert_eq!(values(original.rows()), expected("A", 20, all("A").probe));
    index.close();
    drop(original);
    let closed_alias = world
        .runtime()
        .with_host_advancement(|phase| world.runtime().on_branch(alias).close(&phase).unwrap())
        .unwrap();
    assert_eq!(
        closed_alias.retired_component_count(),
        0,
        "ReuseExact owns no component deletion work"
    );
    assert!(world.runtime().branches().pending_cleanup().is_empty());
}
