//! Rich data becomes public only after the matching consuming inspection terminal.
use super::super::{
    report::rich::FundedRichInspectionReport,
    rich::{WorthQueryApplicationRichMaterialization, WorthQueryMaterializedEvidenceReference},
};
use super::*;
use worth_runtime_bridge::facade::BridgeIdentityEvidence;
pub struct WorthQueryMaterializedApplicationInspection<'o> {
    original: &'o WorthQueryApplicationQueryObservation,
    report: InspectionReportPayload,
    materialization: WorthQueryApplicationRichMaterialization,
    identity: BridgeIdentityEvidence,
    _memory: Option<ExecutionMemoryReservation>,
}
impl<'o> WorthQueryMaterializedApplicationInspection<'o> {
    pub fn observation(&self) -> &'o WorthQueryApplicationQueryObservation {
        self.original
    }
    pub fn observation_identity(&self) -> &str {
        self.report.observation_identity()
    }
    pub fn report_identity(&self) -> &str {
        self.identity.terminal_projection_for_reporting()
    }
    pub fn materialization(&self) -> &WorthQueryApplicationRichMaterialization {
        &self.materialization
    }
    pub fn evidence_references(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthQueryMaterializedEvidenceReference<'_>> {
        self.report
            .envelope()
            .bindings()
            .iter()
            .map(|binding| WorthQueryMaterializedEvidenceReference { binding })
    }
    pub fn close(self) {
        drop(self)
    }
}
pub(in super::super) struct PreparedRichInspection<'o> {
    original: &'o WorthQueryApplicationQueryObservation,
    report: InspectionReportPayload,
    materialization: WorthQueryApplicationRichMaterialization,
    identity: BridgeIdentityEvidence,
}
impl CompletedInspection {
    pub(in super::super) fn publish_rich<'o>(
        self,
        execution: &ExecutionRequest<'_, '_>,
        original: &'o WorthQueryApplicationQueryObservation,
        report: FundedRichInspectionReport,
        memory: &mut Option<ExecutionMemoryReservation>,
    ) -> Result<PreparedRichInspection<'o>, InspectionAdmissionDenial> {
        self.validate_report(execution, &report.reference)?;
        if !report.body.matches(&report.reference)
            || !report.reference.target().matches_original(original)
            || report.reference.target().before().is_some()
            || report.reference.target().connection_source().is_some()
        {
            return Err(InspectionAdmissionDenial::ForeignIssuedAccess);
        }
        let (payload, _) = report.reference.release_target();
        let super::super::admission::rich::PreparedRichMaterialization { scope, artifact } =
            report.body;
        drop(scope);
        let result = PreparedRichInspection {
            original,
            report: payload,
            materialization: artifact,
            identity: report.identity,
        };
        self.finish_publication(execution, report.retained_bytes, memory)?;
        Ok(result)
    }
}
impl<'o> PreparedRichInspection<'o> {
    pub(in super::super) fn fund(
        self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> WorthQueryMaterializedApplicationInspection<'o> {
        WorthQueryMaterializedApplicationInspection {
            original: self.original,
            report: self.report,
            materialization: self.materialization,
            identity: self.identity,
            _memory: memory,
        }
    }
}
