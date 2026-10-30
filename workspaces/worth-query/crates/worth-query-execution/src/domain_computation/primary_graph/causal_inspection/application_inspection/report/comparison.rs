//! Query binds two genuine single-original reports to the derived comparison.
use super::super::super::CausalInspectionReason;
use super::super::identity::comparison_identity;
use super::*;
use worth_runtime_bridge::facade::BridgeIdentityEvidence;

pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) struct ComparisonReportPayload
{
    before: InspectionReportPayload,
    after: InspectionReportPayload,
    identity: BridgeIdentityEvidence,
    reason: CausalInspectionReason,
    connection: Option<super::super::connection::WorthQueryApplicationProducerConnection>,
}

pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) struct FundedComparisonReport
{
    before: FundedInspectionReport,
    after: FundedInspectionReport,
    identity: BridgeIdentityEvidence,
    reason: CausalInspectionReason,
    connection: Option<super::super::connection::WorthQueryApplicationProducerConnection>,
    retained_bytes: u64,
}

pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn materialize_comparison(
    execution: &ExecutionRequest<'_, '_>,
    before: FundedInspectionReport,
    after: FundedInspectionReport,
    reason: CausalInspectionReason,
    connection: Option<super::super::connection::WorthQueryApplicationProducerConnection>,
    memory: &mut Option<ExecutionMemoryReservation>,
) -> Result<FundedComparisonReport, CaptureStop<InspectionReportDenial>> {
    run_retained_stage(execution, memory, |context, mut memory| {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        if before.reference != InspectionOriginalReference::Before
            || after.reference != InspectionOriginalReference::After
            || !Arc::ptr_eq(&before.target, &after.target)
            || before.session != after.session
            || before.plan != after.plan
            || !matches!(
                reason,
                CausalInspectionReason::ChangedResult | CausalInspectionReason::UnchangedResult
            )
        {
            return Err(MapKernelFailure::Domain(
                InspectionReportDenial::ForeignComparison,
            ));
        }
        let mut admit = |work: usize, bytes: usize| {
            let work = u64::try_from(work).map_err(|_| {
                MapKernelFailure::Stop(worth_execution::MapKernelStop::WorkCounterOverflow)
            })?;
            checkpoint(context, work).map_err(MapKernelFailure::Stop)?;
            let bytes = u64::try_from(bytes)
                .map_err(|_| MapKernelFailure::Domain(RetainedStorageDenial::Overflow))?;
            if let Some(memory) = memory.as_deref_mut() {
                let total = memory
                    .bytes()
                    .checked_add(bytes)
                    .ok_or(MapKernelFailure::Domain(RetainedStorageDenial::Overflow))?;
                memory.resize(total).map_err(MapKernelFailure::Domain)?;
            }
            Ok(())
        };
        let identity = comparison_identity(
            &before.payload,
            &after.payload,
            reason,
            connection.as_ref(),
            &mut admit,
        )
        .map_err(|cause| MapKernelFailure::Domain(InspectionReportDenial::Identity(cause)))?;
        drop(admit);
        let retained_bytes = before
            .retained_bytes
            .checked_add(after.retained_bytes)
            .and_then(|bytes| {
                bytes.checked_add(memory.as_deref().map_or(0, |memory| memory.bytes()))
            })
            .ok_or(MapKernelFailure::Domain(InspectionReportDenial::Storage(
                RetainedStorageDenial::Overflow,
            )))?;
        Ok(FundedComparisonReport {
            before,
            after,
            identity,
            reason,
            connection,
            retained_bytes,
        })
    })
}

impl FundedComparisonReport {
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn before(
        &self,
    ) -> &FundedInspectionReport {
        &self.before
    }
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn after(
        &self,
    ) -> &FundedInspectionReport {
        &self.after
    }
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn release_targets(
        self,
    ) -> (ComparisonReportPayload, u64) {
        let (before, _) = self.before.release_target();
        let (after, _) = self.after.release_target();
        (
            ComparisonReportPayload {
                before,
                after,
                identity: self.identity,
                reason: self.reason,
                connection: self.connection,
            },
            self.retained_bytes,
        )
    }
}

impl ComparisonReportPayload {
    pub(in super::super) fn before_availability(
        &self,
    ) -> super::super::availability::WorthQueryInspectionEvidenceAvailability {
        self.before.availability()
    }
    pub(in super::super) fn after_availability(
        &self,
    ) -> super::super::availability::WorthQueryInspectionEvidenceAvailability {
        self.after.availability()
    }

    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn connection(
        &self,
    ) -> Option<&super::super::connection::WorthQueryApplicationProducerConnection> {
        self.connection.as_ref()
    }

    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn before_identity(
        &self,
    ) -> &str {
        self.before.observation_identity()
    }
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn after_identity(
        &self,
    ) -> &str {
        self.after.observation_identity()
    }
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn before_report_identity(
        &self,
    ) -> &str {
        self.before.report_identity()
    }
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn after_report_identity(
        &self,
    ) -> &str {
        self.after.report_identity()
    }
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn report_identity(
        &self,
    ) -> &str {
        self.identity.terminal_projection_for_reporting()
    }
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn reason(
        &self,
    ) -> CausalInspectionReason {
        self.reason
    }
}
