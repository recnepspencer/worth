//! Requested rich body is associated with the very same funded report scope.
use super::super::admission::rich::PreparedRichMaterialization;
use super::*;
use worth_runtime_bridge::facade::BridgeIdentityEvidence;
pub(in super::super) struct FundedRichInspectionReport {
    pub(in super::super) reference: FundedInspectionReport,
    pub(in super::super) body: PreparedRichMaterialization,
    pub(in super::super) identity: BridgeIdentityEvidence,
    pub(in super::super) retained_bytes: u64,
}
pub(in super::super) fn materialize_rich(
    execution: &ExecutionRequest<'_, '_>,
    reference: FundedInspectionReport,
    body: PreparedRichMaterialization,
    memory: &mut Option<ExecutionMemoryReservation>,
) -> Result<FundedRichInspectionReport, CaptureStop<InspectionReportDenial>> {
    if !body.matches(&reference) {
        return Err(CaptureStop::Kernel(worth_execution::ScopeStop::Failure(
            MapKernelFailure::Domain(InspectionReportDenial::ForeignMaterialization),
        )));
    }
    run_retained_stage(execution, memory, |context, mut memory| {
        let mut admit = |work: usize, bytes: usize| {
            checkpoint(
                context,
                u64::try_from(work).map_err(|_| {
                    MapKernelFailure::Stop(worth_execution::MapKernelStop::WorkCounterOverflow)
                })?,
            )
            .map_err(MapKernelFailure::Stop)?;
            if let Some(memory) = memory.as_deref_mut() {
                let bytes = u64::try_from(bytes)
                    .map_err(|_| MapKernelFailure::Domain(RetainedStorageDenial::Overflow))?;
                let total = memory
                    .bytes()
                    .checked_add(bytes)
                    .ok_or(MapKernelFailure::Domain(RetainedStorageDenial::Overflow))?;
                memory.resize(total).map_err(MapKernelFailure::Domain)?;
            }
            Ok(())
        };
        let identity = super::super::identity::rich::rich_identity(
            &reference.payload,
            &body.artifact,
            &mut admit,
        )
        .map_err(|cause| MapKernelFailure::Domain(InspectionReportDenial::Identity(cause)))?;
        drop(admit);
        let retained_bytes = reference
            .retained_bytes
            .checked_add(memory.as_deref().map_or(0, |m| m.bytes()))
            .ok_or(MapKernelFailure::Domain(InspectionReportDenial::Storage(
                RetainedStorageDenial::Overflow,
            )))?;
        Ok(FundedRichInspectionReport {
            reference,
            body,
            identity,
            retained_bytes,
        })
    })
}
impl PreparedRichMaterialization {
    pub(in super::super) fn matches(&self, reference: &FundedInspectionReport) -> bool {
        self.scope.session == reference.session()
            && self.scope.plan == reference.plan()
            && Arc::ptr_eq(&self.scope.target, reference.target())
            && reference.target().rich_request() == Some(self.scope.request)
            && self.artifact.request() == self.scope.request
    }
}
