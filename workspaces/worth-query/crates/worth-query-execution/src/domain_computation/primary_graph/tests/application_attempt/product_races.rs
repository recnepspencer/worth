use super::{
    admitted_program, authenticated_principal, idempotency, installed_authorization_world,
    live_scope, resolved_account, WorthQueryApplicationCommitOutcome,
};

#[test]
fn concurrent_independent_attempts_preserve_one_product_winner_and_the_exact_loser_posture() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let accounts = [
        resolved_account(&world, "open", &request),
        resolved_account(&world, "unrelated", &request),
    ];
    let replacements = ["first-independent", "second-independent"];
    let keys = [idempotency(13, 13), idempotency(14, 14)];
    let selected = world.selected_product();
    let commit_count = || {
        world
            .application
            .primary_provider
            .graph
            .with_runtime(|runtime| runtime.history().immutable_commit_count())
    };
    let baseline = commit_count();
    let first = admitted_program(&world, &principal, &accounts[0], &request, replacements[0]);
    let second = admitted_program(&world, &principal, &accounts[1], &request, replacements[1]);
    let (left, right) = std::thread::scope(|scope| {
        let left = scope.spawn(|| {
            world
                .application
                .compare_and_commit_application(first, keys[0])
        });
        let right = scope.spawn(|| {
            world
                .application
                .compare_and_commit_application(second, keys[1])
        });
        (left.join().unwrap(), right.join().unwrap())
    });
    let (winner, loser, loser_index) = match (left, right) {
        (WorthQueryApplicationCommitOutcome::Committed(winner), loser) => (winner, loser, 1),
        (loser, WorthQueryApplicationCommitOutcome::Committed(winner)) => (winner, loser, 0),
        pair => panic!("same-head race must produce exactly one performed product: {pair:?}"),
    };
    let current = world.selected_product();
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
            let readmitted = admitted_program(
                &world,
                &principal,
                &accounts[loser_index],
                &request,
                replacements[loser_index],
            );
            let outcome = world
                .application
                .compare_and_commit_application(readmitted, keys[loser_index]);
            let WorthQueryApplicationCommitOutcome::Committed(fresh) = outcome else {
                panic!("explicit fresh admission must commit the unchanged independent facts: {outcome:?}");
            };
            assert_ne!(fresh.commit_id(), winner.commit_id());
            let after = world.selected_product();
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
            let readmitted = admitted_program(
                &world,
                &principal,
                &accounts[loser_index],
                &request,
                replacements[loser_index],
            );
            let outcome = world
                .application
                .compare_and_commit_application(readmitted, keys[loser_index]);
            let WorthQueryApplicationCommitOutcome::Committed(fresh) = outcome else {
                panic!("fresh admission after pre-effect product staleness must commit: {outcome:?}");
            };
            assert_ne!(fresh.commit_id(), winner.commit_id());
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
            let after = world.selected_product();
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
}
#[test]
fn product_drift_stales_selected_attempt_and_fresh_admission_can_commit_unchanged_facts() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let unrelated = resolved_account(&world, "unrelated", &request);
    let selected = world.selected_product();
    let commit_count = || {
        world
            .application
            .primary_provider
            .graph
            .with_runtime(|runtime| runtime.history().immutable_commit_count())
    };
    let baseline = commit_count();

    let first = admitted_program(&world, &principal, &account, &request, "first");
    let unrelated_program =
        admitted_program(&world, &principal, &unrelated, &request, "unrelated-after");

    let unrelated_outcome = world
        .application
        .compare_and_commit_application(unrelated_program, idempotency(1, 1));
    assert!(
        matches!(
            unrelated_outcome,
            WorthQueryApplicationCommitOutcome::Committed(_)
        ),
        "unexpected unrelated outcome: {unrelated_outcome:?}"
    );
    let current = world.selected_product();
    assert_ne!(
        selected.product().selected_commit(),
        current.product().selected_commit()
    );
    assert_eq!(commit_count(), baseline + 1);
    let outcome = world
        .application
        .compare_and_commit_application(first, idempotency(2, 2));
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!(
            "old Relational basis must fail before effects despite unchanged decision facts: {outcome:?}"
        );
    };
    assert_eq!(
        denial.kind(),
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind::ProductBasisStale
    );
    assert_eq!(
        denial.stage(),
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    let after_stale = world.selected_product();
    assert_eq!(
        after_stale.product().selected_commit(),
        current.product().selected_commit()
    );
    assert_eq!(commit_count(), baseline + 1);
    let fresh = admitted_program(&world, &principal, &account, &request, "first");
    let losing = admitted_program(&world, &principal, &account, &request, "losing");
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(fresh, idempotency(2, 2)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert_eq!(commit_count(), baseline + 2);
    let outcome = world
        .application
        .compare_and_commit_application(losing, idempotency(3, 3));
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("the second old-product attempt must fail before effects: {outcome:?}");
    };
    assert_eq!(
        denial.kind(),
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind::ProductBasisStale
    );
    assert_eq!(
        denial.stage(),
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    assert_eq!(commit_count(), baseline + 2);
}
