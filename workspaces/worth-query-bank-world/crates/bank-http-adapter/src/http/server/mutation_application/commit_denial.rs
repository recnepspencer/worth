//! Bank's action for each typed application commit refusal.
use super::{BankHttpDenial, BankHttpDenialKind, BankHttpMutationFailureKind, BankHttpNextAction};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel as Level,
    },
    primary_graph::WorthQueryApplicationCommitDenialKind,
};

pub(in crate::http::server) fn commit_denial(
    kind: WorthQueryApplicationCommitDenialKind,
) -> (BankHttpMutationFailureKind, BankHttpDenial) {
    use WorthQueryApplicationCommitDenialKind as Denial;
    match kind {
        Denial::ExecutionResource { denial, .. } => execution_resource(denial),
        Denial::ExecutionWorkerPanicked { .. } => operator_denial(),
        Denial::ExecutionUncheckedCustomKernel { .. } => operator_denial(),
        Denial::ExecutionIdentitiesNotCanonical { .. } => operator_denial(),
        Denial::ProductBasisStale => (
            BankHttpMutationFailureKind::ProductStale,
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh),
        ),
        Denial::CustomInvariantDenied => (
            BankHttpMutationFailureKind::InvariantViolated,
            BankHttpDenial::new(
                BankHttpDenialKind::MalformedRequest,
                BankHttpNextAction::CorrectRequest,
            ),
        ),
        Denial::IdempotencyIntentDrift => (
            BankHttpMutationFailureKind::Aborted,
            BankHttpDenial::new(
                BankHttpDenialKind::Stale,
                BankHttpNextAction::CorrectRequest,
            ),
        ),
        // The original request applied; only its answer left the window. A
        // dedicated kind keeps clients from resubmitting it under a new key.
        Denial::IdempotencyWindowExpired => (
            BankHttpMutationFailureKind::IdempotencyWindowExpired,
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh),
        ),
        // The key's earlier commit took effect; reading current state shows it.
        Denial::IdempotencyReceiptNotRetained { .. } => (
            BankHttpMutationFailureKind::Stale,
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh),
        ),
        Denial::IdempotencyIntentUnverifiable => (
            BankHttpMutationFailureKind::Aborted,
            BankHttpDenial::new(
                BankHttpDenialKind::InternalDenied,
                BankHttpNextAction::ContactOperator,
            ),
        ),
        Denial::UniqueValueTaken
        | Denial::CandidateValidatorWorkExceeded { .. }
        | Denial::WorkflowSettlementDenied { .. }
        | Denial::PreparedRootBudgetExhausted { .. }
        | Denial::ElevationTransitionRequired
        | Denial::ElevationRequestProgramMismatch
        | Denial::ElevationApprovalProgramMismatch
        | Denial::ElevationCloseProgramMismatch
        | Denial::MandatoryReviewProgramMismatch
        | Denial::DelegationActivationRequired
        | Denial::CapabilityRevocationRequired
        | Denial::ApplicationProgramRequired
        | Denial::WorkflowAuthorityRequired => (
            BankHttpMutationFailureKind::Aborted,
            BankHttpDenial::new(
                BankHttpDenialKind::MalformedRequest,
                BankHttpNextAction::CorrectRequest,
            ),
        ),
        // A nested stop only records that an inner run stopped: cancellation,
        // a deadline or memory pressure among them. It stays retryable.
        Denial::ExecutionNestedPatternStopped { .. }
        | Denial::ProviderRejected
        | Denial::ActiveSnapshotCapacityExhausted { .. }
        | Denial::RetentionCapacityExhausted
        | Denial::IndexMaintenanceBudgetExceeded => (
            BankHttpMutationFailureKind::Aborted,
            BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry),
        ),
        Denial::RetentionIdentityExhausted
        | Denial::SnapshotIdentityExhausted
        | Denial::CandidateIdentityExhausted
        | Denial::IndexGenerationIdentityExhausted
        | Denial::ProgramActivationUnresolved
        | Denial::UniqueIndexUnavailable
        | Denial::ProgramSupportRetired
        | Denial::MutationBindingMismatch
        | Denial::MutationInputMismatch => (
            BankHttpMutationFailureKind::Aborted,
            BankHttpDenial::new(
                BankHttpDenialKind::Unavailable,
                BankHttpNextAction::ContactOperator,
            ),
        ),
        Denial::ProgramNotActiveOnOccurrence { .. } => (
            BankHttpMutationFailureKind::ProductStale,
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh),
        ),
    }
}

// Deliberate exception to HEAD's Unavailable/Retry projection: repeating an
// unchanged request cannot repair its own limit or a host fault. Process
// pressure, busy ownership and control stops keep Retry; request-sized limits
// ask for correction, while configuration and faults ask for the operator.
pub(super) fn execution_resource(
    denial: Resource,
) -> (BankHttpMutationFailureKind, BankHttpDenial) {
    let (kind, next) = match denial {
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
        Resource::WorkerLimit
        | Resource::WorkCounterOverflow
        | Resource::CapacityOverflow
        | Resource::ChargedBytesOverflow
        | Resource::NestedLeaseMisuse
        | Resource::EquivalenceContractUnavailable => (
            BankHttpDenialKind::InternalDenied,
            BankHttpNextAction::ContactOperator,
        ),
    };
    (
        BankHttpMutationFailureKind::Aborted,
        BankHttpDenial::new(kind, next),
    )
}

fn operator_denial() -> (BankHttpMutationFailureKind, BankHttpDenial) {
    (
        BankHttpMutationFailureKind::Aborted,
        BankHttpDenial::new(
            BankHttpDenialKind::InternalDenied,
            BankHttpNextAction::ContactOperator,
        ),
    )
}

#[cfg(test)]
mod tests;
