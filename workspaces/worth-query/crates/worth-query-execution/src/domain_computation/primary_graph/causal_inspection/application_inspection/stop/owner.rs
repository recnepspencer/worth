//! Preserve exact admission causes through the public inspection denial.
use super::super::{admission::InspectionPlanDenial, connection::ConnectionRefusal};
use super::*;
pub use crate::domain_computation::authorization::application_disclosure::contract::WorthQueryApplicationDisclosureContractDenial;
pub use crate::domain_computation::authorization::application_disclosure::DiagnosticComparisonDenial;
use crate::domain_computation::authorization::application_disclosure::DiagnosticContractDenial;
pub use crate::domain_computation::authorization::application_disclosure::WorthQueryApplicationQueryGovernanceDenialKind;
use {crate::domain_computation::authorization::application_disclosure::DiagnosticPreparationDenial, crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial, crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind};
pub use {crate::domain_computation::primary_graph::vocabulary::application_attempt::indexed_entity_selection::IndexedSelectionReobserveDenial, crate::domain_computation::primary_graph::application_attempt::OutputFieldCaptureFailure, crate::domain_computation::primary_graph::application_attempt::StoreDenial, crate::domain_computation::primary_graph::provider::FactlessCurrentness, crate::domain_computation::primary_graph::provider::OwnEffectOnReads, crate::domain_computation::primary_graph::vocabulary::provider::diagnostic::RebaseVerificationReason};
pub use crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSessionStartDenial;
pub use crate::domain_computation::provider_session::{
    WorthQueryInspectionReadTerminalDenial, WorthQueryManagedGraphReadDenial,
};
use worth_execution::{MapKernelFailure, ScopeStop};
mod stage;
use stage::{capture, storage};
pub use worth_query_installation::facade::{
    WorthQueryApplicationQueryInstallationDenial, WorthQueryApplicationQueryInstallationDenialKind,
};

/// The retained owner cause; no protected observation payload is projected.
#[derive(Clone, Copy, Debug)]
pub enum WorthQueryApplicationInspectionOwnerRefusal<'a> {
    Authorization(&'a WorthQueryApplicationQueryAdmissionDenial),
    CurrentAuthorization(&'a WorthQueryOperationAuthorizationDenialKind),
    Capability(&'a WorthQueryOperationAuthorizationDenial),
    Scope(&'a WorthQueryEntityResolutionDenial),
    Security(&'a crate::basis::WorthQueryProductBranchAdmissionDenial),
    Governance(&'a WorthQueryApplicationQueryGovernanceDenialKind),
    DiagnosticPreparationGovernance(&'a WorthQueryApplicationQueryGovernanceDenialKind),
    Principal(&'a WorthQueryPrincipalResolutionDenial),
    Installation(
        &'a worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenial,
    ),
    DiagnosticContract(&'a WorthQueryApplicationDisclosureContractDenial),
    Graph(&'a worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkAdmissionDenial),
    ReportingInput(&'a worth_runtime_bridge::facade::BridgeQueryObservationReportingInputDenial),
    ForeignTarget,
    ProducerCapture(&'a OutputFieldCaptureFailure),
    Session(&'a WorthQueryManagedGraphWorkSessionStartDenial),
    Advancement(&'a WorthQueryAdvancementDenial),
    DiagnosticAdmission(&'a DiagnosticComparisonDenial),
    Read(&'a WorthQueryManagedGraphReadDenial),
    Terminal(&'a WorthQueryInspectionReadTerminalDenial),
    Unavailable,
    ForeignIssuedAccess,
    MissingGraph,
    MissingInspection,
    FacetDenied,
    MissingOriginalProjection,
    DiagnosticContractNotDeclared,
    DiagnosticContractUnsupportedShape,
    ComparisonParametersUnavailable,
    SourceUnresolvedField,
    SourceMissingOriginalValue,
    ComparisonDisclosureInterpretationChanged,
    ComparisonOriginalDisclosureIncomplete,
    ComparisonNestingExceedsDeclaredShape,
    PlanAccountingOverflow,
    ReportForeignComparison,
    ReportForeignMaterialization,
    ReportIdentityOverflow,
    ReportIdentityForeignSession,
    ReportIdentityProductRequired,
    ReportBridgeAccountingOverflow,
    ReportBridgeInvalidReportingAssociation,
    StorageAccountingOverflow(WorthQueryApplicationInspectionStage),
    KernelPanic(WorthQueryApplicationInspectionStage),
    KernelResultCapacityExceeded(WorthQueryApplicationInspectionStage),
}
/// The actual owner boundary that returned a common runtime or storage failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationInspectionStage {
    Binding,
    DiagnosticAdmission,
    ComparisonParameters,
    Connection,
    SourceMaterialization,
    QueryMaterialization,
    Comparison,
    DiagnosticContract,
    DiagnosticPreparation,
    Target,
    Storage,
    Plan,
    Report,
    ReportIdentity,
    ReportBridge,
}
use crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenial;
use crate::domain_computation::primary_graph::vocabulary::entity_resolution_denial::WorthQueryEntityResolutionDenial;
use crate::domain_computation::vocabulary::primary_graph::resolution_denial::WorthQueryPrincipalResolutionDenial;
use WorthQueryApplicationInspectionStage as Stage;

type Refusal<'a> = WorthQueryApplicationInspectionOwnerRefusal<'a>;

pub(super) fn cause(cause: &InspectionCause) -> Option<Refusal<'_>> {
    match cause {
        InspectionCause::Query(cause) => Some(Refusal::Authorization(cause)),
        InspectionCause::Scope(cause) => Some(Refusal::Scope(cause)),
        InspectionCause::Selection(cause) => Some(Refusal::Security(cause)),
        InspectionCause::Principal(cause) => Some(Refusal::Principal(cause)),
        InspectionCause::Unavailable => Some(Refusal::Unavailable),
        InspectionCause::Advancement(cause) => Some(Refusal::Advancement(cause)),
        InspectionCause::Binding(stop) => capture(Stage::Binding, stop, |cause| match cause {
            InspectionBindingDenial::Installation(cause) => Some(Refusal::Installation(cause)),
            InspectionBindingDenial::Storage(cause) => storage(Stage::Binding, cause),
        }),
        InspectionCause::Admission(cause) => admission(cause),
    }
}

fn admission(cause: &InspectionAdmissionDenial) -> Option<Refusal<'_>> {
    use InspectionAdmissionDenial as Cause;
    match cause {
        Cause::Authorization(cause) => Some(Refusal::Authorization(cause)),
        Cause::CurrentAuthorization(cause) => Some(Refusal::CurrentAuthorization(cause)),
        Cause::Capability(cause) => Some(Refusal::Capability(cause)),
        Cause::Scope(cause) => Some(Refusal::Scope(cause)),
        Cause::Security(cause) => Some(Refusal::Security(cause)),
        Cause::Session(cause) => Some(Refusal::Session(cause)),
        Cause::Read(cause) => Some(Refusal::Read(cause)),
        Cause::Terminal(cause) => Some(Refusal::Terminal(cause)),
        Cause::DiagnosticAdmission(stop) => capture(Stage::DiagnosticAdmission, stop, |cause| {
            Some(Refusal::DiagnosticAdmission(cause))
        }),
        Cause::ForeignIssuedAccess => Some(Refusal::ForeignIssuedAccess),
        Cause::MissingGraph => Some(Refusal::MissingGraph),
        Cause::MissingInspection => Some(Refusal::MissingInspection),
        Cause::FacetDenied => Some(Refusal::FacetDenied),
        Cause::MissingOriginalProjection => Some(Refusal::MissingOriginalProjection),
        // Exact request causes already have resource_refusal/request_interruption readers.
        Cause::Request(_) | Cause::RequestControl(_) => None,
        Cause::Connection(stop) => capture(Stage::Connection, stop, |cause| match cause {
            ConnectionRefusal::Capture(cause) => Some(Refusal::ProducerCapture(cause)),
            // These retain exact dedicated producer_connection/output readers.
            ConnectionRefusal::General(_) | ConnectionRefusal::Output(_) => None,
        }),
        Cause::Target(cause) => match cause {
            super::super::target::InspectionTargetDenial::ForeignBinding => {
                Some(Refusal::ForeignTarget)
            }
            super::super::target::InspectionTargetDenial::Storage(stop) => {
                capture(Stage::Target, stop, |cause| storage(Stage::Target, cause))
            }
        },
        Cause::DiagnosticContract(stop) => {
            capture(Stage::DiagnosticContract, stop, |cause| match cause {
                DiagnosticContractDenial::Contract(cause) => {
                    Some(Refusal::DiagnosticContract(cause))
                }
                DiagnosticContractDenial::NotDeclared => {
                    Some(Refusal::DiagnosticContractNotDeclared)
                }
                DiagnosticContractDenial::UnsupportedShape => {
                    Some(Refusal::DiagnosticContractUnsupportedShape)
                }
                DiagnosticContractDenial::Storage(cause) => {
                    storage(Stage::DiagnosticContract, cause)
                }
            })
        }
        Cause::Plan(stop) => capture(Stage::Plan, stop, |cause| match cause {
            InspectionPlanDenial::Graph(cause) => Some(Refusal::Graph(cause)),
            InspectionPlanDenial::Storage(cause) => storage(Stage::Plan, cause),
            InspectionPlanDenial::Overflow => Some(Refusal::PlanAccountingOverflow),
        }),
        Cause::Report(stop) => capture(Stage::Report, stop, stage::report),
        Cause::Governance(cause) => Some(Refusal::Governance(cause)),
        Cause::DiagnosticPreparation(stop) => {
            capture(Stage::DiagnosticPreparation, stop, |cause| match cause {
                DiagnosticPreparationDenial::Governance(cause) => {
                    Some(Refusal::DiagnosticPreparationGovernance(cause))
                }
            })
        }
        Cause::ComparisonParameters(stop) => capture(Stage::ComparisonParameters, stop, |()| {
            Some(Refusal::ComparisonParametersUnavailable)
        }),
        // The public materialization_refusal reader retains the complete domain cause.
        Cause::Rich(stop) => capture(Stage::QueryMaterialization, stop, |cause| {
            use crate::domain_computation::primary_graph::application_query::WorthQueryProjectionMaterializationRefusal as Materialization;
            match cause {
                Materialization::DisclosureInterpretationChanged
                | Materialization::OriginalDisclosureIncomplete => None,
            }
        }),
        Cause::RichSource(stop) => capture(Stage::SourceMaterialization, stop, |cause| {
            use super::super::rich::source::SourceMaterializationDenial as Source;
            match cause {
                Source::UnresolvedField => Some(Refusal::SourceUnresolvedField),
                Source::MissingOriginalValue => Some(Refusal::SourceMissingOriginalValue),
                Source::Array(_) => None, // resource_refusal exposes the exact array denial.
            }
        }),
        Cause::Comparison(stop) => capture(Stage::Comparison, stop, |cause| {
            use crate::domain_computation::primary_graph::application_query::ProjectionComparisonDenial as Comparison;
            match cause {
                Comparison::DisclosureInterpretationChanged => {
                    Some(Refusal::ComparisonDisclosureInterpretationChanged)
                }
                Comparison::OriginalDisclosureIncomplete => {
                    Some(Refusal::ComparisonOriginalDisclosureIncomplete)
                }
                Comparison::NestingExceedsDeclaredShape => {
                    Some(Refusal::ComparisonNestingExceedsDeclaredShape)
                }
                Comparison::Storage(cause) => storage(Stage::Comparison, cause),
                Comparison::Allocation(_) => None, // exact resource_refusal allocation reader.
            }
        }),
        Cause::Storage(stop) => {
            capture(Stage::Storage, stop, |cause| storage(Stage::Storage, cause))
        }
    }
}

#[cfg(test)]
mod extended_tests;
#[cfg(test)]
mod lifecycle_tests;
#[cfg(test)]
mod stage_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod capture_stage_tests;
