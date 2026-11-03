//! Phase custody wins before unavailable-completion demand affinity.
use worth_query_execution::domain_computation::primary_graph::application_installation::program::demand::unavailable::admit_unavailable_completion;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;

#[test]
fn unavailable_completion_checks_foreign_phase_before_foreign_demand() {
    let owner = installed_authorization_world(true);
    let other = installed_authorization_world(true);
    for (foreign_phase, foreign_demand) in [(true, true), (true, false), (false, true)] {
        let caller = if foreign_phase { &other } else { &owner };
        caller
            .application
            .with_host_advancement(|phase| {
                let mut demand_checks = 0;
                let denial = admit_unavailable_completion(
                    &phase,
                    &owner.application.product_runtime,
                    || {
                        demand_checks += 1;
                        !foreign_demand
                    },
                )
                .unwrap_err();
                let expected = if foreign_phase {
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(
                        WorthQueryAdvancementDenial::Resource(
                            WorthQueryManagedComputationResourceDenial::ForeignAdvancementPhase,
                        ),
                    )
                } else {
                    WorthQueryOutputDemandDenialKind::ForeignDemand
                };
                assert_eq!(denial.kind(), expected);
                assert_eq!(demand_checks, usize::from(!foreign_phase));
            })
            .unwrap();
    }
}
