//! Source custody survives the result and releases with its last owner.
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::declared_queries::*;
use worth_execution::CancellationSource;
use worth_execution::ExecutionRequest;
use worth_execution::SerialMemoryBudget;
use worth_execution::SerialRequest;
#[test]
fn complete_declared_path_retains_original_memory_until_last_source() {
    let world = diamond(6, vec![]);
    let payer = SerialMemoryBudget::new(None);
    let host = SerialRequest::from_memory(payer.clone(), CancellationSource::new().token(), None);
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let result = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_walks::declared_queries::read::execute(
        &world,
        &ExecutionRequest::serial(&host),
        &request,
        Forward::reference(),
        0,
    )
    .unwrap();
    assert!(payer.charged_memory_bytes() > 0);
    let body = std::sync::Arc::clone(
        &(*result.observed_sources()[0]
            .footprint_for_test()
            .root_selection
            .as_ref()
            .unwrap()
            .walks_for_test())
            .as_ref()
            .unwrap()
            .values()[0],
    );
    let retained = result.observed_sources()[0].clone();
    drop(result);
    assert!(
        payer.charged_memory_bytes() > 0,
        "one real source still retains the funded Native body"
    );
    let with_source_metadata = payer.charged_memory_bytes();
    drop(retained);
    assert!(
        payer.charged_memory_bytes() > 0,
        "the separately retained real Native body remains funded"
    );
    assert!(
        payer.charged_memory_bytes() < with_source_metadata,
        "dropping the last source returns its own metadata backing while the Native body remains"
    );
    drop(body);
    assert_eq!(
        payer.charged_memory_bytes(),
        0,
        "last owner returns original payment"
    );
}
