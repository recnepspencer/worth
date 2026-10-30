//! One genuine observed report binds the independently admitted performed connection.
use super::super::connection::ProducerExplanation;
use super::*;
use worth_runtime_bridge::facade::BridgeIdentityEvidence;
pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) struct FundedConnectedReport
{
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) query:
        FundedInspectionReport,
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) explanation:
        ProducerExplanation,
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) identity:
        BridgeIdentityEvidence,
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) retained_bytes:
        u64,
}
pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn materialize_connected(
    execution: &ExecutionRequest<'_, '_>,
    query: FundedInspectionReport,
    explanation: ProducerExplanation,
    session: &WorthQueryManagedGraphWorkSession,
    memory: &mut Option<ExecutionMemoryReservation>,
) -> Result<FundedConnectedReport, CaptureStop<InspectionReportDenial>> {
    run_retained_stage(execution, memory, |context, mut memory| {
        let source = query
            .target
            .connection_source()
            .ok_or(MapKernelFailure::Domain(
                InspectionReportDenial::ForeignComparison,
            ))?;
        let identity = super::super::identity::producer_explanation_identity(
            &query.payload,
            source,
            session,
            &explanation,
            &mut |work, bytes| {
                checkpoint(
                    context,
                    u64::try_from(work).map_err(|_| {
                        MapKernelFailure::Stop(worth_execution::MapKernelStop::WorkCounterOverflow)
                    })?,
                )
                .map_err(MapKernelFailure::Stop)?;
                if let Some(memory) = memory.as_deref_mut() {
                    let total = memory
                        .bytes()
                        .checked_add(u64::try_from(bytes).map_err(|_| {
                            MapKernelFailure::Domain(RetainedStorageDenial::Overflow)
                        })?)
                        .ok_or(MapKernelFailure::Domain(RetainedStorageDenial::Overflow))?;
                    memory.resize(total).map_err(MapKernelFailure::Domain)?;
                }
                Ok(())
            },
        )
        .map_err(|cause| MapKernelFailure::Domain(InspectionReportDenial::Identity(cause)))?;
        let retained_bytes = query
            .retained_bytes
            .checked_add(memory.as_deref().map_or(0, |v| v.bytes()))
            .ok_or(MapKernelFailure::Domain(InspectionReportDenial::Storage(
                RetainedStorageDenial::Overflow,
            )))?;
        Ok(FundedConnectedReport {
            query,
            explanation,
            identity,
            retained_bytes,
        })
    })
}
