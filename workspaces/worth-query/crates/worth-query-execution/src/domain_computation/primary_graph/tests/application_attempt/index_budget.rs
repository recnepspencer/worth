use super::{
    admitted_program, authenticated_principal, idempotency, installed_authorization_world,
    live_scope, resolved_account,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome,
};

#[test]
fn pre_effect_index_budget_denial_leaves_no_effect_and_retries_through_query() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let denied = admitted_program(&world, &principal, &account, &request, "budget-replacement");

    // Constrain only the next ordinary preflight. The real Relational candidate
    // and index preparation still run; no World publication is faulted.
    world.faults.constrain_next_index_maintenance_budget();
    let WorthQueryApplicationCommitOutcome::Denied(denial) = world
        .application
        .compare_and_commit_application(denied, idempotency(41, 41))
    else {
        panic!("index work exhaustion must deny before a World effect");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::IndexMaintenanceBudgetExceeded
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::ProviderCommit
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .published_application_commit_count(),
        0
    );
    let _still_open = resolved_account(&world, "open", &live_scope());

    let retry = admitted_program(&world, &principal, &account, &request, "budget-replacement");
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = world
        .application
        .compare_and_commit_application(retry, idempotency(41, 41))
    else {
        panic!("the identical intent must commit when ordinary index work is admitted");
    };
    let index_work = receipt
        .mutation_work()
        .expect("the committed receipt retains mutation work")
        .index_maintenance_work();
    assert!(index_work.work_units > 0);
    assert_eq!(index_work.cold_record_slots, 0);
    assert!(index_work.generation_publications_reserved > 0);
    let _committed = resolved_account(&world, "budget-replacement", &live_scope());
}
