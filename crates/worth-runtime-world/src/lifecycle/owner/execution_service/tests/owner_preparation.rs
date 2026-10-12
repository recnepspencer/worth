use super::*;

pub(super) fn prepare_both_owners(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    fixture: &RealReferenceFixture,
    owner: &TestOwner,
    expected: &ProductBranchObservation,
    operation_name: &str,
) -> PreparedCompositePublicationWithSignal {
    let cancellation = RuntimeWorldCancellationSource::new();
    RuntimeWorldPreparationService::prepare_publication(
        owner,
        expected.clone(),
        CompositePublicationIntent::with_signal(Some(RelationalTransactionIntent::ordinary()))
            .with_prepared_relational_candidate(
                fixture.prepare_relational_owner_candidate(execution, operation_name),
            ),
        &cancellation.token(),
        None,
    )
    .expect("the exact observed head admits both owner legs")
}
