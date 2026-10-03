//! Cold installed operation decisions carried into live capacity admission.

use std::{convert::Infallible, mem::size_of, sync::Arc};

use worth_query_installation::facade::{
    WorthQueryInstalledApplicationOperation, WorthQueryInstalledGraphObligationSet,
};

use super::*;
use crate::domain_computation::execution_resource_admission::{
    prepare_execution_resource_plan, reserve_execution_resource_plan_admitted,
    PreparedExecutionResourcePlan, PreparedResourcePlanAdmissionStop,
};
use crate::graph_obligation::{
    selected_set::SelectedInstalledGraphObligations, selection::select_core,
    WorthQueryGraphObligationSelectionAdmissionStop, WorthQueryGraphObligationSelectionCounters,
    WorthQueryGraphWorkIntent,
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
    let selected = match select_core(
        SelectedInstalledGraphObligations::PreparedOperation(Arc::clone(&obligations)),
        intent,
        |_, _| Ok::<(), Infallible>(()),
    ) {
        Ok(selected) => selected,
        Err(WorthQueryGraphObligationSelectionAdmissionStop::Selection(denial)) => {
            return Err(WorthQueryGraphWorkAdmissionDenial::Selection(denial));
        }
        Err(WorthQueryGraphObligationSelectionAdmissionStop::Admission(never)) => match never {},
        Err(WorthQueryGraphObligationSelectionAdmissionStop::AccountingOverflow) => {
            return Err(WorthQueryGraphWorkAdmissionDenial::CapacityUnavailable);
        }
    };
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

pub fn admit_prepared_application_operation_graph_work_admitted<Schema, Operation, Input, Stop>(
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
    prepared: &WorthQueryPreparedApplicationOperationGraphWork,
    invocation_binding_identity: &str,
    actual_support: &WorthQueryExecutionResourceSupportSnapshot,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<WorthQueryAdmittedGraphWorkPlan, WorthQueryGraphWorkCapacityAdmissionStop<Stop>> {
    use WorthQueryGraphWorkCapacityAdmissionStop as Refusal;
    // The immutable schema binding contains 80 bytes; obligation digest 32.
    // Each exact comparison reads both operands.
    admit(2 * (80 + 32), 0).map_err(Refusal::Admission)?;
    if operation.binding_identity() != prepared.obligations.binding_identity()
        || operation.graph_obligations().identity() != prepared.obligations.identity()
    {
        return Err(Refusal::Denial(
            WorthQueryGraphWorkAdmissionDenial::OperationAuthorityMismatch,
        ));
    }
    admit(3, 0).map_err(Refusal::Admission)?;
    if !prepared
        .resources
        .support()
        .has_same_installed_authority(actual_support)
    {
        return Err(Refusal::Denial(
            WorthQueryGraphWorkAdmissionDenial::ProviderSupportUnavailable,
        ));
    }
    admit(1, 0).map_err(Refusal::Admission)?;
    let selected_rows = prepared.obligations.rows().len();
    let selected_work = u64::try_from(size_of::<WorthQuerySelectedGraphObligations>())
        .map_err(|_| Refusal::AccountingOverflow)?
        .checked_add(1)
        .ok_or(Refusal::AccountingOverflow)?;
    // One invocation observer retains the cold-validated set. No row walk or
    // repeated selection belongs to this invocation's counters.
    admit(selected_work, 0).map_err(Refusal::Admission)?;
    let selected = WorthQuerySelectedGraphObligations::seal(
        SelectedInstalledGraphObligations::PreparedOperation(Arc::clone(&prepared.obligations)),
        prepared.intent,
        WorthQueryGraphObligationSelectionCounters::retained_selection(selected_rows),
    );
    let counters = WorthQueryExecutionResourceAdmissionCounters {
        support_snapshot_checks: 1,
        ..Default::default()
    };
    let resources = prepared
        .resources
        .bind_admitted(invocation_binding_identity, counters, admit)
        .map_err(|stop| match stop {
            PreparedResourcePlanAdmissionStop::Admission(stop) => Refusal::Admission(stop),
            PreparedResourcePlanAdmissionStop::AccountingOverflow => Refusal::AccountingOverflow,
        })?;
    let capacity = reserve_execution_resource_plan_admitted(resources, admit)
        .map_err(|stop| match stop {
            WorthQueryCapacityReservationAdmissionStop::Admission(stop) => Refusal::Admission(stop),
            WorthQueryCapacityReservationAdmissionStop::AccountingOverflow => {
                Refusal::AccountingOverflow
            }
        })?
        .ok_or(Refusal::Denial(
            WorthQueryGraphWorkAdmissionDenial::CapacityUnavailable,
        ))?;
    admit(2, 0).map_err(Refusal::Admission)?;
    WorthQueryAdmittedGraphWorkPlan::seal(
        selected,
        WorthQueryGraphWorkAdmissionMechanics::ApplicationOperation {
            capacity: Some(capacity),
        },
    )
    .ok_or(Refusal::Denial(
        WorthQueryGraphWorkAdmissionDenial::IdentityExhausted,
    ))
}
