use super::*;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryManagedComputationResourceDenial, WorthQueryMemoryLimitLevel,
    },
    application_entry::{
        WorthQueryApplicationHistoryDenial, WorthQueryDocumentHistoryDenial,
        WorthQueryHistoryEditDenial,
    },
    primary_graph::{
        bound_advancement_requests_on_this_thread_for_test,
        place_managed_computations_on_this_thread_for_test, relational_execution_kind,
        WorthQueryExecutionObservation, WorthQueryExecutionPlacementForTest,
        WorthQueryProviderSessionDenialKind,
    },
    runtime::ExecutionBudget,
};
use worth_query_testkit_document_retention::facade::document_retention_model::document_history::{
    detailed_edit_label, DetailedRetentionHistory,
};

#[test]
fn record_capacity_refusal_precedes_candidate_work_and_preserves_a_lawful_retry() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = publish_on_first_program();
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let initial = head(&host);
    let document = host
        .with_host_advancement(|phase| {
            host.runtime()
                .request(&principal, &scope)
                .on_branch(initial.branch())
                .attach_history::<DetailedRetentionHistory>(
                    &phase,
                    initial,
                    DocumentRetentionRead {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                    },
                )
                .unwrap()
        })
        .unwrap();
    let root = document.current_step(&request).unwrap();
    let edit = document
        .edit(SetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 3,
            },
        })
        .idempotency_owned(&request, 0x9182_0460)
        .unwrap()
        .retain_in_program::<RetentionProgramP0, Action>(&host)
        .unwrap_or_else(|(_, cause)| panic!("declared action: {cause:?}"));
    let previous = place_managed_computations_on_this_thread_for_test(
        WorthQueryExecutionPlacementForTest::Serial,
    );
    let prior_budget = bound_advancement_requests_on_this_thread_for_test(Some(
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(40 * 1024), None),
    ));
    let outcome = edit.advance(&request);
    bound_advancement_requests_on_this_thread_for_test(prior_budget);
    place_managed_computations_on_this_thread_for_test(previous);
    let Advance::Refused { edit, denial } = outcome else {
        panic!("fixed budget must reach actual recording capacity refusal")
    };
    let WorthQueryHistoryEditDenial::History(WorthQueryApplicationHistoryDenial::Document(
        WorthQueryDocumentHistoryDenial::Recording(cause),
    )) = denial
    else {
        panic!("fixture refused at the wrong boundary: {denial:?}")
    };
    assert!(matches!(relational_execution_kind(cause, None), Ok(
        WorthQueryProviderSessionDenialKind::ExecutionResource {
            denial: WorthQueryManagedComputationResourceDenial::MemoryLimit {
                requested, admitted, level: WorthQueryMemoryLimitLevel::Policy,
            }, ..
        }) if requested > admitted));
    assert_eq!(edit.posture(), Posture::AwaitingAdmission);
    assert!(edit.source_operation().is_some());
    assert!(edit.completed_source().is_none());
    assert_eq!(edit.read(&request).unwrap().rows()[0].retention_days, 7);
    let (document, step) = finish(&host, ready(&host, edit), 3);
    assert_eq!(step.parent(), Some(root.id()));
    assert_eq!(step.label(), Some(detailed_edit_label().as_str()));
    close(&host, document, 2);
}
