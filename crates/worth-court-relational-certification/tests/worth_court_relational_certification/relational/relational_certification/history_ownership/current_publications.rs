//! Current obligations come from Native custody, including without accounting.
use super::publication_fixture::PublishedModel;
use super::*;
use worth_relational::facade::branch::RelationalOwnedModelOutputRoute as Route;
fn exercise(worker: bool) {
    let host = serial_request();
    let setup = &ExecutionRequest::serial(&host);
    let world = PublishedModel::new(setup);
    let mut second = world.meaning.clone();
    second.family = "second-obligation".into();
    publication_fixture::publish(
        setup,
        &world.runtime,
        &world.owner,
        &world.installed,
        &second,
        "second",
    );
    let selected = transfer_fixture::basis(&world.runtime);
    let other = PublishedModel::new(setup);
    let foreign = transfer_fixture::basis(&other.runtime);
    let foreign_publication = other
        .runtime
        .prepare_historical_output_selection(
            setup,
            &other.installed,
            &foreign,
            &foreign,
            &other.meaning,
        )
        .unwrap();
    let lease = worth_execution::ExecutionResourceLease::serial(worth_execution::LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::for_workers(
            worth_foundational::ExecutionPosture::Automatic,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::new(1).unwrap(),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    })
    .unwrap();
    let execution = if worker {
        ExecutionRequest::leased(&lease)
    } else {
        ExecutionRequest::serial(&host)
    };
    if worker {
        assert!(!execution.has_accounting());
    }
    let read = world
        .runtime
        .observe_owned_model_publications(
            &execution,
            &world.installed,
            &selected,
            &ownership(),
            world.model,
        )
        .unwrap();
    assert_eq!(read.basis().descriptor(), selected.descriptor());
    let mut families = std::collections::BTreeSet::new();
    for route in read.routes() {
        let Route::Published(route) = route else {
            panic!("live publication must stay live");
        };
        families.insert(route.meaning().family.clone());
        let original = world
            .runtime
            .prepare_historical_output_selection(
                &execution,
                &world.installed,
                &selected,
                &selected,
                route.meaning(),
            )
            .unwrap();
        assert!(route
            .matches_publication(&execution, original.publication())
            .unwrap());
        assert!(
            !route
                .matches_publication(&execution, foreign_publication.publication())
                .unwrap(),
            "equal ids in another runtime cannot match"
        );
    }
    assert_eq!(
        families,
        [world.meaning.family.clone(), second.family]
            .into_iter()
            .collect()
    );
    let cancelled = worth_execution::CancellationSource::new();
    cancelled.cancel();
    let stopped = host.clone().with_cancellation(cancelled.token());
    let error = world
        .runtime
        .observe_owned_model_publications(
            &ExecutionRequest::serial(&stopped),
            &world.installed,
            &selected,
            &ownership(),
            world.model,
        )
        .err()
        .expect("cancelled projection");
    assert!(error.kernel_stop().is_some(), "{error:?}");
    assert_eq!(
        transfer_fixture::basis(&world.runtime).descriptor(),
        selected.descriptor()
    );
}
#[test]
fn current_obligations_keep_exact_native_publication_coordinates() {
    exercise(false);
}
#[test]
fn worker_only_current_obligations_keep_exact_native_publication_coordinates() {
    exercise(true);
}
#[test]
fn cold_current_obligations_equal_the_warm_current_facts() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let world = PublishedModel::new(execution);
    let selected = transfer_fixture::basis(&world.runtime);
    let warm = world
        .runtime
        .observe_owned_model_publications(
            execution,
            &world.installed,
            &selected,
            &ownership(),
            world.model,
        )
        .unwrap();
    let meanings: Vec<_> = warm
        .routes()
        .map(|route| match route {
            Route::Published(route) => route.meaning().clone(),
            _ => panic!("live route"),
        })
        .collect();
    let checkpoint = world
        .runtime
        .durability_authority()
        .native_checkpoint(execution, execution.allocation_policy())
        .unwrap();
    let mut cold = RelationalRuntimeApi::builder()
        .schema_registry(publication_fixture::registry())
        .build()
        .unwrap();
    let expected = cold
        .prepare_initial_schema_installation()
        .unwrap()
        .install_with_custom_invariants(
            RelationalSchemaRegistry::new(),
            vec![ownership().invariant_registration().unwrap()],
        )
        .unwrap();
    cold.durability_recovery()
        .restore_native_checkpoint(execution, &checkpoint)
        .unwrap();
    let receipt = cold
        .readmit_recovered_initial_schema_installation(expected)
        .unwrap();
    let (_, installed) = receipt.issue_application_publication_owner();
    let installed = installed.with_source_scope_owned_producers([("generator".into(), KindId(3))]);
    let basis = transfer_fixture::basis(&cold);
    let read = cold
        .observe_owned_model_publications(execution, &installed, &basis, &ownership(), world.model)
        .unwrap();
    assert_eq!(
        read.routes()
            .map(|route| match route {
                Route::Published(route) => route.meaning().clone(),
                _ => panic!("live route"),
            })
            .collect::<Vec<_>>(),
        meanings
    );
}
