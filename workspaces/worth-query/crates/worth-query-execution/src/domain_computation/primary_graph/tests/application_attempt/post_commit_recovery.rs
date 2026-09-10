use super::{
    admitted_program, admitted_program_with_emit, authenticated_principal, idempotency,
    installed_authorization_world, live_scope, resolved_account,
};
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
use worth_runtime_bridge::facade::TruthBranchHeadSource;
