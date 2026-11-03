//! Cancel the actual Native traversal through its borrowed Query control.
use crate::domain_computation::primary_graph::application_query::read_execution::interruption::tests::*;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::declared_queries::declarations::Forward;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_walks::declared_queries::read;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_walks::world;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_walks::world::Graph;
#[test]
fn query_caller_cancel_interrupts_complete_native_closure_and_preserves_original_request() {
    let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_walks::world::install(Graph {
        nodes: 1001,
        edges: (0..1000).map(|n| (n, n + 1)).collect(),
        active: false,
        blocked: vec![],
    });
    let host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(None, cancellation.token());
    let visited = Rc::new(Cell::new(0));
    let witness = Rc::clone(&visited);
    CHECKPOINT.with(|slot| {
        *slot.borrow_mut() = Some(Box::new(move |subject| {
            if subject == "Next" {
                witness.set(witness.get() + 1);
                if witness.get() == 25 {
                    cancellation.cancel();
                }
            }
        }))
    });
    let guard = CheckpointGuard;
    let refusal = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_walks::declared_queries::read::execute(&world, &execution, &request, Forward::reference(), 0)
        .err()
        .expect("caller cancellation refuses whole result");
    assert_eq!(
        refusal.kind(),
        WorthQueryApplicationOneShotDenialKind::Cancelled
    );
    assert!(
        (25..=27).contains(&visited.get()),
        "the complete walk stops promptly after the real caller cancellation"
    );
    drop(guard);
    let live = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let twin = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_walks::declared_queries::read::execute(&world, &execution, &live, Forward::reference(), 0).unwrap();
    assert_eq!(
        twin.rows().len(),
        1001,
        "the same original execution request remains live"
    );
}
