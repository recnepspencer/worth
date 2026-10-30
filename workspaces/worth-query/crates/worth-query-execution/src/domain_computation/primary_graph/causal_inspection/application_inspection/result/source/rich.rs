//! Owned source-only detail can outlive the original receipt without gaining authority.
use super::super::super::{
    admission::rich::PreparedRichMaterialization,
    report::rich::FundedRichInspectionReport,
    rich::{
        OriginalQueryDetail, WorthQueryApplicationRichMaterialization,
        WorthQueryMaterializedEvidenceReference,
    },
};
use super::*;
use worth_runtime_bridge::facade::BridgeIdentityEvidence;

pub struct WorthQueryMaterializedPerformedSourceInspection {
    original: SourceOriginal,
    report: InspectionReportPayload,
    materialization: WorthQueryApplicationRichMaterialization,
    identity: BridgeIdentityEvidence,
    _memory: Option<ExecutionMemoryReservation>,
}
impl WorthQueryMaterializedPerformedSourceInspection {
    pub fn source_commit(&self) -> &worth_relational::facade::history::RelationalCommitReceipt {
        self.original.changes.commit_reference()
    }
    pub fn source_observation_identity(&self) -> &str {
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
    pub const fn lower_evidence_posture(&self) -> CausalObservationMissingReferencePosture {
        CausalObservationMissingReferencePosture::MissingOptional
    }
    pub fn close(self) {
        drop(self)
    }
}
pub(in super::super::super) struct PreparedMaterializedSourceInspection {
    original: SourceOriginal,
    report: InspectionReportPayload,
    materialization: WorthQueryApplicationRichMaterialization,
    identity: BridgeIdentityEvidence,
}
impl CompletedInspection {
    pub(in super::super::super) fn publish_source_rich(
        self,
        execution: &ExecutionRequest<'_, '_>,
        original: SourceOriginal,
        report: FundedRichInspectionReport,
        memory: &mut Option<ExecutionMemoryReservation>,
    ) -> Result<PreparedMaterializedSourceInspection, InspectionAdmissionDenial> {
        self.validate_report(execution, &report.reference)?;
        if report.reference.reference()
            != super::super::super::identity::InspectionOriginalReference::Source
            || !report.body.matches(&report.reference)
            || !report
                .reference
                .target()
                .source()
                .is_some_and(|source| source.same_original(&original))
            || report.reference.target().before().is_some()
            || report.reference.target().connection_source().is_some()
            || report.connection_identity.is_some()
            || report.body.artifact.producer.is_some()
            || !matches!(
                report.body.artifact.original,
                OriginalQueryDetail::NotRequested | OriginalQueryDetail::Unselected
            )
        {
            return Err(InspectionAdmissionDenial::ForeignIssuedAccess);
        }
        let (payload, _) = report.reference.release_target();
        let PreparedRichMaterialization { scope, artifact } = report.body;
        drop(scope);
        let result = PreparedMaterializedSourceInspection {
            original,
            report: payload,
            materialization: artifact,
            identity: report.identity,
        };
        self.finish_publication(execution, report.retained_bytes, memory)?;
        Ok(result)
    }
}
impl PreparedMaterializedSourceInspection {
    pub(in super::super::super) fn fund(
        self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> WorthQueryMaterializedPerformedSourceInspection {
        WorthQueryMaterializedPerformedSourceInspection {
            original: self.original,
            report: self.report,
            materialization: self.materialization,
            identity: self.identity,
            _memory: memory,
        }
    }
}
