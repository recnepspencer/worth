//! Applied provider work uses the same installed capacity and session issuer.
use super::*;
use crate::domain_computation::provider_session::{
    ManagedIndexReadSource, WorthQueryAppliedSourceRead,
};

pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn start<
    'a,
    'request,
    'authority,
    S: ApplicationSchema,
>(
    execution: &ExecutionRequest<'request, 'authority>,
    application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
    parent: &'a WorthQueryManagedGraphWorkSession,
    request: &'a WorthQueryRequestScope,
    association: &WorthQueryInstalledManagedIndexParent,
    source: Arc<WorthQueryAppliedSourceRead>,
    allocation: &Arc<SourceFactsAllocation>,
) -> Result<WorthQueryManagedIndexReadSession<'a, 'request, 'authority>, Denial> {
    super::start_with_source(
        execution,
        application,
        parent,
        request,
        association,
        ManagedIndexReadSource::Applied(source),
        allocation,
    )
}
