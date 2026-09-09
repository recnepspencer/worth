use worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkIntent;
use worth_query_admission::integration::{
    admit_application_operation_graph_work, admit_application_operation_read_graph_work,
    select_installed_graph_obligations,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationOperation,
    WorthQueryInstalledApplicationOperationGraphAuthority,
};
use worth_relational::facade::identity::EntityId;

use crate::domain_computation::primary_graph::{
    application_resource_request, WorthQueryApplicationSnapshotLease,
    WorthQueryApplicationSnapshotLeaseDenial, WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::provider_session::{
    WorthQueryGraphWorkAccessContextAffinity, WorthQueryManagedGraphWorkSession,
};

use super::{WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind};

pub(super) fn start_operation_graph_work<Schema, Operation, Input>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
    resource_binding_identity: &str,
    principal: EntityId,
    access: WorthQueryGraphWorkAccessContextAffinity,
) -> Result<WorthQueryManagedGraphWorkSession, WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
{
    let graph = runtime.runtime.primary_graph().ok_or_else(|| {
        denial(
            WorthQueryOperationAuthorizationDenialKind::ForeignRuntime,
            operation.operation(),
        )
    })?;
    let lease = WorthQueryApplicationSnapshotLease::acquire(
        graph.integration_handle(),
        graph.retain_layout(),
        runtime
            .admit_product_publication()
            .map_err(|_| graph_work_denial(operation.operation()))?,
    )
    .map_err(|denial| snapshot_lease_denial(denial, operation.operation()))?;
    let intent = if operation
        .graph_obligations()
        .rows()
        .iter()
        .any(|row| {
            matches!(
                row.effect_posture(),
                worth_query_installation::facade::WorthQueryInstalledGraphObligationEffectPosture::Mutating
                    | worth_query_installation::facade::WorthQueryInstalledGraphObligationEffectPosture::Invariant
            )
        })
    {
        WorthQueryGraphWorkIntent::application_operation_mutation()
    } else {
        WorthQueryGraphWorkIntent::application_operation_read()
    };
    let obligations = operation.retain_graph_obligations_for_admission();
    let obligation_identity = obligations.identity().clone();
    let selected = select_installed_graph_obligations(obligations, intent)
        .map_err(|_| graph_work_denial(operation.operation()))?;
    let request = application_resource_request(operation.contracts())
        .ok_or_else(|| graph_work_denial(operation.operation()))?;
    let plan = admit_application_operation_graph_work(
        selected,
        resource_binding_identity,
        &request,
        runtime.graph_work_resource_support(),
    )
    .map_err(|_| graph_work_denial(operation.operation()))?;
    WorthQueryManagedGraphWorkSession::start_mutation(
        plan,
        runtime.runtime.authority_identity(),
        operation.binding_identity(),
        &obligation_identity,
        operation.authority_identity(),
        principal,
        access,
        lease,
        runtime.graph_work_provider_identity(),
    )
    .map_err(|_| graph_work_denial(operation.operation()))
}

pub(super) fn start_capability_graph_work<Schema, Operation, Input>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    operation: &WorthQueryInstalledApplicationOperationGraphAuthority<Schema, Operation, Input>,
    principal: EntityId,
    access: WorthQueryGraphWorkAccessContextAffinity,
) -> Result<WorthQueryManagedGraphWorkSession, WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
{
    let graph = runtime.runtime.primary_graph().ok_or_else(|| {
        denial(
            WorthQueryOperationAuthorizationDenialKind::ForeignRuntime,
            operation.operation(),
        )
    })?;
    let lease = WorthQueryApplicationSnapshotLease::acquire(
        graph.integration_handle(),
        graph.retain_layout(),
        runtime
            .admit_product_publication()
            .map_err(|_| graph_work_denial(operation.operation()))?,
    )
    .map_err(|denial| snapshot_lease_denial(denial, operation.operation()))?;
    let mutating = operation.graph_obligations().rows().iter().any(|row| {
        matches!(
            row.effect_posture(),
            worth_query_installation::facade::WorthQueryInstalledGraphObligationEffectPosture::Mutating
                | worth_query_installation::facade::WorthQueryInstalledGraphObligationEffectPosture::Invariant
        )
    });
    let intent = if mutating {
        WorthQueryGraphWorkIntent::application_operation_mutation()
    } else {
        WorthQueryGraphWorkIntent::application_operation_read()
    };
    let obligations = operation.retain_graph_obligations_for_admission();
    let obligation_identity = obligations.identity().clone();
    let selected = select_installed_graph_obligations(obligations, intent)
        .map_err(|_| graph_work_denial(operation.operation()))?;
    let support = runtime.graph_work_resource_support();
    let plan = admit_application_operation_read_graph_work(selected, &support)
        .map_err(|_| graph_work_denial(operation.operation()))?;
    WorthQueryManagedGraphWorkSession::start_mutation(
        plan,
        runtime.runtime.authority_identity(),
        operation.binding_identity(),
        &obligation_identity,
        operation.authority_identity(),
        principal,
        access,
        lease,
        runtime.graph_work_provider_identity(),
    )
    .map_err(|_| graph_work_denial(operation.operation()))
}

pub(super) fn transition_capability_to_operation_graph_work<Schema, Operation, Input>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
    resource_binding_identity: &str,
    resource: EntityId,
    capability_graph_work: WorthQueryManagedGraphWorkSession,
) -> Result<WorthQueryManagedGraphWorkSession, WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
{
    let principal = capability_graph_work.principal();
    let installed_capability = capability_graph_work
        .capability_access_context()
        .ok_or_else(|| graph_work_denial(operation.operation()))?;
    drop(capability_graph_work);
    start_operation_graph_work(
        runtime,
        operation,
        resource_binding_identity,
        principal,
        WorthQueryGraphWorkAccessContextAffinity::governed_entity(resource, installed_capability),
    )
}

fn graph_work_denial(subject: impl Into<String>) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::GraphWorkAdmissionUnavailable,
        subject,
    )
}

fn snapshot_lease_denial(
    lease_denial: WorthQueryApplicationSnapshotLeaseDenial,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    let kind = match lease_denial {
        WorthQueryApplicationSnapshotLeaseDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => WorthQueryOperationAuthorizationDenialKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        WorthQueryApplicationSnapshotLeaseDenial::SnapshotIdentityExhausted => {
            WorthQueryOperationAuthorizationDenialKind::SnapshotIdentityExhausted
        }
        _ => WorthQueryOperationAuthorizationDenialKind::GraphWorkAdmissionUnavailable,
    };
    denial(kind, subject)
}

fn denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}
