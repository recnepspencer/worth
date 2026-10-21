//! Exact selected mutation graph work on one carried request admission.

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_admission::integration::admit_prepared_application_operation_graph_work;
use worth_query_admission::integration::WorthQueryPreparedApplicationOperationGraphWork;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_installation::facade::ApplicationSchema;
use worth_query_installation::facade::WorthQueryCurrentRetainedMutationBinding;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use crate::domain_computation::primary_graph::SharedSelectedProductOperation;
use crate::domain_computation::primary_graph::WorthQueryApplicationSnapshotLease;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::provider_session::WorthQueryGraphWorkAccessContextAffinity;
use crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession;

pub(in crate::domain_computation) enum SelectedOperationGraphWorkStop {
    ForeignSelectedRuntime,
    ForeignInstalledIssuer,
    Interrupted(WorthQueryRequestInterruption),
    Source(CompanionPreflightStop),
    Capacity,
    Session,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation) fn start_selected_operation_graph_work_admitted<Schema, Binding>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    shared: &SharedSelectedProductOperation<'_, Schema>,
    current: &WorthQueryCurrentRetainedMutationBinding<'_, Schema, Binding>,
    prepared: &WorthQueryPreparedApplicationOperationGraphWork,
    resource_binding_identity: &str,
    principal: EntityId,
    access: WorthQueryGraphWorkAccessContextAffinity,
    request: &WorthQueryRequestScope,
) -> Result<WorthQueryManagedGraphWorkSession, SelectedOperationGraphWorkStop>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    if !std::ptr::eq(shared.selected().application(), runtime) {
        return Err(SelectedOperationGraphWorkStop::ForeignSelectedRuntime);
    }
    if !current.belongs_to(
        runtime.runtime.installed_packages(),
        runtime.installed_schema(),
    ) {
        return Err(SelectedOperationGraphWorkStop::ForeignInstalledIssuer);
    }
    // Wrapper plumbing below reads the issued operation, actual provider
    // support, binding and obligation identities, and copies the eight-byte
    // runtime authority. The session owner's later claim funds its own work.

    let operation = current.operation();
    check_request(request)?;
    let lease = WorthQueryApplicationSnapshotLease::from_shared_selected(shared)
        .map_err(SelectedOperationGraphWorkStop::Source)?;
    let plan = admit_prepared_application_operation_graph_work(
        operation,
        prepared,
        resource_binding_identity,
        runtime.graph_work_resource_support_ref(),
    )
    .map_err(|_| SelectedOperationGraphWorkStop::Capacity)?;
    check_request(request)?;
    let session = WorthQueryManagedGraphWorkSession::start_mutation(
        plan,
        runtime.runtime.authority_identity(),
        operation.binding_identity(),
        operation.graph_obligations().identity(),
        operation.authority_identity(),
        principal,
        access,
        lease,
        runtime.graph_work_provider_identity(),
    )
    .map_err(|_| SelectedOperationGraphWorkStop::Session)?;
    check_request(request)?;
    Ok(session)
}

fn check_request(request: &WorthQueryRequestScope) -> Result<(), SelectedOperationGraphWorkStop> {
    if let Some(interruption) = request.interruption() {
        Err(SelectedOperationGraphWorkStop::Interrupted(interruption))
    } else {
        Ok(())
    }
}
