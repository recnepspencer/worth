//! Cancellation cannot erase a performed Source cutover or its cleanup custody.
use super::*;
use std::{num::NonZeroUsize, sync::Mutex};
use worth_query_host::facade::runtime::CompositeComponentChangePosture;
mod restoration;

#[test]
fn generated_source_late_cancellation_preserves_performed_and_published_recovery() {
    lifecycle::isolated_allocation_case(
        run,
        concat!(
            module_path!(),
            "::generated_source_late_cancellation_preserves_performed_and_published_recovery"
        ),
    );
}
fn run() {
    journey();
    let region = stats_alloc::Region::new(lifecycle::ALLOCATOR);
    journey();
    let released = region.change();
    assert_eq!(
        released.bytes_allocated, released.bytes_deallocated,
        "all actual performed, published recovery, Source and retained reader owners close"
    );
}
fn read_source(directory: &str) -> query::ReadSource {
    query::ReadSource {
        directory: directory.into(),
    }
}
fn journey() {
    let calls = Arc::new(provider::Calls::default());
    let configuration = Configuration::new(calls.clone());
    let rules = configuration.rules.clone();
    let prefix = Arc::new(prefix::Calls::default());
    let prefix_rules = Arc::new(prefix_rule::Calls::new(prefix.clone()));
    let world = host::install_geometry(prefix, prefix_rules, configuration);
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
    let observation = request.retain_current_read().unwrap();
    let pinned = request.at(&observation);
    let a_source = request
        .query(source::ReadDirectory {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    let generated_source = a_source.observed_sources()[0].clone();
    let a_scope = a_source.rows()[0].coordinate;
    let b_scope = request
        .query(source::ReadDirectory {
            directory: "B".into(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .coordinate;
    let scopes = [(a_scope, "A"), (b_scope, "B")];
    rules.capture_complete_source.store(true, Ordering::SeqCst);
    rules.records.lock().unwrap().clear();

    let cancellation = WorthQueryCancellationSource::new();
    let original_scope = WorthQueryRequestScope::new(None, cancellation.token());
    let observed = Arc::new(Mutex::new(None));
    let recorded = observed.clone();
    let before = observation.selected_commit().clone();
    let control = world.world_operation_control_for_test();
    let observer = control.observe_next_performed_publication_for_test(move |performed| {
        assert_eq!(performed.old_product_head().selected_commit(), &before);
        assert_ne!(performed.commit().identity(), &before);
        assert_eq!(performed.cost_counters().cas_wins(), 1);
        assert_eq!(
            performed.component_results().relational_posture(),
            CompositeComponentChangePosture::Published
        );
        assert_eq!(
            performed.component_results().signal_posture(),
            CompositeComponentChangePosture::RetainExact
        );
        *recorded.lock().unwrap() = Some(performed.commit().identity().clone());
        cancellation.cancel();
    });
    let selected = world
        .with_host_advancement(|phase| {
            world
                .on_branch(world.current_world())
                .select(&phase)
                .unwrap()
        })
        .unwrap();
    let suspended = selected
        .suspend_current_generated_output::<producer::Producer>(&original_scope, generated_source)
        .expect("late cancellation preserves the actual performed suspended-output custody");
    assert!(original_scope.cancellation().is_cancelled());
    let published = observed
        .lock()
        .unwrap()
        .take()
        .expect("the actual World successor was observed");
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &published
    );
    assert_eq!(world.world_active_publication_attempts_for_test(), 0);
    assert!(world
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
    assert_rules(&rules, scopes, true, 10);
    for directory in ["A", "B"] {
        assert_suspended(request.query(all(directory)).execute());
        assert_suspended(request.query(read_source(directory)).execute());
    }
    drop(observer);
    let completed = reconstruction::complete(&world, suspended, 20);
    restoration::run(&world, completed, &rules, scopes);
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
            expected(directory, 10)
        );
    }
    assert_eq!(values(original.rows()), expected("A", 10));
    a.close();
    b.close();
}
