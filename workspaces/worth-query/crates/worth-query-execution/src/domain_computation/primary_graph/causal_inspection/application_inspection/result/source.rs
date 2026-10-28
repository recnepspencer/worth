//! The source report owns immutable evidence, never the original receipt or read.
use super::super::source_original::SourceOriginal;
use super::*;

pub struct WorthQueryPerformedSourceInspection {
    original: SourceOriginal,
    report: InspectionReportPayload,
    _memory: Option<ExecutionMemoryReservation>,
}
impl WorthQueryPerformedSourceInspection {
    pub fn source_commit(&self) -> &worth_relational::facade::history::RelationalCommitReceipt {
        self.original.changes.commit_reference()
    }
    pub fn source_observation_identity(&self) -> &str {
        self.report.observation_identity()
    }
    pub fn report_identity(&self) -> &str {
        self.report.report_identity()
    }
    /// No producer, output change or suppression is inferred by this mode.
    pub const fn lower_evidence_posture(&self) -> CausalObservationMissingReferencePosture {
        CausalObservationMissingReferencePosture::MissingOptional
    }
    pub fn close(self) {
        drop(self);
    }
}
pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) struct PreparedSourceInspection
{
    original: SourceOriginal,
    report: InspectionReportPayload,
}
impl CompletedInspection {
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn publish_source(
        self,
        execution: &ExecutionRequest<'_, '_>,
        original: SourceOriginal,
        report: FundedInspectionReport,
        memory: &mut Option<ExecutionMemoryReservation>,
    ) -> Result<PreparedSourceInspection, InspectionAdmissionDenial> {
        self.validate_report(execution, &report)?;
        if report.reference() != super::super::identity::InspectionOriginalReference::Source
            || !report
                .target()
                .source()
                .is_some_and(|source| source.same_original(&original))
        {
            return Err(InspectionAdmissionDenial::ForeignIssuedAccess);
        }
        let (report, retained_bytes) = report.release_target();
        self.finish_publication(execution, retained_bytes, memory)?;
        Ok(PreparedSourceInspection { original, report })
    }
}
impl PreparedSourceInspection {
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn fund(
        self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> WorthQueryPerformedSourceInspection {
        WorthQueryPerformedSourceInspection {
            original: self.original,
            report: self.report,
            _memory: memory,
        }
    }
}
