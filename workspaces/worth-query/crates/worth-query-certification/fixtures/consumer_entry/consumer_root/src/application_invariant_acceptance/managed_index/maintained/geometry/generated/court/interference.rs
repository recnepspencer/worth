//! A real Signal successor wins while Native Source suspension awaits World CAS.
use super::*;
use crate::application_invariant_acceptance::managed_index::prefix_observation;
use std::{num::NonZeroUsize, time::Duration};
use worth_query_host::facade::runtime::{CompositeComponentChangePosture, ProductUnpublishedCause};

#[test]
fn generated_source_suspension_loses_to_signal_successor() {
    lifecycle::isolated_allocation_case(
        run,
        concat!(
            module_path!(),
            "::generated_source_suspension_loses_to_signal_successor"
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
        "runtime shutdown releases physical owners of unresolved losing custody"
    );
}
fn journey() {
    let calls = Arc::new(provider::Calls::default());
    let configuration = Configuration::new(calls.clone());
    let rules = configuration.rules.clone();
    let prefix = Arc::new(prefix_observation::Calls::default());
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
    let source = request
        .query(source::ReadDirectory {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    let generated_source = source.observed_sources()[0].clone();
    let a_scope = source.rows()[0].coordinate;
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
    let before = observation.selected_commit().clone();
    assert!(world
        .product_publication_recovery_page(None, NonZeroUsize::MIN)
        .unwrap()
        .rows()
        .is_empty());
    let contender_scope = authentication::request_scope();
    let contender =
        world.prepare_signal_only_publication_for_test(world.current_world(), &contender_scope);
    assert_eq!(world.world_active_publication_attempts_for_test(), 1);
    let pause = world
        .world_operation_control_for_test()
        .pause_before_product_compare_for_test(NonZeroUsize::MIN);
    let (outcome, winner) = std::thread::scope(|threads| {
        let contender = threads.spawn(move || {
            // On unwind, release the issuing thread before the scope joins it.
            let pause = pause;
            assert!(
                pause.wait_until_reached(Duration::from_secs(10)),
                "actual Native suspension reaches the final World comparison"
            );
            let winner = contender();
            pause.release();
            winner
        });
        let selected = world
            .with_host_advancement(|phase| {
                world
                    .on_branch(world.current_world())
                    .select(&phase)
                    .unwrap()
            })
            .unwrap();
        let outcome = selected
            .suspend_current_generated_output::<producer::Producer>(&scope, generated_source);
        (
            outcome,
            contender
                .join()
                .expect("the real Signal contender completed"),
        )
    });
    let mut recovery = match outcome {
        Err(WorthQueryGeneratedOutputSuspensionFailure::ProductUnpublished(recovery)) => recovery,
        Err(other) => panic!("the uncancelled Native loser retains actual custody: {other:?}"),
        Ok(_) => panic!("the World CAS loser cannot report performed suspension"),
    };
    assert_eq!(winner.old_product_head().selected_commit(), &before);
    assert_ne!(winner.commit().identity(), &before);
    assert_eq!(
        winner.component_results().relational_posture(),
        CompositeComponentChangePosture::RetainExact
    );
    assert_eq!(
        winner.component_results().signal_posture(),
        CompositeComponentChangePosture::Published
    );
    assert_eq!(
        winner
            .commit()
            .basis()
            .relational_basis()
            .admission_identity(),
        winner
            .old_product_head()
            .basis()
            .relational_basis()
            .admission_identity()
    );
    assert_ne!(
        winner.commit().basis().signal_basis().admission_identity(),
        winner
            .old_product_head()
            .basis()
            .signal_basis()
            .admission_identity()
    );
    assert_rules(&rules, scopes, true, 10);
    let builds = calls.builds.load(Ordering::SeqCst);
    let updates = calls.updates.load(Ordering::SeqCst);
    let assert_selected = || {
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            winner.commit().identity()
        );
        assert_eq!(world.world_active_publication_attempts_for_test(), 0);
        for directory in ["A", "B"] {
            let source = || query::ReadSource {
                directory: directory.into(),
            };
            assert_eq!(
                values(request.query(all(directory)).execute().unwrap().rows()),
                expected(directory, 10)
            );
            assert_eq!(
                values(request.query(source()).execute().unwrap().rows()),
                expected(directory, 10)
            );
            assert_eq!(
                values(pinned.query(source()).execute().unwrap().rows()),
                expected(directory, 10)
            );
        }
        assert_eq!(values(original.rows()), expected("A", 10));
        assert_eq!(calls.builds.load(Ordering::SeqCst), builds);
        assert_eq!(calls.updates.load(Ordering::SeqCst), updates);
        assert_rules(&rules, scopes, true, 10);
    };
    let records = world
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert!(records.next_after().is_none());
    assert_eq!(
        records.rows().len(),
        1,
        "exactly the actual losing publication is retained"
    );
    let record = records.rows()[0].handle();
    let assert_custody = || {
        let retained = world.readmit_product_publication_recovery(record).unwrap();
        let effects = retained.inspect().unwrap();
        assert_eq!(effects.expected_product().selected_commit(), &before);
        assert_eq!(
            effects.cause(),
            ProductUnpublishedCause::ProductPublicationLost
        );
        assert_eq!(effects.owner_effect_count(), 1);
        assert!(effects.live_obligation_count() > 0);
        // Only read-only inspection views close; the generated carrier still owns recovery.
    };
    assert_selected();
    assert_custody();
    for _ in 0..2 {
        let fresh = authentication::request_scope();
        let refused = match world
            .continue_generated_output_suspension_recovery::<producer::Producer>(recovery, &fresh)
        {
            Err(refused) => refused,
            Ok(_) => panic!("an independent Signal successor grants no rebase authority"),
        };
        assert_eq!(
            refused.stage(),
            WorthQueryGeneratedOutputSuspensionRecoveryStage::StaleOccurrence
        );
        recovery = refused.into_recovery();
        assert_eq!(recovery.product_branch(), world.current_world());
        assert_selected();
        assert_custody();
        let records = world
            .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
            .unwrap();
        assert!(records.next_after().is_none());
        assert_eq!(records.rows().len(), 1);
        assert_eq!(records.rows()[0].handle(), record);
    }
    // The unresolved carrier and real catalog survive every assertion; shutdown
    // releases their physical owners without claiming adoption or rollback.
    drop(recovery);
    a.close();
    b.close();
}
