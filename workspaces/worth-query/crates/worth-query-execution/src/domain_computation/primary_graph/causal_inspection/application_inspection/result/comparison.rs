//! The same completed Inspection publishes both original reports and their relation.
use super::super::report::{ComparisonReportPayload, FundedComparisonReport};
use super::*;

/// A complete original-output comparison after fresh metadata and diagnostic
/// admission. An explicitly requested producer connection carries only the
/// exact verified source/output relation; Native cells and readers never escape.
pub struct WorthQueryComparedApplicationInspection<'original> {
    before: &'original WorthQueryApplicationQueryObservation,
    after: &'original WorthQueryApplicationQueryObservation,
    report: ComparisonReportPayload,
    _memory: Option<ExecutionMemoryReservation>,
}

impl<'original> WorthQueryComparedApplicationInspection<'original> {
    pub fn before(&self) -> &'original WorthQueryApplicationQueryObservation {
        self.before
    }
    pub fn after(&self) -> &'original WorthQueryApplicationQueryObservation {
        self.after
    }
    pub fn before_observation_identity(&self) -> &str {
        self.report.before_identity()
    }
    pub fn after_observation_identity(&self) -> &str {
        self.report.after_identity()
    }
    pub fn before_report_identity(&self) -> &str {
        self.report.before_report_identity()
    }
    pub fn after_report_identity(&self) -> &str {
        self.report.after_report_identity()
    }
    /// Binds both genuine original report identities and the derived reason.
    pub fn report_identity(&self) -> &str {
        self.report.report_identity()
    }
    /// ChangedResult or UnchangedResult; a withheld original refuses this mode.
    pub fn reason(&self) -> CausalInspectionReason {
        self.report.reason()
    }
    /// Present only when this fresh inspection verified both exact joins.
    pub fn producer_connection(
        &self,
    ) -> Option<&super::super::connection::WorthQueryApplicationProducerConnection> {
        self.report.connection()
    }
    pub const fn lower_evidence_posture(&self) -> CausalObservationMissingReferencePosture {
        CausalObservationMissingReferencePosture::MissingOptional
    }
    pub fn close(self) {
        drop(self);
    }
}

pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) struct PreparedComparedInspection<
    'original,
> {
    before: &'original WorthQueryApplicationQueryObservation,
    after: &'original WorthQueryApplicationQueryObservation,
    report: ComparisonReportPayload,
}

impl CompletedInspection {
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn publish_comparison<
        'original,
    >(
        self,
        execution: &ExecutionRequest<'_, '_>,
        before: &'original WorthQueryApplicationQueryObservation,
        after: &'original WorthQueryApplicationQueryObservation,
        report: FundedComparisonReport,
        memory: &mut Option<ExecutionMemoryReservation>,
    ) -> Result<PreparedComparedInspection<'original>, InspectionAdmissionDenial> {
        self.validate_report(execution, report.before())?;
        self.validate_report(execution, report.after())?;
        if !report.before().target().matches_before(before)
            || !report.after().target().matches_original(after)
        {
            return Err(InspectionAdmissionDenial::ForeignIssuedAccess);
        }
        let (report, retained_bytes) = report.release_targets();
        let result = PreparedComparedInspection {
            before,
            after,
            report,
        };
        self.finish_publication(execution, retained_bytes, memory)?;
        Ok(result)
    }
}

impl<'original> PreparedComparedInspection<'original> {
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn fund(
        self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> WorthQueryComparedApplicationInspection<'original> {
        WorthQueryComparedApplicationInspection {
            before: self.before,
            after: self.after,
            report: self.report,
            _memory: memory,
        }
    }
}
