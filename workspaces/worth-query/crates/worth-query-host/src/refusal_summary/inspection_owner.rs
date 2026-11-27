//! An inspection's own owner cause, exhaustively: expired authentication
//! asks for a renewed session, technical stops keep their own category, and
//! every policy, scope or disclosure refusal reads as withheld evidence so a
//! refusal never reveals whether the withheld target exists.
use worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkAdmissionDenial as Graph;
use worth_query_execution::facade::application_inspection::{
    DiagnosticComparisonDenial as Diagnostic, WorthQueryApplicationInspectionOwnerRefusal as Owner,
    WorthQueryInspectionReadTerminalDenial as Terminal, WorthQueryManagedGraphReadDenial as Read,
    WorthQueryManagedGraphWorkSessionStartDenial as SessionStart,
};

use super::audit_inspection::FIELD_DETAIL_UNSUPPORTED;
use super::query_read::admission;
use super::request::{BUSY, CAPACITY, INTERNAL, STALE, STORAGE};
use super::stops::{authorization, native, principal_resolution, work};
use super::summary::{
    WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary,
    WorthQueryUserFacingRefusal,
};

pub(super) const SOURCE_UNAVAILABLE: Summary = Summary::with_message(
    Category::NotFound,
    "The record of how this was changed is no longer available.",
);
pub(super) const EVIDENCE_WITHHELD: Summary = Summary::with_message(
    Category::PermissionDenied,
    "You don't have permission to see how this was changed.",
);

/// The owner's exact cause, in end-user terms.
pub(super) fn owner(cause: Owner<'_>) -> Summary {
    match cause {
        Owner::Principal(denial) => withheld(principal_resolution(denial.kind())),
        Owner::Capability(denial) => withheld(authorization(&denial.kind())),
        Owner::CurrentAuthorization(kind) => withheld(authorization(kind)),
        Owner::Authorization(denial) => withheld(admission(&denial.kind())),
        Owner::Scope(denial) => withheld(denial.user_summary()),
        Owner::Advancement(cause) => cause.user_summary(),
        Owner::Read(cause) => read(cause),
        Owner::Terminal(Terminal::Request(cause)) => work(cause),
        Owner::Terminal(Terminal::Read(cause)) => read(cause),
        Owner::Graph(cause) => graph(cause),
        Owner::Session(
            SessionStart::IdentityExhausted | SessionStart::ManagedRunIdentityExhausted,
        )
        | Owner::KernelResultCapacityExceeded(_) => CAPACITY,
        Owner::Security(_) => BUSY,
        Owner::Governance(_) | Owner::DiagnosticPreparationGovernance(_) | Owner::FacetDenied => {
            EVIDENCE_WITHHELD
        }
        Owner::DiagnosticAdmission(cause) => diagnostic(cause),
        Owner::DiagnosticContractUnsupportedShape | Owner::DiagnosticContractNotDeclared => {
            FIELD_DETAIL_UNSUPPORTED
        }
        Owner::Unavailable
        | Owner::MissingOriginalProjection
        | Owner::ComparisonParametersUnavailable
        | Owner::ComparisonOriginalDisclosureIncomplete => SOURCE_UNAVAILABLE,
        Owner::ComparisonDisclosureInterpretationChanged => STALE,
        Owner::SourceMissingOriginalValue => STORAGE,
        // Foreign or misused authority, declaration defects, arithmetic
        // overflow and panics: report rather than retry.
        Owner::Session(SessionStart::PlanAffinityMismatch | SessionStart::BasisBranchMismatch)
        | Owner::Installation(_)
        | Owner::DiagnosticContract(_)
        | Owner::ReportingInput(_)
        | Owner::ForeignTarget
        | Owner::ProducerCapture(_)
        | Owner::ForeignIssuedAccess
        | Owner::MissingGraph
        | Owner::MissingInspection
        | Owner::SourceUnresolvedField
        | Owner::ComparisonNestingExceedsDeclaredShape
        | Owner::PlanAccountingOverflow
        | Owner::ReportForeignComparison
        | Owner::ReportForeignMaterialization
        | Owner::ReportIdentityOverflow
        | Owner::ReportIdentityForeignSession
        | Owner::ReportIdentityProductRequired
        | Owner::ReportBridgeAccountingOverflow
        | Owner::ReportBridgeInvalidReportingAssociation
        | Owner::StorageAccountingOverflow(_)
        | Owner::KernelPanic(_) => INTERNAL,
    }
}

/// Keeps technical categories and an expired session; renders every policy,
/// scope or disclosure refusal as withheld evidence.
fn withheld(summary: Summary) -> Summary {
    match summary.category() {
        Category::PermissionDenied
        | Category::NotFound
        | Category::Conflict
        | Category::Validation
        | Category::Unsupported => EVIDENCE_WITHHELD,
        Category::SessionExpired
        | Category::Capacity
        | Category::Interrupted
        | Category::Unavailable
        | Category::Storage
        | Category::Internal => summary,
    }
}

fn diagnostic(cause: &Diagnostic) -> Summary {
    match cause {
        Diagnostic::UnsupportedShape | Diagnostic::NotDeclared => FIELD_DETAIL_UNSUPPORTED,
        Diagnostic::FieldDenied
        | Diagnostic::RelationDenied
        | Diagnostic::InternalInfluenceDenied => EVIDENCE_WITHHELD,
        Diagnostic::ForeignBinding => INTERNAL,
    }
}

fn read(cause: &Read) -> Summary {
    match cause {
        Read::RequestStop(cause) => native(cause),
        Read::SnapshotUnavailable => BUSY,
        Read::IndexNotInstalled(_)
        | Read::MutationSession
        | Read::ForeignBasis
        | Read::ForeignGraph
        | Read::ForeignReadProof
        | Read::ForeignReadIntent => INTERNAL,
    }
}

fn graph(cause: &Graph) -> Summary {
    match cause {
        Graph::ExecutionResource(_)
        | Graph::CapacityUnavailable
        | Graph::PlanIdentityExhausted
        | Graph::CapacityReservationIdentityExhausted => CAPACITY,
        Graph::ProviderSupportUnavailable => BUSY,
        _ => INTERNAL,
    }
}
