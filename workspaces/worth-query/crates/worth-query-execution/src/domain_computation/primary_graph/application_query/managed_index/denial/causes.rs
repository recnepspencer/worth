//! Closed descriptive cause projection; private proof/session types stay private.
use super::super::{
    execution::ProviderExecutionDenial, retention::WorthQueryManagedIndexLifecycleDenial,
    selection::SelectionDenial, source_acquisition::AcquisitionCause,
    source_completion::SourceDenial,
};
use crate::domain_computation::{
    primary_graph::application_query,
    provider_session::{
        WorthQueryManagedGraphReadDenial, WorthQueryManagedGraphWorkSessionStartDenial,
        WorthQueryManagedIndexSessionDenial,
    },
};
use worth_query_admission::facade::graph_obligation::{
    WorthQueryGraphWorkAdmissionDenial, WorthQueryGraphWorkCapacityAdmissionStop,
};

/// A closed refusal without a newly owned diagnostic payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedIndexRefusal {
    ForeignPhase,
    ForeignBinding,
    UnsupportedMode,
    NotIndexParent,
    MissingProvider,
    MissingProbe,
    ForeignAssociation,
    SessionAuthorityMismatch,
    ForeignSelection,
    ForeignContext,
    Disposed,
    BuildInProgress,
    EpochExhausted,
    StaleImage,
    NotReady,
    RegistrationCapacityExceeded,
    RegistrationIdentityExhausted,
    SessionIdentityExhausted,
    ManagedRunIdentityExhausted,
    PlanAffinityMismatch,
    BasisBranchMismatch,
    TerminalReleaseMismatch,
    SnapshotUnavailable,
    IndexNotInstalled(worth_relational::facade::indexes::DerivedIndexId),
    MutationSession,
    ForeignBasis,
    ForeignGraph,
    ForeignReadProof,
    ForeignReadIntent,
    InstallationAccountingOverflow,
    CapacityAccountingOverflow,
}

/// The actual original cause, borrowed from the owned operation denial.
#[derive(Debug)]
pub enum WorthQueryManagedIndexDenialCause<'a> {
    Refused(WorthQueryManagedIndexRefusal),
    Execution(&'a worth_relational::facade::transactions::RelationalExecutionDenialCause),
    Request(worth_execution::WorkCeilingDenial),
    Interrupted(
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption,
    ),
    GraphWork(&'a WorthQueryGraphWorkAdmissionDenial),
    Installation(
        &'a worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenial,
    ),
    Product(&'a crate::basis::WorthQueryProductBranchAdmissionDenial),
    Principal(&'a crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial),
    Scope(&'a crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial),
    QueryAdmission(&'a application_query::WorthQueryApplicationQueryAdmissionDenial),
    QueryRead(&'a application_query::WorthQueryApplicationOneShotDenial),
    Source(&'a application_query::WorthQuerySourceExpectationDenial),
    Work(&'a super::super::WorthQueryManagedIndexWorkDenial),
}
use WorthQueryManagedIndexDenialCause as Cause;
use WorthQueryManagedIndexRefusal as Refusal;

pub(super) fn provider(cause: &ProviderExecutionDenial) -> Cause<'_> {
    match cause {
        ProviderExecutionDenial::Execution(cause) => Cause::Execution(cause),
        ProviderExecutionDenial::Capacity(cause) => match cause {
            WorthQueryGraphWorkCapacityAdmissionStop::Admission(cause) => Cause::Execution(cause),
            WorthQueryGraphWorkCapacityAdmissionStop::Denial(cause) => Cause::GraphWork(cause),
            WorthQueryGraphWorkCapacityAdmissionStop::AccountingOverflow => {
                Cause::Refused(Refusal::CapacityAccountingOverflow)
            }
        },
        ProviderExecutionDenial::Identity(cause) => Cause::Refused(match cause {
            WorthQueryManagedGraphWorkSessionStartDenial::PlanAffinityMismatch => {
                Refusal::PlanAffinityMismatch
            }
            WorthQueryManagedGraphWorkSessionStartDenial::BasisBranchMismatch => {
                Refusal::BasisBranchMismatch
            }
            WorthQueryManagedGraphWorkSessionStartDenial::IdentityExhausted => {
                Refusal::SessionIdentityExhausted
            }
            WorthQueryManagedGraphWorkSessionStartDenial::ManagedRunIdentityExhausted => {
                Refusal::ManagedRunIdentityExhausted
            }
        }),
        ProviderExecutionDenial::Session(cause) => session(cause),
        ProviderExecutionDenial::Acquisition(cause) => match &cause.cause {
            AcquisitionCause::ForeignContext => Cause::Refused(Refusal::ForeignContext),
            AcquisitionCause::Execution(cause) => Cause::Execution(cause),
            AcquisitionCause::Principal(cause) => Cause::Principal(cause),
            AcquisitionCause::Scope(cause) => Cause::Scope(cause),
            AcquisitionCause::Product(cause) => Cause::Product(cause),
            AcquisitionCause::Admission(cause) => Cause::QueryAdmission(cause),
            AcquisitionCause::Source(cause) => source(cause),
        },
        ProviderExecutionDenial::Lifecycle(cause) => lifecycle(cause),
        ProviderExecutionDenial::Work(cause) => Cause::Work(cause),
        ProviderExecutionDenial::Selection(cause) => match cause {
            SelectionDenial::Session(cause) => session(cause),
            SelectionDenial::Work(cause) => Cause::Work(cause),
            SelectionDenial::Execution(cause) => Cause::Execution(cause),
            SelectionDenial::ForeignSelection => Cause::Refused(Refusal::ForeignSelection),
        },
        ProviderExecutionDenial::ForeignAssociation => Cause::Refused(Refusal::ForeignAssociation),
    }
}
fn source(cause: &SourceDenial) -> Cause<'_> {
    match cause {
        SourceDenial::Read(cause) => Cause::QueryRead(cause),
        SourceDenial::Work(cause) => Cause::Work(cause),
        SourceDenial::Source(cause) => Cause::Source(cause),
    }
}
pub(super) fn lifecycle(cause: &WorthQueryManagedIndexLifecycleDenial) -> Cause<'_> {
    use WorthQueryManagedIndexLifecycleDenial as Denial;
    match cause {
        Denial::Execution(cause) => Cause::Execution(cause),
        Denial::Session(cause) => session(cause),
        Denial::ForeignAssociation => Cause::Refused(Refusal::ForeignAssociation),
        Denial::RegistrationCapacityExceeded => {
            Cause::Refused(Refusal::RegistrationCapacityExceeded)
        }
        Denial::RegistrationIdentityExhausted => {
            Cause::Refused(Refusal::RegistrationIdentityExhausted)
        }
        Denial::Disposed => Cause::Refused(Refusal::Disposed),
        Denial::BuildInProgress => Cause::Refused(Refusal::BuildInProgress),
        Denial::EpochExhausted => Cause::Refused(Refusal::EpochExhausted),
        Denial::StaleImage => Cause::Refused(Refusal::StaleImage),
        Denial::NotReady => Cause::Refused(Refusal::NotReady),
    }
}
fn session(cause: &WorthQueryManagedIndexSessionDenial) -> Cause<'_> {
    use WorthQueryManagedIndexSessionDenial as Denial;
    match cause {
        Denial::Request(cause) => Cause::Request(*cause),
        Denial::Interrupted(cause) => Cause::Interrupted(*cause),
        Denial::Execution(cause) => Cause::Execution(cause),
        Denial::Product(cause) => Cause::Product(cause),
        Denial::AuthorityMismatch => Cause::Refused(Refusal::SessionAuthorityMismatch),
        Denial::TerminalReleaseMismatch => Cause::Refused(Refusal::TerminalReleaseMismatch),
        Denial::Parent(cause) => match cause {
            WorthQueryManagedGraphReadDenial::RequestStop(cause) => {
                Cause::Request(worth_execution::WorkCeilingDenial::Stopped(*cause))
            }
            WorthQueryManagedGraphReadDenial::SnapshotUnavailable => {
                Cause::Refused(Refusal::SnapshotUnavailable)
            }
            WorthQueryManagedGraphReadDenial::IndexNotInstalled(id) => {
                Cause::Refused(Refusal::IndexNotInstalled(*id))
            }
            WorthQueryManagedGraphReadDenial::MutationSession => {
                Cause::Refused(Refusal::MutationSession)
            }
            WorthQueryManagedGraphReadDenial::ForeignBasis => Cause::Refused(Refusal::ForeignBasis),
            WorthQueryManagedGraphReadDenial::ForeignGraph => Cause::Refused(Refusal::ForeignGraph),
            WorthQueryManagedGraphReadDenial::ForeignReadProof => {
                Cause::Refused(Refusal::ForeignReadProof)
            }
            WorthQueryManagedGraphReadDenial::ForeignReadIntent => {
                Cause::Refused(Refusal::ForeignReadIntent)
            }
            WorthQueryManagedGraphReadDenial::TerminalReleaseMismatch => {
                Cause::Refused(Refusal::TerminalReleaseMismatch)
            }
        },
    }
}
