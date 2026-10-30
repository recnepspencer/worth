//! Inspection reserves its actual provider capacity on the original request ticket.
use super::*;
use crate::domain_computation::primary_graph::admitted_capture::{
    checkpoint, run_retained_stage, CaptureStop, RetainedStorageDenial,
};
use worth_execution::MapKernelFailure;
use worth_query_admission::facade::graph_obligation::{
    admit_application_inspection_graph_work, WorthQueryAdmittedGraphWorkPlan,
    WorthQueryGraphWorkAdmissionDenial, WorthQueryGraphWorkCapacityAdmissionStop,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationInspectionContract,
};

pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) enum InspectionPlanDenial
{
    Graph(WorthQueryGraphWorkAdmissionDenial),
    Storage(RetainedStorageDenial),
    Overflow,
}

pub(super) fn admit<Schema: ApplicationSchema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
    contract: &WorthQueryInstalledApplicationInspectionContract,
    memory: &mut Option<ExecutionMemoryReservation>,
) -> Result<WorthQueryAdmittedGraphWorkPlan, CaptureStop<InspectionPlanDenial>> {
    run_retained_stage(execution, memory, |context, mut memory| {
        let mut admit = |work, bytes| {
            checkpoint(context, work).map_err(MapKernelFailure::Stop)?;
            if let Some(memory) = memory.as_deref_mut() {
                let bytes = memory
                    .bytes()
                    .checked_add(bytes)
                    .ok_or(MapKernelFailure::Domain(InspectionPlanDenial::Overflow))?;
                memory.resize(bytes).map_err(|cause| {
                    MapKernelFailure::Domain(InspectionPlanDenial::Storage(cause))
                })?;
            }
            Ok(())
        };
        admit_application_inspection_graph_work(
            contract,
            runtime.primary_provider.application_resource_support_ref(),
            &mut admit,
        )
        .map_err(|cause| match cause {
            WorthQueryGraphWorkCapacityAdmissionStop::Admission(cause) => cause,
            WorthQueryGraphWorkCapacityAdmissionStop::Denial(cause) => {
                MapKernelFailure::Domain(InspectionPlanDenial::Graph(cause))
            }
            WorthQueryGraphWorkCapacityAdmissionStop::AccountingOverflow => {
                MapKernelFailure::Domain(InspectionPlanDenial::Overflow)
            }
        })
    })
}
