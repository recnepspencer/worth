//! Independent host calls used to observe the state after a fixture operation.
use super::*;

pub(in super::super) fn retain_read<Schema: ApplicationSchema + 'static>(
    application: &primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        '_,
        '_,
        '_,
        Schema,
    >,
) -> Result<
    worth_query_host::facade::application_entry::WorthQueryApplicationReadObservation,
    primary_graph::WorthQueryProductBranchAdmissionDenial,
> {
    let cancellation = authentication::WorthQueryCancellationSource::new();
    let scope = authentication::WorthQueryRequestScope::new(
        std::time::Instant::now() + std::time::Duration::from_secs(120),
        cancellation.token(),
    );
    application
        .with_application_advancement(&scope, |phase| request.retain_read(&phase))
        .expect("the fixture observation host policy admits its call")
}

pub(in super::super) fn host_call<Schema: ApplicationSchema + 'static, Output>(
    application: &primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    operation: impl FnOnce(
        &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    ) -> Output,
) -> Output {
    let cancellation = authentication::WorthQueryCancellationSource::new();
    let scope = authentication::WorthQueryRequestScope::new(
        std::time::Instant::now() + std::time::Duration::from_secs(120),
        cancellation.token(),
    );
    application
        .with_application_advancement(&scope, |phase| operation(&phase))
        .expect("the fixture host policy admits its call")
}
