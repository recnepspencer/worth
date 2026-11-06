use crate::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::affinity_tests::*;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as Cause;

#[test]
fn prepared_running_refuses_foreign_phase_with_the_same_typed_cause() {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    let foreign =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    let prepared = world
        .application
        .with_host_advancement(|phase| {
            let preparation = prepare_application_commit(
                &phase.borrowed_request_for_test(),
                WorthQueryApplicationCommitPreparationRequest::new_for_test(
                    world
                        .application
                        .validate_commit_program_in_advancement_for_test(
                            &phase,
                            retained_program( &(&phase).test_host_request_for_test(), &world, "foreign-phase"),
                        )
                        .unwrap_or_else(|outcome| panic!("fixture admission: {outcome:?}")),
                    idempotency(197, 198),
                    None,
                    None,
                ),
            );
            let WorthQueryApplicationCommitPreparation::Ready(prepared) = preparation else {
                panic!("the real owner prepares its retained candidate");
            };
            prepared
        })
        .unwrap();
    foreign.application.with_host_advancement(|phase| {
        let outcome = start_managed_application_commit(&phase, &world.application, prepared).err().unwrap();
        let WorthQueryApplicationCommitOutcome::Denied(refusal) = outcome else {
            panic!("foreign phase cannot start a prepared run");
        };
        assert_eq!(refusal.execution_denial_cause(), Some(Ok(Cause::ExecutionResource {
            denial: Resource::ForeignAdvancementPhase, partition_identity: None, policy_ancestor: None,
        })));
        assert_eq!(refusal.stage(), worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage::ResourceAdmission);
    }).unwrap();
}
