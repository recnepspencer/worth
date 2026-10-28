//! Descriptive results borrow originals and release the same actual session.
mod comparison;
mod source;
use super::super::{
    CausalInspectionReason, CausalObservationMissingReferencePosture,
    WorthQueryApplicationQueryObservation,
};
use super::admission::{require_request, InspectionAdmissionDenial};
use super::report::{FundedInspectionReport, InspectionReportPayload};
use crate::domain_computation::{
    primary_graph::admitted_capture::{CaptureStop, RetainedStorageDenial},
    provider_session::WorthQueryInspectionReadCompletion,
};
pub(super) use comparison::PreparedComparedInspection;
pub use comparison::WorthQueryComparedApplicationInspection;
pub use source::WorthQueryPerformedSourceInspection;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelFailure, ScopeStop};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

/// One original Query reference published only after fresh metadata admission,
/// currentness and the real Inspection session's consuming terminal.
pub struct WorthQueryObservedApplicationInspection<'original> {
    original: &'original WorthQueryApplicationQueryObservation,
    report: InspectionReportPayload,
    _memory: Option<ExecutionMemoryReservation>,
}

impl<'original> WorthQueryObservedApplicationInspection<'original> {
    /// The actual original immutable observation; it supplies no read authority.
    pub fn observation(&self) -> &'original WorthQueryApplicationQueryObservation {
        self.original
    }
    /// Identity derived from the actual original binding, source and basis.
    pub fn observation_identity(&self) -> &str {
        self.report.observation_identity()
    }
    /// Descriptive report identity associated with this completed fresh request.
    pub fn report_identity(&self) -> &str {
        self.report.report_identity()
    }
    /// A single completed original query establishes Observed, without Changed.
    pub const fn reason(&self) -> CausalInspectionReason {
        CausalInspectionReason::ObservedResult
    }

    /// This mode establishes the Query reference only. Optional lower causal
    /// connections are unestablished; this makes no claim that a producer is absent.
    pub const fn lower_evidence_posture(&self) -> CausalObservationMissingReferencePosture {
        CausalObservationMissingReferencePosture::MissingOptional
    }

    /// Release the report and its original-request backing without a graph read.
    pub fn close(self) {
        drop(self);
    }
}

/// The consuming terminal is retained until its matching report is checked.
pub(super) struct CompletedInspection {
    completion: WorthQueryInspectionReadCompletion,
    request: WorthQueryRequestScope,
}

/// Publication remains private until the caller transfers its original ticket.
pub(super) struct PreparedObservedInspection<'original> {
    original: &'original WorthQueryApplicationQueryObservation,
    report: InspectionReportPayload,
}

impl CompletedInspection {
    pub(super) fn from_terminal(
        completion: WorthQueryInspectionReadCompletion,
        request: WorthQueryRequestScope,
    ) -> Self {
        Self {
            completion,
            request,
        }
    }

    pub(super) fn publish<'original>(
        self,
        execution: &ExecutionRequest<'_, '_>,
        original: &'original WorthQueryApplicationQueryObservation,
        report: FundedInspectionReport,
        memory: &mut Option<ExecutionMemoryReservation>,
    ) -> Result<PreparedObservedInspection<'original>, InspectionAdmissionDenial> {
        self.validate_report(execution, &report)?;
        if !report.target().matches_original(original)
            || report.target().before().is_some()
            || report.reference() != super::identity::InspectionOriginalReference::After
        {
            return Err(InspectionAdmissionDenial::ForeignIssuedAccess);
        }
        let (payload, retained_bytes) = report.release_target();
        let result = PreparedObservedInspection {
            original,
            report: payload,
        };
        self.finish_publication(execution, retained_bytes, memory)?;
        Ok(result)
    }

    fn validate_report(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        report: &FundedInspectionReport,
    ) -> Result<(), InspectionAdmissionDenial> {
        require_request(execution, &self.request)?;
        self.completion
            .matches_report(report.session(), report.plan(), report.target())
            .then_some(())
            .ok_or(InspectionAdmissionDenial::ForeignIssuedAccess)
    }

    fn finish_publication(
        self,
        execution: &ExecutionRequest<'_, '_>,
        retained_bytes: u64,
        memory: &mut Option<ExecutionMemoryReservation>,
    ) -> Result<(), InspectionAdmissionDenial> {
        drop(self.completion);
        // Every transient target/session/auth/governance owner has ended. The
        // caller retains the original ticket through refusal and publication.
        if let Some(memory) = memory.as_mut() {
            if memory.bytes() < retained_bytes {
                return Err(InspectionAdmissionDenial::ForeignIssuedAccess);
            }
            memory.resize(retained_bytes).map_err(|cause| {
                InspectionAdmissionDenial::Storage(CaptureStop::Kernel(ScopeStop::Failure(
                    MapKernelFailure::Domain(RetainedStorageDenial::Memory(cause)),
                )))
            })?;
        }
        require_request(execution, &self.request)
    }
}

impl<'original> PreparedObservedInspection<'original> {
    pub(super) fn fund(
        self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> WorthQueryObservedApplicationInspection<'original> {
        WorthQueryObservedApplicationInspection {
            original: self.original,
            report: self.report,
            _memory: memory,
        }
    }
}
