//! Capacity and identity are issued by the existing admission/session owners.
use super::ProviderExecutionDenial as Denial;
use crate::domain_computation::{
    primary_graph::{
        application_query::one_shot::outcome::material::MaterialCompletion,
        source_facts::SourceFactsAllocation, WorthQueryPrimaryGraphApplicationRuntime,
    },
    provider_session::{
        WorthQueryCompletedSourceRead, WorthQueryManagedGraphWorkSession,
        WorthQueryManagedIndexReadSession, WorthQueryPreparedProductReadSessionIdentity,
    },
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_query_admission::{
    facade::authenticated_principal::WorthQueryRequestScope,
    integration::{
        admit_prepared_managed_index_read_graph_work, prepare_managed_index_read_graph_work,
    },
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledManagedIndexParent;

#[allow(clippy::too_many_arguments)]
pub(super) fn start<'a, 'request, 'authority, S: ApplicationSchema>(
    execution: &ExecutionRequest<'request, 'authority>,
    application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
    parent: &'a WorthQueryManagedGraphWorkSession,
    request: &'a WorthQueryRequestScope,
    association: &WorthQueryInstalledManagedIndexParent,
    source: Arc<WorthQueryCompletedSourceRead>,
    allocation: &Arc<SourceFactsAllocation>,
) -> Result<WorthQueryManagedIndexReadSession<'a, 'request, 'authority>, Denial> {
    let storage = MaterialCompletion::new(execution, allocation);
    let mut admit = |work, bytes| {
        storage.work(work)?;
        allocation.claim(execution, bytes)
    };
    let support = application.graph_work_resource_support_ref();
    let prepared = prepare_managed_index_read_graph_work(association, support, &mut admit)
        .map_err(Denial::Capacity)?;
    let admitted =
        admit_prepared_managed_index_read_graph_work(association, &prepared, support, &mut admit)
            .map_err(Denial::Capacity)?;
    let reserved =
        WorthQueryPreparedProductReadSessionIdentity::reserve().map_err(Denial::Identity)?;
    WorthQueryManagedIndexReadSession::start(
        execution,
        parent,
        &application.product_runtime,
        request,
        association.clone(),
        source,
        reserved,
        admitted,
    )
    .map_err(Denial::Session)
}
