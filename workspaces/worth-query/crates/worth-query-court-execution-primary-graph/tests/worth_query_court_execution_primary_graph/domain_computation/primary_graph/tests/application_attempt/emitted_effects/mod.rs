pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program_with_emit;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitTerminalKind;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::basis::historical_authority::WorthQueryApplicationHistoricalRead;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenialKind;

#[test]
fn typed_emission_is_published_with_the_exact_provider_commit() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let selected = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    let _live = (*(*world.application.runtime_for_test()).primary_provider_for_test())
        .observe_application_commit_causality_for_test(selected.product());
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            admitted_program_with_emit(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "emitted",
                Some("account-activity"),
            )
        })
        .expect("the fixture read owns its installed host call");

    let WorthQueryApplicationCommitOutcome::Committed(receipt) =
        world.application.compare_and_commit_application(
            program,
            idempotency(31, 31),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("typed emission attempt must commit");
    };
    assert_eq!(receipt.emitted_effect_count(), 1);
    let emissions = (*(*world.application.runtime_for_test()).primary_provider_for_test())
        .committed_application_emissions_for_test(receipt.committed_product_publication());
    assert_eq!(emissions.len(), 1);
    assert_eq!(emissions[0].effect_for_test(), "AccountActivityEffect");
    assert_eq!(
        emissions[0]
            .payload_for_test::<String>()
            .map(String::as_str),
        Some("account-activity")
    );
}

#[test]
fn response_loss_recovers_emit_receipt_without_duplicate_publication() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let first = world
        .application
        .with_host_advancement(|phase| {
            admitted_program_with_emit(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "published",
                Some("once"),
            )
        })
        .expect("the fixture read owns its installed host call");
    let retry = world
        .application
        .with_host_advancement(|phase| {
            admitted_program_with_emit(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "published",
                Some("once"),
            )
        })
        .expect("the fixture read owns its installed host call");

    world.faults.lose_next_commit_response();
    let WorthQueryApplicationCommitOutcome::Committed(original) =
        world.application.compare_and_commit_application(
            first,
            idempotency(33, 33),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("idempotency must recover the response-lost emit commit");
    };
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) =
        world.application.compare_and_commit_application(
            retry,
            idempotency(33, 33),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("retry must recover the exact emit commit");
    };
    assert!(recovered.is_same_authoritative_commit(&original));
    assert_eq!(
        original.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Executed
    );
    assert_eq!(
        recovered.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Recovered
    );
    assert_eq!(recovered.emitted_effect_count(), 1);
    assert_eq!(
        (*(*world.application.runtime_for_test()).primary_provider_for_test())
            .published_application_commit_count_for_test(),
        1
    );
}

#[test]
fn in_window_receipt_admits_its_exact_historical_basis() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let first = world
        .application
        .with_host_advancement(|phase| {
            admitted_program_with_emit(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "historical",
                None,
            )
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryApplicationCommitOutcome::Committed(receipt) =
        world.application.compare_and_commit_application(
            first,
            idempotency(41, 41),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("historical transaction must commit");
    };
    let exact_version = receipt.commit_reference().version_id;
    let external_clone = receipt.clone();

    let current_account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "historical",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let advanced = world
        .application
        .with_host_advancement(|phase| {
            admitted_program_with_emit(
                &phase,
                &world,
                &principal,
                &current_account,
                &request,
                "advanced",
                None,
            )
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryApplicationCommitOutcome::Committed(advanced) =
        world.application.compare_and_commit_application(
            advanced,
            idempotency(42, 42),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("advancing transaction must commit");
    };
    assert_ne!(advanced.commit_reference().version_id, exact_version);

    let historical_read =
        WorthQueryApplicationHistoricalRead::at_application_commit_for_test(&external_clone);
    drop(external_clone);
    drop(receipt);
    let basis = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .admit_application_historical_basis_for_test(
                    &host_custody.borrowed_request_for_test(),
                    historical_read,
                    &request,
                )
                .expect("the bounded owner must retain an in-window receipt basis")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(basis.version_id_for_test(), exact_version);
    assert!(basis.release_for_test().released_for_test());
}

#[path = "external_receipt_clones_do_not_pin_an_evicted_historical_basis/mod.rs"]
pub mod external_receipt_clones_do_not_pin_an_evicted_historical_basis;
#[path = "pretransaction_emit_refusal/mod.rs"]
pub mod pretransaction_emit_refusal;
