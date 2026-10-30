//! The installed owner admits its real lookup and owned reissue before copying.
use super::*;
use crate::domain_computation::primary_graph::admitted_capture::{
    checkpoint, run_retained_stage, CaptureStop, RetainedStorageDenial,
};
use worth_execution::MapKernelFailure;
use worth_query_installation::facade::{
    WorthQueryApplicationQueryInstallationDenial, WorthQueryInstalledApplicationQueryBinding,
    WorthQueryInstalledQueryBindingAdmissionStop,
};

pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) enum InspectionBindingDenial
{
    Installation(WorthQueryApplicationQueryInstallationDenial),
    Storage(RetainedStorageDenial),
}

pub(super) fn installed<Schema: ApplicationSchema, Binding: ApplicationQueryBinding<Schema>>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
    memory: &mut Option<ExecutionMemoryReservation>,
) -> Result<
    WorthQueryInstalledApplicationQueryBinding<Schema, Binding>,
    CaptureStop<InspectionBindingDenial>,
> {
    run_retained_stage(execution, memory, |context, mut memory| {
        let mut admit = |work, bytes| {
            checkpoint(context, work).map_err(MapKernelFailure::Stop)?;
            if let Some(memory) = memory.as_deref_mut() {
                let total = memory
                    .bytes()
                    .checked_add(bytes)
                    .ok_or(MapKernelFailure::Domain(RetainedStorageDenial::Overflow))?;
                memory.resize(total).map_err(MapKernelFailure::Domain)?;
            }
            Ok(())
        };
        runtime
            .installed_schema()
            .installed_query_binding_admitted::<Binding, _>(&mut admit)
            .map_err(|cause| match cause {
                WorthQueryInstalledQueryBindingAdmissionStop::Installation(cause) => {
                    MapKernelFailure::Domain(InspectionBindingDenial::Installation(cause))
                }
                WorthQueryInstalledQueryBindingAdmissionStop::AccountingOverflow => {
                    MapKernelFailure::Domain(InspectionBindingDenial::Storage(
                        RetainedStorageDenial::Overflow,
                    ))
                }
                WorthQueryInstalledQueryBindingAdmissionStop::Admission(
                    MapKernelFailure::Stop(cause),
                ) => MapKernelFailure::Stop(cause),
                WorthQueryInstalledQueryBindingAdmissionStop::Admission(
                    MapKernelFailure::Domain(cause),
                ) => MapKernelFailure::Domain(InspectionBindingDenial::Storage(cause)),
                WorthQueryInstalledQueryBindingAdmissionStop::Admission(
                    MapKernelFailure::Panic,
                ) => MapKernelFailure::Panic,
                WorthQueryInstalledQueryBindingAdmissionStop::Admission(
                    MapKernelFailure::ResultCapacityExceeded,
                ) => MapKernelFailure::ResultCapacityExceeded,
            })
    })
}
