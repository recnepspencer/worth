//! Closed descriptive cause projection; private proof/session types stay private.
use crate::domain_computation::primary_graph::application_query;
use worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkAdmissionDenial;

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
    /// The selected physical Source has no retained managed material.
    RootUnavailable,
    InvariantOwned,
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
    /// A coupled successor round probed a record its own chain wrote: the
    /// applied candidate holds the chain's effect there, so the round stops
    /// (`PriorEffectRead`) before the probe returns a row.
    PriorEffectRead,
}

/// The actual original cause, borrowed from the owned operation denial.
#[derive(Debug, Eq, PartialEq)]
pub enum WorthQueryManagedIndexDenialCause<'a> {
    CurrentControl(&'a crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial),
    SourceVerification {
        kind: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenialKind,
        authorization: Option<
            crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind,
        >,
        execution: Option<worth_relational::facade::transactions::RelationalExecutionDenialCause>,
    },
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
    LiveAdmission(&'a application_query::WorthQueryApplicationLiveOpenDenial),
    Source(&'a application_query::WorthQuerySourceExpectationDenial),
    Work(&'a super::super::WorthQueryManagedIndexWorkDenial),
    NativeSource(&'a worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial),
    NativeAdmission(&'a worth_relational::facade::mvcc::CompanionPreflightStop),
}
