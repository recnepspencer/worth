//! Refusal after graph execution preserves the accumulated door counters.
use super::*;
#[path = "foreign_phase_fixture.rs"]
mod fixture;
#[test]
fn direct_executor_refuses_foreign_phase_with_accumulated_counters() {
    let _observation = worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let host = crate::bridge_host_request::serial_request();
    let request = worth_execution::ExecutionRequest::serial(&host);
    let mut receiver = fixture::workspace(&request);
    let foreign = fixture::workspace(&request);
    let operation = fixture::admitted(&receiver);
    let owner = receiver.advancement_owner();
    let completed = owner
        .with_advancement(|phase| {
            let prepared = match WorthQueryPreparedDirectExecution::prepare(
                &phase,
                operation,
                &mut receiver,
            ) {
                Ok(value) => value,
                Err(_) => panic!("honest preparation"),
            };

            match prepared.invoke_graphs(&phase) {
                Ok(value) => value,
                Err(_) => panic!("honest graph execution"),
            }
        })
        .unwrap();
    let counters = completed.prepared.counters;
    assert_eq!(counters.runtime_authority_checks, 1);
    // This direct declaration requires no graph provider. Preparation already
    // performed its one runtime authority check, so default counters differ.
    assert_eq!(counters.graph_provider_contacts, 0);
    // A phase issued by the other real owner enters the executor door.
    foreign
        .advancement_owner()
        .with_advancement(|foreign_phase| {
            let TransitionOutcome::Denied(denial) =
                completed.invoke_executor(&foreign_phase, &mut receiver)
            else {
                panic!("foreign executor phase");
            };
            assert_eq!(denial.counters(), counters);
            use worth_query_execution::facade::application_contribution::{
                WorthQueryAdvancementDenial as Denial,
                WorthQueryManagedComputationResourceDenial as Resource,
            };
            assert_eq!(
                denial.kind(),
                &WorthQueryBoundExecutionDenialKind::ExecutionRequest(Denial::Resource(
                    Resource::ForeignAdvancementPhase
                ))
            );
        })
        .unwrap();
}
