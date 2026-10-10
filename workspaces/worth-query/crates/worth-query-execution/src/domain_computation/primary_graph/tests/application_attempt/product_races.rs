use super::{
    admitted_program, authenticated_principal, idempotency, installed_authorization_world,
    live_scope, resolved_account, WorthQueryApplicationCommitOutcome,
};

#[test]
fn concurrent_independent_attempts_preserve_one_product_winner_and_the_exact_loser_posture() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let world = installed_authorization_world(true);
        let request = live_scope();
        let principal = authenticated_principal(
            &(host_custody.phase_for_application(&world.application)).test_host_request(),
            &world,
            &request,
        );
        let accounts = [
            resolved_account(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                "open",
                &request,
            ),
            resolved_account(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                "unrelated",
                &request,
            ),
        ];
        let replacements = ["first-independent", "second-independent"];
        let keys = [idempotency(13, 13), idempotency(14, 14)];
        let selected = world.selected_product(
            &(host_custody.phase_for_application(&world.application)).test_host_request(),
        );
        let commit_count = || {
            world
                .application
                .primary_provider
                .graph
                .with_runtime(&host_custody.borrowed_request_for_test(), |runtime| {
                    runtime.history().immutable_commit_count()
                })
        };
        let baseline = commit_count();
        let first = admitted_program(
            &host_custody.phase_for_application(&world.application),
            &world,
            &principal,
            &accounts[0],
            &request,
            replacements[0],
        );
        let second = admitted_program(
            &host_custody.phase_for_application(&world.application),
            &world,
            &principal,
            &accounts[1],
            &request,
            replacements[1],
        );
        let (left, right) = std::thread::scope(|scope| {
            let left = scope.spawn(|| {
                world.application.compare_and_commit_application(
                    first,
                    keys[0],
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
            });
            let right = scope.spawn(|| {
                world.application.compare_and_commit_application(
                    second,
                    keys[1],
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
            });
            (left.join().unwrap(), right.join().unwrap())
        });
        if let (
            WorthQueryApplicationCommitOutcome::Committed(left),
            WorthQueryApplicationCommitOutcome::Committed(right),
        ) = (&left, &right)
        {
            assert_ne!(
                left.committed_product_publication(),
                right.committed_product_publication()
            );
            assert_eq!(commit_count(), baseline + 2);
            let current = world.selected_product(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
            );
            assert!(
                current.product().relational_basis_descriptor() == left.basis_descriptor()
                    || current.product().relational_basis_descriptor() == right.basis_descriptor()
            );
            return;
        }
        let (winner, loser, loser_index) = match (left, right) {
            (WorthQueryApplicationCommitOutcome::Committed(winner), loser) => (winner, loser, 1),
            (loser, WorthQueryApplicationCommitOutcome::Committed(winner)) => (winner, loser, 0),
            pair => panic!("same-head race must produce exactly one performed product: {pair:?}"),
        };
        let current = world.selected_product(
            &(host_custody.phase_for_application(&world.application)).test_host_request(),
        );
        assert_ne!(
            current.product().selected_commit(),
            selected.product().selected_commit()
        );
        assert_eq!(
            current.product().relational_basis_descriptor(),
            winner.basis_descriptor()
        );
        match loser {
        WorthQueryApplicationCommitOutcome::ProductStale(stale) => {
            assert_eq!(
                stale.expected_product().selected_commit(),
                selected.product().selected_commit()
            );
            if let Some(observed) = stale.observed_product() {
                assert_eq!(
                    observed.selected_commit(),
                    current.product().selected_commit()
                );
            }
            assert_eq!(
                commit_count(),
                baseline + 1,
                "no-effect loser must leave no lower commit"
            );
            let readmitted = admitted_program(&host_custody.phase_for_application(&world.application),
                &world,
                &principal,
                &accounts[loser_index],
                &request,
                replacements[loser_index],
            );
            let outcome = world
                .application
                .compare_and_commit_application(readmitted, keys[loser_index], crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation);
            let WorthQueryApplicationCommitOutcome::Committed(fresh) = outcome else {
                panic!("explicit fresh admission must commit the unchanged independent facts: {outcome:?}");
            };
            assert_ne!(
                fresh.committed_product_publication(),
                winner.committed_product_publication()
            );
            let after = world.selected_product(&(host_custody.phase_for_application(&world.application)).test_host_request(), );
            assert_eq!(
                after.product().relational_basis_descriptor(),
                fresh.basis_descriptor()
            );
            assert_eq!(commit_count(), baseline + 2);
        }
        WorthQueryApplicationCommitOutcome::Denied(denial)
            if denial.kind()
                == crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind::ProductBasisStale
                && denial.stage()
                    == crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::InvariantExecution =>
        {
            assert_eq!(
                commit_count(),
                baseline + 1,
                "pre-effect product-basis loser must leave no lower commit"
            );
            let readmitted = admitted_program(&host_custody.phase_for_application(&world.application),
                &world,
                &principal,
                &accounts[loser_index],
                &request,
                replacements[loser_index],
            );
            let outcome = world
                .application
                .compare_and_commit_application(readmitted, keys[loser_index], crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation);
            let WorthQueryApplicationCommitOutcome::Committed(fresh) = outcome else {
                panic!("fresh admission after pre-effect product staleness must commit: {outcome:?}");
            };
            assert_ne!(
                fresh.committed_product_publication(),
                winner.committed_product_publication()
            );
            assert_eq!(commit_count(), baseline + 2);
        }
        WorthQueryApplicationCommitOutcome::ProductUnpublished(partial) => {
            assert_eq!(
                partial.cause(),
                worth_runtime_world::facade::ProductUnpublishedCause::ProductPublicationLost
            );
            assert_eq!(partial.owner_effect_count(), 1);
            assert_eq!(partial.inspect().unwrap().owner_effect_count(), 1);
            assert_eq!(
                commit_count(),
                baseline + 2,
                "retained loser must represent its actual owner movement"
            );
            let after = world.selected_product(&(host_custody.phase_for_application(&world.application)).test_host_request(), );
            assert_eq!(
                after.product().selected_commit(),
                current.product().selected_commit(),
                "retained owner effects cannot move product truth"
            );
        }
        other => {
            panic!("race loser must retain exact no-effect or performed-owner evidence: {other:?}")
        }
    }
    })
}
#[test]
fn unrelated_product_drift_preserves_the_prepared_attempt_and_changed_facts_stale() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let unrelated = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "unrelated", &request)
        })
        .expect("the fixture read owns its installed host call");
    let selected = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.selected_product(&(phase).test_host_request())
        })
        .expect("the fixture read owns its installed host call");
    let commit_count = || {
        world
            .application
            .primary_provider
            .graph
            .with_runtime(request_execution, |runtime| {
                runtime.history().immutable_commit_count()
            })
    };
    let baseline = commit_count();

    let first = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(&phase, &world, &principal, &account, &request, "first")
        })
        .expect("the fixture read owns its installed host call");
    let fresh = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(&phase, &world, &principal, &account, &request, "first")
        })
        .expect("the fixture read owns its installed host call");
    let losing = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(&phase, &world, &principal, &account, &request, "losing")
        })
        .expect("the fixture read owns its installed host call");
    let unrelated_program = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(
                &phase,
                &world,
                &principal,
                &unrelated,
                &request,
                "unrelated-after",
            )
        })
        .expect("the fixture read owns its installed host call");

    let unrelated_outcome = world.application.compare_and_commit_application(
        unrelated_program,
        idempotency(1, 1),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        matches!(
            unrelated_outcome,
            WorthQueryApplicationCommitOutcome::Committed(_)
        ),
        "unexpected unrelated outcome: {unrelated_outcome:?}"
    );
    let current = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.selected_product(&(phase).test_host_request())
        })
        .expect("the fixture read owns its installed host call");
    assert_ne!(
        selected.product().selected_commit(),
        current.product().selected_commit()
    );
    assert_eq!(commit_count(), baseline + 1);
    let outcome = world.application.compare_and_commit_application(
        first,
        idempotency(2, 2),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(matches!(
        outcome,
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_ne!(
                world
                    .selected_product(&(phase).test_host_request(),)
                    .product()
                    .selected_commit(),
                current.product().selected_commit()
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(commit_count(), baseline + 2);
    assert!(matches!(
        world.application.compare_and_commit_application(
            fresh,
            idempotency(2, 2),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
    ));
    assert_eq!(commit_count(), baseline + 2);
    let outcome = world.application.compare_and_commit_application(
        losing,
        idempotency(3, 3),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    super::assert_changed_decision(outcome, "the sealed status changed");
    assert_eq!(commit_count(), baseline + 2);
}
