//! Connected observation publishes only after the common consuming terminal.
use super::super::{
    connection::{
        ProducerExplanation, WorthQueryApplicationProducerConnection,
        WorthQueryApplicationProducerEvaluation,
    },
    report::FundedConnectedReport,
};
use super::*;
use worth_runtime_bridge::facade::BridgeIdentityEvidence;
/// One original query with its verified performed source/producer connection.
pub struct WorthQueryConnectedApplicationInspection<'original> {
    original: &'original WorthQueryApplicationQueryObservation,
    report: InspectionReportPayload,
    connection: WorthQueryApplicationProducerConnection,
    identity: BridgeIdentityEvidence,
    _memory: Option<ExecutionMemoryReservation>,
}
impl<'a> WorthQueryConnectedApplicationInspection<'a> {
    /// Immutable original custody admitted by this report's fresh metadata session.
    /// This grants no field access and makes no claim about evictable diagnostics.
    pub fn evidence_availability(
        &self,
    ) -> super::super::availability::WorthQueryInspectionEvidenceAvailability {
        self.report.availability()
    }
    pub fn observation(&self) -> &'a WorthQueryApplicationQueryObservation {
        self.original
    }
    pub fn report_identity(&self) -> &str {
        self.identity.terminal_projection_for_reporting()
    }
    pub fn observation_identity(&self) -> &str {
        self.report.observation_identity()
    }
    pub fn reason(&self) -> CausalInspectionReason {
        CausalInspectionReason::ObservedResult
    }
    pub fn producer_connection(&self) -> &WorthQueryApplicationProducerConnection {
        &self.connection
    }
    pub fn close(self) {
        drop(self)
    }
}
pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) struct PreparedConnectedInspection<
    'a,
> {
    original: &'a WorthQueryApplicationQueryObservation,
    report: InspectionReportPayload,
    explanation: ProducerExplanation,
    identity: BridgeIdentityEvidence,
}
impl CompletedInspection {
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn publish_connected<
        'a,
    >(
        self,
        execution: &ExecutionRequest<'_, '_>,
        original: &'a WorthQueryApplicationQueryObservation,
        report: FundedConnectedReport,
        memory: &mut Option<ExecutionMemoryReservation>,
    ) -> Result<PreparedConnectedInspection<'a>, InspectionAdmissionDenial> {
        self.validate_report(execution, &report.query)?;
        if !report.query.target().matches_original(original)
            || report.query.target().connection_source().is_none()
            || report.query.target().before().is_some()
        {
            return Err(InspectionAdmissionDenial::ForeignIssuedAccess);
        }
        let (query, _) = report.query.release_target();
        let result = PreparedConnectedInspection {
            original,
            report: query,
            explanation: report.explanation,
            identity: report.identity,
        };
        self.finish_publication(execution, report.retained_bytes, memory)?;
        Ok(result)
    }
}
impl<'a> PreparedConnectedInspection<'a> {
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn fund(
        self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> SourceInspectionResult<'a> {
        match self.explanation {
            ProducerExplanation::Creation(connection) => {
                SourceInspectionResult::Creation(WorthQueryConnectedApplicationInspection {
                    original: self.original,
                    report: self.report,
                    connection,
                    identity: self.identity,
                    _memory: memory,
                })
            }
            ProducerExplanation::Evaluation(evaluation) => {
                SourceInspectionResult::Evaluation(WorthQueryEvaluatedApplicationInspection {
                    original: self.original,
                    report: self.report,
                    evaluation,
                    identity: self.identity,
                    _memory: memory,
                })
            }
        }
    }
}

/// One original query with an independently admitted, actual output-field evaluation.
pub struct WorthQueryEvaluatedApplicationInspection<'original> {
    original: &'original WorthQueryApplicationQueryObservation,
    report: InspectionReportPayload,
    evaluation: WorthQueryApplicationProducerEvaluation,
    identity: BridgeIdentityEvidence,
    _memory: Option<ExecutionMemoryReservation>,
}
impl<'a> WorthQueryEvaluatedApplicationInspection<'a> {
    /// Immutable original custody admitted by this report's fresh metadata session.
    /// This grants no field access and makes no claim about evictable diagnostics.
    pub fn evidence_availability(
        &self,
    ) -> super::super::availability::WorthQueryInspectionEvidenceAvailability {
        self.report.availability()
    }
    pub fn observation(&self) -> &'a WorthQueryApplicationQueryObservation {
        self.original
    }
    pub fn report_identity(&self) -> &str {
        self.identity.terminal_projection_for_reporting()
    }
    pub fn observation_identity(&self) -> &str {
        self.report.observation_identity()
    }
    pub fn reason(&self) -> CausalInspectionReason {
        CausalInspectionReason::ObservedResult
    }
    pub fn producer_evaluation(&self) -> &WorthQueryApplicationProducerEvaluation {
        &self.evaluation
    }
    pub fn close(self) {
        drop(self)
    }
}
pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) enum SourceInspectionResult<
    'a,
> {
    Creation(WorthQueryConnectedApplicationInspection<'a>),
    Rich(super::WorthQueryMaterializedApplicationInspection<'a>),
    Evaluation(WorthQueryEvaluatedApplicationInspection<'a>),
}
impl<'a> SourceInspectionResult<'a> {
    pub(in super::super) fn rich(self) -> super::WorthQueryMaterializedApplicationInspection<'a> {
        let Self::Rich(result) = self else {
            unreachable!("private rich request produces rich result")
        };
        result
    }

    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn creation(
        self,
    ) -> WorthQueryConnectedApplicationInspection<'a> {
        let Self::Creation(result) = self else {
            unreachable!("private creation request produces its creation result")
        };
        result
    }
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn evaluation(
        self,
    ) -> WorthQueryEvaluatedApplicationInspection<'a> {
        let Self::Evaluation(result) = self else {
            unreachable!("private evaluation request produces its evaluation result")
        };
        result
    }
}
