//! Original owner causes and their backing survive the complete fresh request.
use super::{admission::InspectionAdmissionDenial, request::binding::InspectionBindingDenial};
use worth_execution::{ExecutionMemoryReservation, WorkCeilingDenial};
use {
    crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenial,
    crate::domain_computation::primary_graph::vocabulary::entity_resolution_denial::WorthQueryEntityResolutionDenial,
    crate::domain_computation::request_execution::WorthQueryAdvancementDenial,
    crate::domain_computation::vocabulary::primary_graph::admitted_capture::CaptureStop,
    crate::domain_computation::vocabulary::primary_graph::resolution_denial::WorthQueryPrincipalResolutionDenial,
};
mod captured_domain;
mod kind;
mod resource;
pub use resource::WorthQueryApplicationInspectionResourceRefusal;
mod canonical;
pub use canonical::WorthQueryApplicationInspectionCanonicalRefusal;
mod owner;
pub use {owner::DiagnosticComparisonDenial, owner::FactlessCurrentness, crate::domain_computation::primary_graph::vocabulary::application_attempt::indexed_entity_selection::IndexedSelectionReobserveDenial, owner::OutputFieldCaptureFailure, owner::OwnEffectOnReads, crate::domain_computation::primary_graph::vocabulary::provider::diagnostic::RebaseVerificationReason, owner::StoreDenial, owner::WorthQueryApplicationDisclosureContractDenial, owner::WorthQueryApplicationInspectionOwnerRefusal, owner::WorthQueryApplicationInspectionStage, owner::WorthQueryApplicationQueryGovernanceDenialKind, owner::WorthQueryApplicationQueryInstallationDenial, owner::WorthQueryApplicationQueryInstallationDenialKind, owner::WorthQueryInspectionReadTerminalDenial, owner::WorthQueryManagedGraphReadDenial, owner::WorthQueryManagedGraphWorkSessionStartDenial};
#[cfg(test)]
mod output_tests;

pub enum InspectionCause {
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
        self.connection_refusal().map(|cause| cause.kind())
    }

    /// Evaluation refusal after both original source and query diagnostics were admitted.
    pub fn producer_evaluation_refusal(&self) -> Option<super::WorthQueryProducerEvaluationDenial> {
        match self.producer_connection_refusal()? {
            super::WorthQueryProducerConnectionDenial::Evaluation(cause) => Some(cause),
            super::WorthQueryProducerConnectionDenial::ForeignSource
            | super::WorthQueryProducerConnectionDenial::SourceDisclosureUnavailable
            | super::WorthQueryProducerConnectionDenial::SourceDisclosureMismatch
            | super::WorthQueryProducerConnectionDenial::ProducerUnavailable
            | super::WorthQueryProducerConnectionDenial::PerformedInputsUnavailable
            | super::WorthQueryProducerConnectionDenial::SourceInputUnconnected
            | super::WorthQueryProducerConnectionDenial::OutputUnconnected => None,
        }
    }

    pub fn materialization_refusal(&self)->Option<crate::domain_computation::primary_graph::application_query::WorthQueryProjectionMaterializationRefusal>{
        match &self.cause {
            InspectionCause::Admission(cause) => match cause {
                InspectionAdmissionDenial::Rich(stop) => captured_domain::captured(stop).copied(),
                InspectionAdmissionDenial::Request(_)
                | InspectionAdmissionDenial::RequestControl(_)
                | InspectionAdmissionDenial::ForeignIssuedAccess
                | InspectionAdmissionDenial::MissingGraph
                | InspectionAdmissionDenial::MissingInspection
                | InspectionAdmissionDenial::Authorization(_)
                | InspectionAdmissionDenial::CurrentAuthorization(_)
                | InspectionAdmissionDenial::Capability(_)
                | InspectionAdmissionDenial::Scope(_)
                | InspectionAdmissionDenial::Security(_)
                | InspectionAdmissionDenial::Governance(_)
                | InspectionAdmissionDenial::FacetDenied
                | InspectionAdmissionDenial::DiagnosticAdmission(_)
                | InspectionAdmissionDenial::ComparisonParameters(_)
                | InspectionAdmissionDenial::Connection(_)
                | InspectionAdmissionDenial::MissingOriginalProjection
                | InspectionAdmissionDenial::RichSource(_)
                | InspectionAdmissionDenial::Comparison(_)
                | InspectionAdmissionDenial::DiagnosticContract(_)
                | InspectionAdmissionDenial::DiagnosticPreparation(_)
                | InspectionAdmissionDenial::Target(_)
                | InspectionAdmissionDenial::Storage(_)
                | InspectionAdmissionDenial::Plan(_)
                | InspectionAdmissionDenial::Session(_)
                | InspectionAdmissionDenial::Read(_)
                | InspectionAdmissionDenial::Terminal(_)
                | InspectionAdmissionDenial::Report(_) => None,
            },
            InspectionCause::Unavailable
            | InspectionCause::Advancement(_)
            | InspectionCause::Selection(_)
            | InspectionCause::Binding(_)
            | InspectionCause::Principal(_)
            | InspectionCause::Scope(_)
            | InspectionCause::Query(_) => None,
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
            super::connection::ConnectionRefusal::General(_)
            | super::connection::ConnectionRefusal::Capture(_) => None,
        }
    }

    fn connection_refusal(&self) -> Option<&super::connection::ConnectionRefusal> {
        match &self.cause {
            InspectionCause::Admission(cause) => match cause {
                InspectionAdmissionDenial::Connection(stop) => captured_domain::captured(stop),
                InspectionAdmissionDenial::Request(_)
                | InspectionAdmissionDenial::RequestControl(_)
                | InspectionAdmissionDenial::ForeignIssuedAccess
                | InspectionAdmissionDenial::MissingGraph
                | InspectionAdmissionDenial::MissingInspection
                | InspectionAdmissionDenial::Authorization(_)
                | InspectionAdmissionDenial::CurrentAuthorization(_)
                | InspectionAdmissionDenial::Capability(_)
                | InspectionAdmissionDenial::Scope(_)
                | InspectionAdmissionDenial::Security(_)
                | InspectionAdmissionDenial::Governance(_)
                | InspectionAdmissionDenial::FacetDenied
                | InspectionAdmissionDenial::DiagnosticAdmission(_)
                | InspectionAdmissionDenial::ComparisonParameters(_)
                | InspectionAdmissionDenial::MissingOriginalProjection
                | InspectionAdmissionDenial::RichSource(_)
                | InspectionAdmissionDenial::Rich(_)
                | InspectionAdmissionDenial::Comparison(_)
                | InspectionAdmissionDenial::DiagnosticContract(_)
                | InspectionAdmissionDenial::DiagnosticPreparation(_)
                | InspectionAdmissionDenial::Target(_)
                | InspectionAdmissionDenial::Storage(_)
                | InspectionAdmissionDenial::Plan(_)
                | InspectionAdmissionDenial::Session(_)
                | InspectionAdmissionDenial::Read(_)
                | InspectionAdmissionDenial::Terminal(_)
                | InspectionAdmissionDenial::Report(_) => None,
            },
            InspectionCause::Unavailable
            | InspectionCause::Advancement(_)
            | InspectionCause::Selection(_)
            | InspectionCause::Binding(_)
            | InspectionCause::Principal(_)
            | InspectionCause::Scope(_)
            | InspectionCause::Query(_) => None,
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
            InspectionCause::Unavailable => Kind::Unavailable,
            InspectionCause::Advancement(_) | InspectionCause::Binding(CaptureStop::Request(_)) => {
                Kind::Request
            }
            InspectionCause::Admission(cause) => kind::admission(cause),
            InspectionCause::Binding(CaptureStop::Kernel(_))
            | InspectionCause::Selection(_)
            | InspectionCause::Principal(_)
            | InspectionCause::Scope(_)
            | InspectionCause::Query(_) => Kind::Admission,
        }
    }

    pub fn execution_stop(&self) -> Option<&WorkCeilingDenial> {
        match self.resource_refusal() {
            Some(WorthQueryApplicationInspectionResourceRefusal::Request(cause)) => Some(cause),
            None
            | Some(
                WorthQueryApplicationInspectionResourceRefusal::Advancement(_)
                | WorthQueryApplicationInspectionResourceRefusal::Kernel(_)
                | WorthQueryApplicationInspectionResourceRefusal::Native(_)
                | WorthQueryApplicationInspectionResourceRefusal::Lease(_)
                | WorthQueryApplicationInspectionResourceRefusal::Memory(_)
                | WorthQueryApplicationInspectionResourceRefusal::Allocation(_)
                | WorthQueryApplicationInspectionResourceRefusal::Array(_),
            ) => None,
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

    /// The original owner's exact refusal, borrowed without replacing its cause.
    pub fn owner_refusal(&self) -> Option<WorthQueryApplicationInspectionOwnerRefusal<'_>> {
        owner::cause(&self.cause)
    }

    pub fn request_interruption(
        &self,
    ) -> Option<worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption>
    {
        match &self.cause {
            InspectionCause::Admission(cause) => match cause {
                InspectionAdmissionDenial::RequestControl(cause) => Some(*cause),
                InspectionAdmissionDenial::Request(_)
                | InspectionAdmissionDenial::ForeignIssuedAccess
                | InspectionAdmissionDenial::MissingGraph
                | InspectionAdmissionDenial::MissingInspection
                | InspectionAdmissionDenial::Authorization(_)
                | InspectionAdmissionDenial::CurrentAuthorization(_)
                | InspectionAdmissionDenial::Capability(_)
                | InspectionAdmissionDenial::Scope(_)
                | InspectionAdmissionDenial::Security(_)
                | InspectionAdmissionDenial::Governance(_)
                | InspectionAdmissionDenial::FacetDenied
                | InspectionAdmissionDenial::DiagnosticAdmission(_)
                | InspectionAdmissionDenial::ComparisonParameters(_)
                | InspectionAdmissionDenial::Connection(_)
                | InspectionAdmissionDenial::MissingOriginalProjection
                | InspectionAdmissionDenial::RichSource(_)
                | InspectionAdmissionDenial::Rich(_)
                | InspectionAdmissionDenial::Comparison(_)
                | InspectionAdmissionDenial::DiagnosticContract(_)
                | InspectionAdmissionDenial::DiagnosticPreparation(_)
                | InspectionAdmissionDenial::Target(_)
                | InspectionAdmissionDenial::Storage(_)
                | InspectionAdmissionDenial::Plan(_)
                | InspectionAdmissionDenial::Session(_)
                | InspectionAdmissionDenial::Read(_)
                | InspectionAdmissionDenial::Terminal(_)
                | InspectionAdmissionDenial::Report(_) => None,
            },
            InspectionCause::Unavailable
            | InspectionCause::Advancement(_)
            | InspectionCause::Selection(_)
            | InspectionCause::Binding(_)
            | InspectionCause::Principal(_)
            | InspectionCause::Scope(_)
            | InspectionCause::Query(_) => None,
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
mod projection_tests;

mod court_doors_for_test;
