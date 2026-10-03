//! Exact selected mutation graph work on one carried request admission.

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_query_admission::integration::{
    admit_prepared_application_operation_graph_work_admitted,
    WorthQueryGraphWorkCapacityAdmissionStop, WorthQueryPreparedApplicationOperationGraphWork,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryCurrentRetainedMutationBinding,
};
use worth_relational::facade::{identity::EntityId, mvcc::CompanionPreflightStop};

use crate::domain_computation::primary_graph::{
    InvalidationEditAdmission, SharedSelectedProductOperation, WorthQueryApplicationSnapshotLease,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::provider_session::{
    WorthQueryAdmittedMutationSessionStartStop, WorthQueryGraphWorkAccessContextAffinity,
    WorthQueryManagedGraphWorkSession,
};

pub(in crate::domain_computation) enum SelectedOperationGraphWorkStop {
    ForeignSelectedRuntime,
    ForeignInstalledIssuer,
    Interrupted(WorthQueryRequestInterruption),
    Source(CompanionPreflightStop),
    Capacity(WorthQueryGraphWorkCapacityAdmissionStop<CompanionPreflightStop>),
    Session(WorthQueryAdmittedMutationSessionStartStop<CompanionPreflightStop>),
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
    admission: &mut InvalidationEditAdmission,
) -> Result<WorthQueryManagedGraphWorkSession, SelectedOperationGraphWorkStop>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    admission
        .charge_external_work(3)
        .map_err(SelectedOperationGraphWorkStop::Source)?;
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
    admission
        .charge_external_work(28)
        .map_err(SelectedOperationGraphWorkStop::Source)?;
    let operation = current.operation();
    check_request(request, admission)?;
    let lease = WorthQueryApplicationSnapshotLease::from_shared_selected(shared, admission)
        .map_err(SelectedOperationGraphWorkStop::Source)?;
    let plan = admit_prepared_application_operation_graph_work_admitted(
        operation,
        prepared,
        resource_binding_identity,
        runtime.graph_work_resource_support_ref(),
        &mut |work, bytes| {
            admission.charge_external_work(work)?;
            admission.admit_read_scratch(bytes)
        },
    )
    .map_err(SelectedOperationGraphWorkStop::Capacity)?;
    check_request(request, admission)?;
    let session = WorthQueryManagedGraphWorkSession::start_mutation_admitted(
        plan,
        runtime.runtime.authority_identity(),
        operation.binding_identity(),
        operation.graph_obligations().identity(),
        operation.authority_identity(),
        principal,
        access,
        lease,
        runtime.graph_work_provider_identity(),
        |work, bytes| {
            let bytes = u64::try_from(bytes)
                .map_err(|_| CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
            admission.charge_external_work(work)?;
            admission.admit_read_scratch(bytes)
        },
    )
    .map_err(SelectedOperationGraphWorkStop::Session)?;
    check_request(request, admission)?;
    Ok(session)
}

fn check_request(
    request: &WorthQueryRequestScope,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), SelectedOperationGraphWorkStop> {
    admission
        .charge_external_work(1)
        .map_err(SelectedOperationGraphWorkStop::Source)?;
    if let Some(interruption) = request.interruption() {
        Err(SelectedOperationGraphWorkStop::Interrupted(interruption))
    } else {
        Ok(())
    }
}
