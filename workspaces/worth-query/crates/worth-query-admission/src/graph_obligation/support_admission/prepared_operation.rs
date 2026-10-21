//! Cold installed operation decisions carried into live capacity admission.

use std::sync::Arc;

use worth_query_installation::facade::{
    WorthQueryInstalledApplicationOperation, WorthQueryInstalledGraphObligationSet,
};

use super::*;
use crate::domain_computation::execution_resource_admission::{
    prepare_execution_resource_plan, reserve_execution_resource_plan, PreparedExecutionResourcePlan,
};
use crate::graph_obligation::{
    selected_set::SelectedInstalledGraphObligations, selection::select_core,
    WorthQueryGraphObligationSelectionCounters, WorthQueryGraphWorkIntent,
};

/// Immutable decisions for one installed operation and actual installed support.
/// This owns no live provider capacity and grants no operation permission.
pub struct WorthQueryPreparedApplicationOperationGraphWork {
    obligations: Arc<WorthQueryInstalledGraphObligationSet>,
    intent: WorthQueryGraphWorkIntent,
    resources: PreparedExecutionResourcePlan,
}

/// Installation lane: call only after the final provider support is installed.
pub fn prepare_application_operation_graph_work<Schema, Operation, Input>(
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
    request: &WorthQueryExecutionResourceRequest,
    support: &WorthQueryExecutionResourceSupportSnapshot,
) -> Result<WorthQueryPreparedApplicationOperationGraphWork, WorthQueryGraphWorkAdmissionDenial> {
    let obligations = Arc::new(operation.retain_graph_obligations_for_admission());
    let intent = WorthQueryGraphWorkIntent::application_operation(obligations.rows());
    let selected = select_core(
        SelectedInstalledGraphObligations::PreparedOperation(Arc::clone(&obligations)),
        intent,
    )
    .map_err(WorthQueryGraphWorkAdmissionDenial::Selection)?;
    validate_owner_requirements(&selected)?;
    let (resources, _) = prepare_execution_resource_plan(
        operation.contracts().resources(),
        request,
        support.clone(),
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .map_err(WorthQueryGraphWorkAdmissionDenial::ExecutionResource)?;
    Ok(WorthQueryPreparedApplicationOperationGraphWork {
        obligations,
        intent,
        resources,
    })
}

pub fn admit_prepared_application_operation_graph_work<Schema, Operation, Input>(
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
    prepared: &WorthQueryPreparedApplicationOperationGraphWork,
    invocation_binding_identity: &str,
    actual_support: &WorthQueryExecutionResourceSupportSnapshot,
) -> Result<WorthQueryAdmittedGraphWorkPlan, WorthQueryGraphWorkAdmissionDenial> {
    if operation.binding_identity() != prepared.obligations.binding_identity()
        || operation.graph_obligations().identity() != prepared.obligations.identity()
    {
        return Err(WorthQueryGraphWorkAdmissionDenial::OperationAuthorityMismatch);
    }
    if !prepared
        .resources
        .support()
        .has_same_installed_authority(actual_support)
    {
        return Err(WorthQueryGraphWorkAdmissionDenial::ProviderSupportUnavailable);
    }
    let selected = WorthQuerySelectedGraphObligations::seal(
        SelectedInstalledGraphObligations::PreparedOperation(Arc::clone(&prepared.obligations)),
        prepared.intent,
        WorthQueryGraphObligationSelectionCounters::retained_selection(
            prepared.obligations.rows().len(),
        ),
    );
    let resources = prepared.resources.bind(
        invocation_binding_identity,
        WorthQueryExecutionResourceAdmissionCounters {
            support_snapshot_checks: 1,
            ..Default::default()
        },
    );
    let capacity = reserve_execution_resource_plan(resources)
        .ok_or(WorthQueryGraphWorkAdmissionDenial::CapacityUnavailable)?;
    WorthQueryAdmittedGraphWorkPlan::seal(
        selected,
        WorthQueryGraphWorkAdmissionMechanics::ApplicationOperation {
            capacity: Some(capacity),
        },
    )
    .ok_or(WorthQueryGraphWorkAdmissionDenial::IdentityExhausted)
}
