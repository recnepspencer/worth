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
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_capability_program(&phase, &world, &principal, &request, "denied").0
        })
        .expect("the fixture read owns its installed host call");

    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            revoke_parent(&phase, &world)
        })
        .expect("the fixture read owns its installed host call");

    assert_decision_read_set_denial(world.application.compare_and_commit_application(
        program,
        idempotency(71, 71),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    ));
}

#[test]
fn parent_revocation_denies_idempotency_receipt_inspection() {
    let world = installed_delegated_capability_world();
    world.authorization_time.script(vec![time(100); 32]);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let first = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_capability_program(&phase, &world, &principal, &request, "committed").0
        })
        .expect("the fixture read owns its installed host call");
    let retry = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_capability_program(&phase, &world, &principal, &request, "committed").0
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryApplicationCommitOutcome::Committed(_) =
        world.application.compare_and_commit_application(
            first,
            idempotency(72, 72),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the current delegated program must establish the idempotency receipt");
    };

    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            revoke_parent(&phase, &world)
        })
        .expect("the fixture read owns its installed host call");

    assert_decision_read_set_denial(world.application.compare_and_commit_application(
        retry,
        idempotency(72, 72),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    ));
}

pub(super) fn revoke_parent(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &super::super::fixture::AuthorizationWorld,
) {
    update_grant_field(
        phase_custody,
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
