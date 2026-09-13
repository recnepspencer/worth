use worth_query_installation::facade::ApplicationScalarValueBinding;

use super::super::application_attempt::{authenticated_principal, idempotency};
use super::super::fixture::capability::CapabilityStatusField;
use super::super::fixture::{installed_delegated_capability_world, live_scope, CapabilityStatus};
use super::capability_delegation_mutation::{field, update_grant_field};
use super::capability_progression::{admitted_capability_program, time};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome,
};

#[test]
fn parent_revocation_after_program_admission_denies_final_commit() {
    let world = installed_delegated_capability_world();
    world.authorization_time.script(vec![time(100); 16]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let program = admitted_capability_program(&world, &principal, &request, "denied").0;

    revoke_parent(&world);

    assert_decision_read_set_denial(
        world
            .application
            .compare_and_commit_application(program, idempotency(71, 71)),
    );
}

#[test]
fn parent_revocation_denies_idempotency_receipt_inspection() {
    let world = installed_delegated_capability_world();
    world.authorization_time.script(vec![time(100); 32]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let first = admitted_capability_program(&world, &principal, &request, "committed").0;
    let retry = admitted_capability_program(&world, &principal, &request, "committed").0;
    let WorthQueryApplicationCommitOutcome::Committed(_) = world
        .application
        .compare_and_commit_application(first, idempotency(72, 72))
    else {
        panic!("the current delegated program must establish the idempotency receipt");
    };

    revoke_parent(&world);

    assert_decision_read_set_denial(
        world
            .application
            .compare_and_commit_application(retry, idempotency(72, 72)),
    );
}

fn revoke_parent(world: &super::super::fixture::AuthorizationWorld) {
    update_grant_field(
        world,
        "capability-parent",
        field(world, CapabilityStatusField::reference()),
        super::super::fixture::CapabilityStatusBinding::encode(&CapabilityStatus::Revoked).unwrap(),
    );
}

fn assert_decision_read_set_denial(outcome: WorthQueryApplicationCommitOutcome) {
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("stale parent authority must disclose neither commit nor idempotency receipt");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProviderRejected,
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::DecisionReadSet,
    );
}
