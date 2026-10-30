//! A real Native fork shares material while owning a fresh branch association.
use super::*;
mod source_expectation;

#[test]
fn forked_geometry_sources_read_before_first_mutation_and_isolate_sibling_edits() {
    lifecycle::isolated_allocation_case(
        run,
        concat!(
            module_path!(),
            "::forked_geometry_sources_read_before_first_mutation_and_isolate_sibling_edits"
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
        "fork header, registry, material and retained reader owners close"
    );
}
fn journey() {
    let calls = Arc::new(provider::Calls::default());
    let configuration = Configuration::new(Arc::clone(&calls));
    let prefix_calls = Arc::new(prefix::Calls::default());
    let prefix_rules = Arc::new(prefix_rule::Calls::new(Arc::clone(&prefix_calls)));
    let world = host::install_geometry(Arc::clone(&prefix_calls), prefix_rules, configuration);
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    setup(&world, &principal, &scope);
    source_expectation::seed(&world, &principal, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let index = request
        .query(all("A"))
        .open_index::<provider::BoxIndex>()
        .unwrap();
    let original = request.query(all("A")).execute().unwrap();
    let builds = calls.builds.load(Ordering::SeqCst);
    let reused = world
        .runtime()
        .with_host_advancement(|phase| {
            world
                .runtime()
                .branches()
                .fork(world.current_world())
                .components(|components| {
                    components
                        .reuse_exact_relational_basis()
                        .reuse_exact_signal_basis()
                })
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let reused_request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(reused);
    let reused_index = reused_request
        .query(all("A"))
        .open_index::<provider::BoxIndex>()
        .unwrap();
    assert_eq!(
        values(reused_request.query(all("A")).execute().unwrap().rows()),
        expected("A", 10, all("A").probe)
    );
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
    let sibling = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch);
    let sibling_index = sibling
        .query(all("A"))
        .open_index::<provider::BoxIndex>()
        .unwrap();
    assert_eq!(
        values(sibling.query(all("A")).execute().unwrap().rows()),
        expected("A", 10, all("A").probe)
    );
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds,
        "actual fork shares complete source material before its first mutation"
    );
    source_expectation::exercise(&world, &principal, &scope, branch, &prefix_calls);
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
        0x919430,
    );
    assert!(sibling
        .query(find("A", 15))
        .execute()
        .unwrap()
        .rows()
        .is_empty());
    let moved = sibling
        .advanced()
        .mutate(Edit::Move {
            directory: "A".into(),
            vertex: "v1".into(),
            x: 30,
            y: 0,
        })
        .idempotency(&0x919431u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            moved,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{moved:?}"
    );
    assert_eq!(
        values(sibling.query(find("A", 25)).execute().unwrap().rows()),
        expected("A", 30, find("A", 25).probe)
    );
    assert!(request
        .query(find("A", 25))
        .execute()
        .unwrap()
        .rows()
        .is_empty());
    assert_eq!(values(original.rows()), expected("A", 10, all("A").probe));
    assert_eq!(
        values(reused_request.query(all("A")).execute().unwrap().rows()),
        expected("A", 10, all("A").probe),
        "ReuseExact retains its genuine selected immutable component after both siblings edit"
    );
    assert_eq!(calls.builds.load(Ordering::SeqCst), builds);
    let before = (
        calls.builds.load(Ordering::SeqCst),
        calls.updates.load(Ordering::SeqCst),
        calls.probe_validations.load(Ordering::SeqCst),
    );
    let denial = sibling.query(all("A")).rebuild_index(&index).unwrap_err();
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(denial) = denial else {
        panic!("the foreign handle must refuse its managed association: {denial:?}")
    };
    assert!(
        matches!(
            denial.cause(),
            WorthQueryManagedIndexDenialCause::Refused(
                WorthQueryManagedIndexRefusal::ForeignAssociation
            )
        ),
        "{denial:?}"
    );
    assert_eq!(
        (
            calls.builds.load(Ordering::SeqCst),
            calls.updates.load(Ordering::SeqCst),
            calls.probe_validations.load(Ordering::SeqCst)
        ),
        before
    );
    // This is a fresh CURRENT World request selecting its genuine older
    // Native component. The Native head must not replace that selection.
    reused_request
        .query(all("A"))
        .evict_index(&reused_index)
        .unwrap();
    let queries = calls.probe_validations.load(Ordering::SeqCst);
    let cold = reused_request.query(all("A")).execute();
    let Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(denial)) = cold else {
        panic!("the reused selected image must become cold")
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
    assert_eq!(calls.probe_validations.load(Ordering::SeqCst), queries);
    assert_eq!(
        values(request.query(all("A")).execute().unwrap().rows()),
        expected("A", 20, all("A").probe)
    );
    assert_eq!(
        values(sibling.query(all("A")).execute().unwrap().rows()),
        expected("A", 30, all("A").probe)
    );
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds,
        "eviction and warm sibling reads reconstruct nothing"
    );
    reused_request
        .query(all("A"))
        .rebuild_index(&reused_index)
        .unwrap();
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds + 1,
        "only the selected older Native component reconstructs"
    );
    assert_eq!(
        values(reused_request.query(all("A")).execute().unwrap().rows()),
        expected("A", 10, all("A").probe)
    );
    assert_eq!(
        values(request.query(all("A")).execute().unwrap().rows()),
        expected("A", 20, all("A").probe)
    );
    assert_eq!(
        values(sibling.query(all("A")).execute().unwrap().rows()),
        expected("A", 30, all("A").probe)
    );
    assert_eq!(calls.builds.load(Ordering::SeqCst), builds + 1);
    index.close();
    sibling_index.close();
    reused_index.close();
    drop((original, moved, sibling, request, reused_request));
    world
        .runtime()
        .with_host_advancement(|phase| world.runtime().on_branch(branch).close(&phase).unwrap())
        .unwrap();
    world
        .runtime()
        .with_host_advancement(|phase| world.runtime().on_branch(reused).close(&phase).unwrap())
        .unwrap();
}
