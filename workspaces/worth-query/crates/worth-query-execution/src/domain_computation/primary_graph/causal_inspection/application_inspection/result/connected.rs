//! Connected observation publishes only after the common consuming terminal.
use super::super::{
    connection::WorthQueryApplicationProducerConnection, report::FundedConnectedReport,
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
    connection: WorthQueryApplicationProducerConnection,
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
            connection: report.connection,
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
    ) -> WorthQueryConnectedApplicationInspection<'a> {
        WorthQueryConnectedApplicationInspection {
            original: self.original,
            report: self.report,
            connection: self.connection,
            identity: self.identity,
            _memory: memory,
        }
    }
}
