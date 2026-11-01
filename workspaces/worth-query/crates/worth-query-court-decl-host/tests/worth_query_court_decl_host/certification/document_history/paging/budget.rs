use super::*;
use worth_query_host::facade::{
    primary_graph::{
        bound_advancement_requests_on_this_thread_for_test,
        place_managed_computations_on_this_thread_for_test, WorthQueryExecutionObservation,
        WorthQueryExecutionPlacementForTest,
    },
    runtime::ExecutionBudget,
};
use worth_query_testkit_document_retention::facade::document_retention_model::document_history::DetailedRetentionHistory;

#[test]
fn page_backing_refusal_keeps_the_original_cursor_for_a_funded_retry() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = publish_on_first_program();
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let owned = head(&host);
    let document = host
        .with_host_advancement(|phase| {
            host.runtime()
                .request(&principal, &scope)
                .on_branch(owned.branch())
                .attach_history::<DetailedRetentionHistory>(
                    &phase,
                    owned,
                    DocumentRetentionRead {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                    },
                )
                .unwrap()
        })
        .unwrap();
    let (document, expected) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_9030),
        ),
        3,
    );
    let cursor = document
        .history_page(&request, limits(1), None)
        .unwrap()
        .continuation()
        .unwrap();
    assert!(matches!(
        page_failure(
            document
                .history_page(&request, limits(1), Some(&cursor))
                .unwrap_err()
        ),
        PageFailure::StorageLimitExceeded
    ));
    let roomy = Limits {
        maximum_owned_bytes: NonZeroUsize::new(128 * 1024).unwrap(),
        ..limits(1)
    };
    let previous = place_managed_computations_on_this_thread_for_test(
        WorthQueryExecutionPlacementForTest::Serial,
    );
    let prior_budget = bound_advancement_requests_on_this_thread_for_test(Some(
        ExecutionBudget::new(NonZeroUsize::MIN, Some(40 * 1024), None),
    ));
    let denied = document.history_page(&request, roomy, Some(&cursor));
    bound_advancement_requests_on_this_thread_for_test(prior_budget);
    place_managed_computations_on_this_thread_for_test(previous);
    let cause = page_failure(denied.unwrap_err());
    assert!(
        matches!(
            cause,
            PageFailure::Request(worth_execution::WorkCeilingDenial::Admission(
                worth_execution::LeaseDenial::MemoryExhausted(_)
            ))
        ),
        "actual original request memory refusal: {cause:?}"
    );
    let page = document
        .history_page(&request, roomy, Some(&cursor))
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    assert_page_entry(page.rows()[0].step(), &expected);
    assert!(page.continuation().is_none());
    close(&host, document, 2);
}
