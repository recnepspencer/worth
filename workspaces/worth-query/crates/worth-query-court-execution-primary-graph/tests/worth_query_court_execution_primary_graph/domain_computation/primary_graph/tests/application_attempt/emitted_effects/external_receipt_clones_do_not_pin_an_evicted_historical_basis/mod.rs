//! external receipt clones do not pin an evicted historical basis.
use crate::domain_computation::primary_graph::tests::application_attempt::emitted_effects::*;

#[test]
fn external_receipt_clones_do_not_pin_an_evicted_historical_basis() {
    {
        let world = installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());

        let selected = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                world.selected_product(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                )
            })
            .expect("the fixture read owns its installed host call");
        let _live = (*(*world.application.runtime_for_test()).primary_provider_for_test())
            .observe_application_commit_causality_for_test(selected.product());
        let request = live_scope();
        let principal = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                authenticated_principal(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                    &world,
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");
        let account = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                resolved_account(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                    &world,
                    "open",
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");
        let first = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                admitted_program_with_emit(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &world,
                    &principal,
                    &account,
                    &request,
                    "first-emitted",
                    Some("recover-after-eviction"),
                )
            })
            .expect("the fixture read owns its installed host call");
        let retry = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                admitted_program_with_emit(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &world,
                    &principal,
                    &account,
                    &request,
                    "first-emitted",
                    Some("recover-after-eviction"),
                )
            })
            .expect("the fixture read owns its installed host call");
        let WorthQueryApplicationCommitOutcome::Committed(original) =
            world.application.compare_and_commit_application(
                first,
                idempotency(40, 40),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
        else {
            panic!("first emitted transaction must commit");
        };
        let external_original_clone = original.clone();

        let mut current_status = "first-emitted".to_string();
        for ordinal in 0..64_u8 {
            let current_account = world
                .application
                .with_host_advancement(|phase| {
                    let host_custody = phase.test_host_request_for_test();
                    resolved_account(
                        &(host_custody.phase_for_application_for_test(&world.application))
                            .test_host_request_for_test(),
                        &world,
                        &current_status,
                        &request,
                    )
                })
                .expect("the fixture read owns its installed host call");
            let replacement = format!("eviction-filler-{ordinal}");
            let program = world
                .application
                .with_host_advancement(|phase| {
                    let host_custody = phase.test_host_request_for_test();
                    admitted_program_with_emit(
                        &host_custody.phase_for_application_for_test(&world.application),
                        &world,
                        &principal,
                        &current_account,
                        &request,
                        &replacement,
                        None,
                    )
                })
                .expect("the fixture read owns its installed host call");
            assert!(matches!(
                world.application.compare_and_commit_application(
                    program,
                    idempotency(ordinal + 64, ordinal + 64),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation
                ),
                WorthQueryApplicationCommitOutcome::Committed(_)
            ));
            current_status = replacement;
        }
        assert!(
            (*(*world.application.runtime_for_test()).primary_provider_for_test())
                .committed_application_emissions_for_test(original.committed_product_publication())
                .is_empty(),
            "the first batch must be absent from the bounded live source"
        );

        let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) =
            world.application.compare_and_commit_application(
                retry,
                idempotency(40, 40),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
        else {
            panic!("provider idempotency must recover after live-source eviction");
        };
        assert!(recovered.is_same_authoritative_commit(&original));
        assert_eq!(
            recovered.terminal().kind(),
            WorthQueryApplicationCommitTerminalKind::Recovered
        );
        assert_eq!(recovered.emitted_effect_count(), 1);
        let external_recovered_clone = recovered.clone();
        assert_eq!(
            (*(*world.application.runtime_for_test()).primary_provider_for_test())
                .retained_receipt_basis_count(),
            64,
            "completed evidence must not grow the bounded receipt-basis owner"
        );
        let expired_read = WorthQueryApplicationHistoricalRead::at_application_commit_for_test(
            &external_recovered_clone,
        );
        let denial = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                world
                    .application
                    .admit_application_historical_basis_for_test(
                        &host_custody.borrowed_request_for_test(),
                        expired_read,
                        &request,
                    )
                    .err()
                    .expect("external receipt clones must not pin an evicted historical basis")
            })
            .expect("the fixture read owns its installed host call");
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationQueryAdmissionDenialKind::BasisUnavailable
        );
        assert!(external_original_clone.is_same_authoritative_commit(&original));
        assert!(external_recovered_clone.is_same_authoritative_commit(&recovered));
    }
}
