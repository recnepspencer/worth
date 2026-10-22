//! Bank's action for each typed application commit refusal.
use super::{BankHttpDenial, BankHttpDenialKind, BankHttpNextAction};
use worth_query_host::facade::{
    admission::authentication_event::WorthQueryAuthenticationEventDenial as Authentication,
    application_contribution::{
        WorthQueryAdvancementDenial as Advancement,
        WorthQueryManagedComputationInterruption as Interruption,
        WorthQueryManagedComputationResourceDenial as Resource,
        WorthQueryMemoryLimitLevel as Level,
    },
    primary_graph::{
        WorthQueryApplicationAttemptDenialKind as Attempt, WorthQueryApplicationCommitDenialKind,
    },
};

pub(in crate::http::server) fn commit_denial(
    kind: WorthQueryApplicationCommitDenialKind,
) -> BankHttpDenial {
    use WorthQueryApplicationCommitDenialKind as Denial;
    match kind {
        Denial::SignedCreationAuthentication {
            kind: Authentication::Cancelled,
        }
        | Denial::SigningLifecycleObservation {
            kind: Attempt::Cancelled,
        } => super::super::advancement_denial::advancement(Advancement::Interrupted(
            Interruption::Cancelled,
        )),
        Denial::SignedCreationAuthentication {
            kind: Authentication::DeadlineExceeded,
        }
        | Denial::SigningLifecycleObservation {
            kind: Attempt::DeadlineExceeded,
        } => super::super::advancement_denial::advancement(Advancement::Interrupted(
            Interruption::DeadlineExceeded,
        )),
        Denial::SigningLifecycleObservation {
            kind: Attempt::ExecutionRequest(cause),
        } => super::denial::pending_execution_denial(cause),
        // Bank's money commands install neither native bodies nor signing
        // lifecycles. A new client attempt cannot configure those host lanes.
        Denial::OperationBodyExecutionRequired
        | Denial::SignedCreationAuthentication { .. }
        | Denial::SignedEntityImmutable
        | Denial::SigningLifecycleForgery
        | Denial::SigningLifecycleUnavailable
        | Denial::SigningLifecycleEvidenceCapacity { .. }
        | Denial::SigningLifecycleSubjectCapacity { .. }
        | Denial::SigningLifecycleRepresentationCapacity
        | Denial::SigningLifecycleObservation { .. } => operator_denial(),
        Denial::ExecutionResource { denial, .. } => execution_resource(denial),
        Denial::ExecutionWorkerPanicked { .. } => operator_denial(),
        Denial::ExecutionUncheckedCustomKernel { .. } => operator_denial(),
        Denial::ExecutionIdentitiesNotCanonical { .. } => operator_denial(),
        Denial::ProductBasisStale => {
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh)
        }
        Denial::CustomInvariantDenied => BankHttpDenial::new(
            BankHttpDenialKind::MalformedRequest,
            BankHttpNextAction::CorrectRequest,
        ),
        Denial::IdempotencyIntentDrift => BankHttpDenial::new(
            BankHttpDenialKind::Stale,
            BankHttpNextAction::CorrectRequest,
        ),
        // The key's earlier commit took effect; reading current state shows it.
        Denial::IdempotencyReceiptNotRetained { .. } => {
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh)
        }
        Denial::IdempotencyIntentUnverifiable => BankHttpDenial::new(
            BankHttpDenialKind::InternalDenied,
            BankHttpNextAction::ContactOperator,
        ),
        Denial::UniqueValueTaken
        | Denial::WorkflowSettlementDenied { .. }
        | Denial::ElevationTransitionRequired
        | Denial::ElevationRequestProgramMismatch
        | Denial::ElevationApprovalProgramMismatch
        | Denial::ElevationCloseProgramMismatch
        | Denial::MandatoryReviewProgramMismatch
        | Denial::DelegationActivationRequired
        | Denial::CapabilityRevocationRequired
        | Denial::ApplicationProgramRequired
        | Denial::WorkflowAuthorityRequired => BankHttpDenial::new(
            BankHttpDenialKind::MalformedRequest,
            BankHttpNextAction::CorrectRequest,
        ),
        // A nested stop only records that an inner run stopped: cancellation,
        // a deadline or memory pressure among them. It stays retryable.
        Denial::ExecutionNestedPatternStopped { .. }
        | Denial::ProviderRejected
        | Denial::ActiveSnapshotCapacityExhausted { .. }
        | Denial::RetentionCapacityExhausted
        | Denial::IndexMaintenanceBudgetExceeded => {
            BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry)
        }
        Denial::RetentionIdentityExhausted
        | Denial::SnapshotIdentityExhausted
        | Denial::CandidateIdentityExhausted
        | Denial::IndexGenerationIdentityExhausted
        | Denial::ProgramActivationUnresolved
        | Denial::UniqueIndexUnavailable
        | Denial::ProgramSupportRetired
        | Denial::MutationBindingMismatch
        | Denial::MutationInputMismatch
        | Denial::RecoveryHandoffMismatch { .. } => BankHttpDenial::new(
            BankHttpDenialKind::Unavailable,
            BankHttpNextAction::ContactOperator,
        ),
        Denial::ProgramNotActiveOnOccurrence { .. } => {
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh)
        }
    }
}

// Repeating an unchanged request cannot repair its own limit or a host fault.
// Process
// pressure, busy ownership and control stops keep Retry; request-sized limits
// ask for correction, while configuration and faults ask for the operator.
pub(in crate::http::server) fn execution_resource(denial: Resource) -> BankHttpDenial {
    let (kind, next) = match denial {
        Resource::NativeAllocationRefused {
            kind,
            requested_payload_bytes: _,
        } => return super::super::advancement_denial::physical_allocation(kind),
        Resource::MemoryLimit {
            level: Level::Process,
            ..
        } => (BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry),
        Resource::MemoryLimit {
            level: Level::Policy,
            ..
        }
        | Resource::MemoryLimit {
            level: Level::Declared,
            ..
        }
        | Resource::WorkExhausted
        | Resource::PolicyMemoryLimit
        | Resource::WorkLimit
        | Resource::RetainedBytesExhausted
        | Resource::ScratchCapacityExceeded
        | Resource::ResultCapacityExceeded => (
            BankHttpDenialKind::MalformedRequest,
            BankHttpNextAction::CorrectRequest,
        ),
        // Requests can shrink their memory/work demands. Worker limits are
        // host configuration; only the operator can repair that refusal.
        Resource::NestedAdvancementOpening => (
            BankHttpDenialKind::InternalDenied,
            BankHttpNextAction::ContactOperator,
        ),
        Resource::ForeignAdvancementPhase => (
            BankHttpDenialKind::InternalDenied,
            BankHttpNextAction::ContactOperator,
        ),
        Resource::WorkerLimit
        | Resource::WorkCounterOverflow
        | Resource::CapacityOverflow
        | Resource::ChargedBytesOverflow
        | Resource::NestedLeaseMisuse
        | Resource::NoActiveExecutionScope
        | Resource::EquivalenceContractUnavailable => (
            BankHttpDenialKind::InternalDenied,
            BankHttpNextAction::ContactOperator,
        ),
    };
    BankHttpDenial::new(kind, next)
}

fn operator_denial() -> BankHttpDenial {
    BankHttpDenial::new(
        BankHttpDenialKind::InternalDenied,
        BankHttpNextAction::ContactOperator,
    )
}

#[cfg(test)]
mod tests;
