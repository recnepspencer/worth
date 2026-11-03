use worth_query_installation::facade::ApplicationScalarValueBinding;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityStatusField;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_delegated_capability_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityStatus;
use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::field;
use crate::domain_computation::primary_graph::tests::authorization::capability_delegation_mutation::update_grant_field;
use crate::domain_computation::primary_graph::tests::authorization::capability_progression::admitted_capability_program;
use crate::domain_computation::primary_graph::tests::authorization::capability_progression::time;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;

#[test]
fn parent_revocation_after_program_admission_denies_final_commit() {
    let world = installed_delegated_capability_world();
    world.authorization_time.script(vec![time(100); 16]);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_program(&phase, &world, &principal, &request, "denied").0
        })
        .expect("the fixture read owns its installed host call");

    world
        .application
        .with_host_advancement(|phase| revoke_parent(&phase, &world))
        .expect("the fixture read owns its installed host call");

    assert_decision_read_set_denial(world.application.compare_and_commit_application(
        program,
        idempotency(71, 71),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
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
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let first = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_program(&phase, &world, &principal, &request, "committed").0
        })
        .expect("the fixture read owns its installed host call");
    let retry = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_program(&phase, &world, &principal, &request, "committed").0
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryApplicationCommitOutcome::Committed(_) =
        world.application.compare_and_commit_application(
            first,
            idempotency(72, 72),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the current delegated program must establish the idempotency receipt");
    };

    world
        .application
        .with_host_advancement(|phase| revoke_parent(&phase, &world))
        .expect("the fixture read owns its installed host call");

    assert_decision_read_set_denial(world.application.compare_and_commit_application(
        retry,
        idempotency(72, 72),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    ));
}

pub fn revoke_parent(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
) {
    update_grant_field(
        phase_custody,
        world,
        "capability-parent",
        field(world, CapabilityStatusField::reference()),
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityStatusBinding::encode(&CapabilityStatus::Revoked).unwrap(),
    );
}

pub fn assert_decision_read_set_denial(outcome: WorthQueryApplicationCommitOutcome) {
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
