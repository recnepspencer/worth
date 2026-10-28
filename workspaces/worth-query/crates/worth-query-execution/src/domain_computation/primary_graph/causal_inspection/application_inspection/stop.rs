//! Original owner causes and their backing survive the complete fresh request.
use super::{admission::InspectionAdmissionDenial, request::binding::InspectionBindingDenial};
use crate::domain_computation::primary_graph::{
    admitted_capture::CaptureStop, application_contribution::WorthQueryAdvancementDenial,
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryEntityResolutionDenial,
    WorthQueryPrincipalResolutionDenial,
};
use worth_execution::{ExecutionMemoryReservation, WorkCeilingDenial};
mod resource;
pub use resource::WorthQueryApplicationInspectionResourceRefusal;
mod canonical;
pub use canonical::WorthQueryApplicationInspectionCanonicalRefusal;

pub(super) enum InspectionCause {
    Unavailable,
    Advancement(WorthQueryAdvancementDenial),
    Selection(crate::basis::WorthQueryProductBranchAdmissionDenial),
    Binding(CaptureStop<InspectionBindingDenial>),
    Principal(WorthQueryPrincipalResolutionDenial),
    Scope(WorthQueryEntityResolutionDenial),
    Query(WorthQueryApplicationQueryAdmissionDenial),
    Admission(InspectionAdmissionDenial),
}

/// A coarse projection that exposes no protected original observation metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationInspectionStopKind {
    Unavailable,
    Request,
    Admission,
}

/// Owns the exact failing cause and the same original request's ticket, last.
pub struct WorthQueryApplicationInspectionDenial {
    pub(super) cause: InspectionCause,
    _memory: Option<ExecutionMemoryReservation>,
}

impl WorthQueryApplicationInspectionDenial {
    /// A requested source/producer join refused after fresh diagnostic admission.
    pub fn producer_connection_refusal(&self) -> Option<super::WorthQueryProducerConnectionDenial> {
        match &self.cause {
            InspectionCause::Admission(InspectionAdmissionDenial::Connection(
                CaptureStop::Kernel(worth_execution::ScopeStop::Failure(
                    worth_execution::MapKernelFailure::Domain(cause),
                )),
            )) => Some(*cause),
            _ => None,
        }
    }

    pub(super) fn new(cause: InspectionCause, memory: Option<ExecutionMemoryReservation>) -> Self {
        Self {
            cause,
            _memory: memory,
        }
    }

    pub fn kind(&self) -> WorthQueryApplicationInspectionStopKind {
        use WorthQueryApplicationInspectionStopKind as Kind;
        if self.resource_refusal().is_some() {
            return Kind::Request;
        }
        match &self.cause {
            InspectionCause::Unavailable
            | InspectionCause::Admission(InspectionAdmissionDenial::MissingOriginalProjection)
            | InspectionCause::Admission(InspectionAdmissionDenial::DiagnosticContract(
                CaptureStop::Kernel(worth_execution::ScopeStop::Failure(worth_execution::MapKernelFailure::Domain(
                    crate::domain_computation::authorization::application_disclosure::DiagnosticContractDenial::NotDeclared
                    | crate::domain_computation::authorization::application_disclosure::DiagnosticContractDenial::UnsupportedShape,
                ))),
            )) => Kind::Unavailable,
            InspectionCause::Advancement(_)
            | InspectionCause::Binding(CaptureStop::Request(_))
            | InspectionCause::Admission(
                InspectionAdmissionDenial::Request(_)
                | InspectionAdmissionDenial::RequestControl(_),
            ) => Kind::Request,
            _ => Kind::Admission,
        }
    }

    pub fn execution_stop(&self) -> Option<&WorkCeilingDenial> {
        match self.resource_refusal() {
            Some(WorthQueryApplicationInspectionResourceRefusal::Request(cause)) => Some(cause),
            _ => None,
        }
    }

    /// The actual original resource stop, including nested capture/report stages.
    pub fn resource_refusal(&self) -> Option<WorthQueryApplicationInspectionResourceRefusal<'_>> {
        resource::cause(&self.cause)
    }

    /// Canonical grammar/compiler refusal, distinct from the original resource stop.
    pub fn canonical_refusal(&self) -> Option<WorthQueryApplicationInspectionCanonicalRefusal<'_>> {
        canonical::cause(&self.cause)
    }

    pub fn request_interruption(
        &self,
    ) -> Option<worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption>
    {
        match &self.cause {
            InspectionCause::Admission(InspectionAdmissionDenial::RequestControl(cause)) => {
                Some(*cause)
            }
            _ => None,
        }
    }
}

impl std::fmt::Debug for WorthQueryApplicationInspectionDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryApplicationInspectionDenial")
            .field("kind", &self.kind())
            .finish_non_exhaustive()
    }
}
