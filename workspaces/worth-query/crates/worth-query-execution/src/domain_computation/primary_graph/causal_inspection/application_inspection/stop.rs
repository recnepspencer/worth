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
#[cfg(test)]
mod output_tests;

pub(super) enum InspectionCause {
    Unavailable,
    Advancement(WorthQueryAdvancementDenial),
    #[allow(
        dead_code,
        reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
    )]
    Selection(crate::basis::WorthQueryProductBranchAdmissionDenial),
    Binding(CaptureStop<InspectionBindingDenial>),
    #[allow(
        dead_code,
        reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
    )]
    Principal(WorthQueryPrincipalResolutionDenial),
    #[allow(
        dead_code,
        reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
    )]
    Scope(WorthQueryEntityResolutionDenial),
    #[allow(
        dead_code,
        reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
    )]
    Query(WorthQueryApplicationQueryAdmissionDenial),
    Admission(InspectionAdmissionDenial),
}

/// A coarse projection that exposes no protected original observation metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationInspectionStopKind {
    StaleIncarnation,
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
        self.connection_refusal().map(|cause| cause.kind())
    }

    /// Evaluation refusal after both original source and query diagnostics were admitted.
    pub fn producer_evaluation_refusal(&self) -> Option<super::WorthQueryProducerEvaluationDenial> {
        match self.producer_connection_refusal()? {
            super::WorthQueryProducerConnectionDenial::Evaluation(cause) => Some(cause),
            _ => None,
        }
    }

    pub fn materialization_refusal(&self)->Option<crate::domain_computation::primary_graph::application_query::WorthQueryProjectionMaterializationRefusal>{
        match &self.cause {
            InspectionCause::Admission(super::admission::InspectionAdmissionDenial::Rich(
                CaptureStop::Kernel(worth_execution::ScopeStop::Failure(
                    worth_execution::MapKernelFailure::Domain(cause),
                )),
            )) => Some(*cause),
            _ => None,
        }
    }

    /// Output join detail after fresh diagnostic admission; no identifiers or values.
    /// With multiple disclosed candidates, reports the longest satisfied prefix,
    /// retaining the first candidate in original order when equally deep.
    pub fn producer_output_refusal(
        &self,
    ) -> Option<super::WorthQueryProducerOutputConnectionDenial> {
        match self.connection_refusal()? {
            super::connection::ConnectionRefusal::Output(cause) => Some(*cause),
            _ => None,
        }
    }

    fn connection_refusal(&self) -> Option<&super::connection::ConnectionRefusal> {
        match &self.cause {
            InspectionCause::Admission(InspectionAdmissionDenial::Connection(
                CaptureStop::Kernel(worth_execution::ScopeStop::Failure(
                    worth_execution::MapKernelFailure::Domain(cause),
                )),
            )) => Some(cause),
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
            InspectionCause::Admission(InspectionAdmissionDenial::Capability(denial))
                if denial.kind() == crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind::StaleIncarnation) => Kind::StaleIncarnation,
            InspectionCause::Admission(InspectionAdmissionDenial::CurrentAuthorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind::StaleIncarnation)))
            | InspectionCause::Admission(InspectionAdmissionDenial::Security(crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(crate::basis::WorthQueryProductBranchAdmissionRefusalKind::StaleIncarnation))) => Kind::StaleIncarnation,
            InspectionCause::Admission(InspectionAdmissionDenial::Authorization(denial))
                if denial.kind() == crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind::Authorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind::StaleIncarnation)) => Kind::StaleIncarnation,
            InspectionCause::Query(denial) if denial.kind() == crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind::Authorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind::StaleIncarnation)) => Kind::StaleIncarnation,
            InspectionCause::Selection(crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(crate::basis::WorthQueryProductBranchAdmissionRefusalKind::StaleIncarnation)) => Kind::StaleIncarnation,
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
            InspectionCause::Selection(_) | InspectionCause::Binding(_)
            | InspectionCause::Principal(_) | InspectionCause::Scope(_)
            | InspectionCause::Query(_) | InspectionCause::Admission(_) => Kind::Admission,
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

#[cfg(test)]
mod incarnation_projection_tests {
    use super::*;
    use crate::domain_computation::authorization::{
        WorthQueryOperationAuthorizationDenial as Denial,
        WorthQueryOperationAuthorizationDenialKind as Kind,
        WorthQueryOperationAuthorizationRefusalKind as Refusal,
    };
    #[test]
    fn stale_disclosure_is_distinct_from_foreign_runtime_in_public_inspection_stop_kind() {
        // Error metadata only: no capability, runtime or Home is fabricated.
        for (cause, expected) in [
            (
                Refusal::StaleIncarnation,
                WorthQueryApplicationInspectionStopKind::StaleIncarnation,
            ),
            (
                Refusal::ForeignRuntime,
                WorthQueryApplicationInspectionStopKind::Admission,
            ),
        ] {
            let denied = WorthQueryApplicationInspectionDenial::new(
                InspectionCause::Admission(InspectionAdmissionDenial::Capability(Denial::new(
                    Kind::Refused(cause),
                    "disclosure",
                ))),
                None,
            );
            assert_eq!(denied.kind(), expected);
        }
    }
}
