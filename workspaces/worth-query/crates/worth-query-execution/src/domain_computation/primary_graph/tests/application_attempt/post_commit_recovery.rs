use super::super::fixture::installed_authorization_world_with_resource_profile;
use super::{
    admitted_program, admitted_program_on_selected, admitted_program_with_emit,
    authenticated_principal, idempotency, installed_authorization_world, live_scope,
    resolved_account,
};
use crate::domain_computation::execution_runtime::WorthQueryApplicationQueryResourceProfile;
use crate::domain_computation::primary_graph::WorthQueryAdmittedChange;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome;

#[test]
fn first_post_commit_admission_failure_retains_exact_idempotent_recovery_evidence() {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let _live = world
        .application
        .primary_provider
        .observe_application_commit_causality(selected.product());
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let published_snapshot_baseline =
        world
            .application
            .primary_provider
            .graph
            .with_runtime(|runtime| {
                runtime
                    .storage_access()
                    .storage_stats()
                    .published_snapshot_handle_count
            });
    let first = admitted_program_with_emit(
        &world,
        &principal,
        &account,
        &request,
        "post-commit-recovery",
        Some("post-commit-effect"),
    );
    let retry = admitted_program(
        &world,
        &principal,
        &account,
        &request,
        "post-commit-recovery",
    );

    world.faults.fail_next_post_commit_snapshot();
    let WorthQueryApplicationCommitOutcome::Committed(first_receipt) = world
        .application
        .compare_and_commit_application(first, idempotency(196, 197))
    else {
        panic!("exact committed evidence must recover the post-commit admission failure");
    };
    assert_eq!(
        world.faults.failed_post_commit_snapshot_consumption_count(),
        1,
        "the injected post-commit snapshot failure must be consumed"
    );
    assert!(
        !world
            .application
            .primary_provider
            .has_pending_application_publication_for_test(),
        "automatic recovery must clear the retained publication state"
    );
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) = world
        .application
        .compare_and_commit_application(retry, idempotency(196, 197))
    else {
        panic!("equivalent retry must resolve the exact committed evidence");
    };
    assert!(recovered.is_same_authoritative_commit(&first_receipt));
    assert_eq!(first_receipt.emitted_effect_count(), 1);
    let emissions = world
        .application
        .primary_provider
        .committed_application_emissions(first_receipt.committed_product_publication());
    assert_eq!(emissions.len(), 1);
    assert_eq!(
        emissions[0].payload::<String>().map(String::as_str),
        Some("post-commit-effect")
    );
    assert!(world
        .application
        .primary_provider
        .retained_application_commit_basis(first_receipt.commit_reference())
        .is_some());
    let bridge_head = world
        .application
        .primary_provider
        .graph
        .relational_bridge_source()
        .load_branch_head_patch(
            &crate::domain_computation::primary_graph::primary_truth_branch_identity(),
        )
        .expect("post-commit recovery binds the exact Bridge head");
    assert_eq!(
        bridge_head.commit_identity(),
        &worth_runtime_bridge::facade::TruthCommitIdentity::from_relational_commit_id(
            first_receipt.commit_reference().commit_id.0,
        )
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .graph
            .with_runtime(|runtime| {
                runtime
                    .storage_access()
                    .storage_stats()
                    .published_snapshot_handle_count
            }),
        published_snapshot_baseline,
        "post-commit recovery must release every temporary snapshot owner"
    );
}

#[test]
fn recovery_capacity_denies_before_any_selected_product_head_moves() {
    let resources =
        WorthQueryApplicationQueryResourceProfile::bounded(5_120, 2_048, usize::MAX, 2).unwrap();
    let world = installed_authorization_world_with_resource_profile(resources);
    let source = world.application.current_world();
    let sibling_a_branch = world
        .application
        .branches()
        .fork(source)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the first real sibling product must publish");
    let sibling_b_branch = world
        .application
        .branches()
        .fork(source)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the second real sibling product must publish");
    let root = world.selected_product();
    let sibling_a = world
        .application
        .on_branch(sibling_a_branch)
        .select()
        .expect("the first sibling remains selectable");
    let sibling_b = world
        .application
        .on_branch(sibling_b_branch)
        .select()
        .expect("the second sibling remains selectable");
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let world_head_before = sibling_b.product().selected_commit().clone();
    let relational_head_before = sibling_b
        .product()
        .observation()
        .basis()
        .relational_basis()
        .descriptor()
        .clone();
    let relational_branch = relational_head_before.branch_id().clone();
    let publication_cost_scope = world
        .application
        .primary_provider
        .graph
        .with_runtime(|runtime| {
            let identity = runtime
                .branch_identity(&relational_branch)
                .expect("the sibling's exact Relational branch remains installed");
            worth_relational::facade::inspection::RelationalMvccCostScope::capture(
                runtime,
                vec![identity],
            )
        });
    let provider_publications_before = world
        .application
        .primary_provider
        .published_application_commit_count();

    let root_slot = world
        .application
        .primary_provider
        .reserve_application_publication_recovery(root.product().observation())
        .expect("total minus one recovery slot remains admissible");
    let sibling_a_slot = world
        .application
        .primary_provider
        .reserve_application_publication_recovery(sibling_a.product().observation())
        .expect("the final installed recovery slot remains admissible");
    let denied_program = admitted_program_on_selected(
        &world,
        &sibling_b,
        &principal,
        &account,
        &request,
        "recovery-capacity-denied",
    );
    let denied = world
        .application
        .on_branch(sibling_b_branch)
        .transaction()
        .apply(WorthQueryAdmittedChange::new(
            denied_program,
            idempotency(200, 201),
        ))
        .commit()
        .expect("the public product token remains admitted");
    assert!(matches!(
        denied,
        WorthQueryApplicationCommitOutcome::Aborted
    ));
    assert_eq!(
        world
            .application
            .primary_provider
            .pending_application_publication_count_for_test(),
        2,
        "the denied attempt must not acquire or disturb either retained slot"
    );
    let after_denial = world
        .application
        .on_branch(sibling_b_branch)
        .select()
        .expect("capacity denial leaves the sibling selectable");
    assert_eq!(after_denial.product().selected_commit(), &world_head_before);
    assert_eq!(
        after_denial
            .product()
            .observation()
            .basis()
            .relational_basis()
            .descriptor(),
        &relational_head_before,
        "recovery capacity must deny before the Relational head moves"
    );
    let publication_cost = world
        .application
        .primary_provider
        .graph
        .with_runtime(|runtime| {
            runtime
                .observe_mvcc_counters(&publication_cost_scope)
                .expect("the exact sibling cost scope remains observable")
        });
    assert_eq!(
        publication_cost.sharing_cost_delta().publication_attempts,
        0,
        "capacity denial must precede Relational publication owner contact"
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .published_application_commit_count(),
        provider_publications_before,
        "capacity denial must publish no application causality"
    );

    drop(root_slot);
    let fresh = admitted_program_on_selected(
        &world,
        &after_denial,
        &principal,
        &account,
        &request,
        "recovery-capacity-denied",
    );
    let committed = world
        .application
        .on_branch(sibling_b_branch)
        .transaction()
        .apply(WorthQueryAdmittedChange::new(fresh, idempotency(200, 201)))
        .commit()
        .expect("the public product token remains admitted")
        .require_committed()
        .expect("releasing one slot must admit and commit a fresh exact twin");
    assert_ne!(
        committed.committed_product_publication().composite_commit(),
        &world_head_before
    );
    drop(committed);
    drop(sibling_a_slot);
    assert_eq!(
        world
            .application
            .primary_provider
            .pending_application_publication_count_for_test(),
        0
    );
    drop(root);
    drop(sibling_a);
    drop(sibling_b);
    drop(after_denial);
    assert!(world
        .application
        .on_branch(sibling_a_branch)
        .close()
        .unwrap()
        .is_complete());
    assert!(world
        .application
        .on_branch(sibling_b_branch)
        .close()
        .unwrap()
        .is_complete());
}

#[test]
fn unwind_after_performed_publication_preserves_one_exact_retriable_record() {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let _live = world
        .application
        .primary_provider
        .observe_application_commit_causality(selected.product());
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let before = selected.product().selected_commit().clone();
    let first = admitted_program_with_emit(
        &world,
        &principal,
        &account,
        &request,
        "post-performed-unwind",
        Some("post-performed-effect"),
    );
    let retry = admitted_program(
        &world,
        &principal,
        &account,
        &request,
        "post-performed-unwind",
    );

    world.faults.panic_next_pending_application_publication();
    let WorthQueryApplicationCommitOutcome::Committed(first_receipt) = world
        .application
        .compare_and_commit_application(first, idempotency(198, 199))
    else {
        panic!("the public commit journey must rediscover and finish the exact retained record");
    };
    assert_eq!(
        world
            .faults
            .panicked_pending_publication_consumption_count(),
        1,
        "the post-Performed unwind must occur exactly once"
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .pending_application_publication_count_for_test(),
        0,
        "the internal exact retry must retire the recovered slot"
    );
    assert_ne!(
        world.selected_product().product().selected_commit(),
        &before,
        "World must already expose the performed successor"
    );

    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) = world
        .application
        .compare_and_commit_application(retry, idempotency(198, 199))
    else {
        panic!("the equivalent retry must recover the exact performed publication");
    };
    assert!(recovered.is_same_authoritative_commit(&first_receipt));
    assert_eq!(recovered.emitted_effect_count(), 1);
    assert_eq!(
        world
            .application
            .primary_provider
            .pending_application_publication_count_for_test(),
        0,
        "terminal recovery must disarm and retire the exact slot"
    );
    let emissions = world
        .application
        .primary_provider
        .committed_application_emissions(recovered.committed_product_publication());
    assert_eq!(emissions.len(), 1);
    assert_eq!(
        emissions[0].payload::<String>().map(String::as_str),
        Some("post-performed-effect")
    );
}
use worth_runtime_bridge::facade::TruthBranchHeadSource;
