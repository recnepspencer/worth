//! Exhaustive recovery kind and HTTP instruction table.
use super::*;

/// Every recovery kind with the instruction it must carry. The exhaustive
/// match stops compiling when a kind is added, so a new kind cannot skip this
/// table.
pub(super) fn every_kind() -> Vec<(Kind, Denial, Next)> {
    let listed = |kind: Kind| match kind {
        Kind::DurableClaimPublication
        | Kind::RetainedMaterialUnavailable
        | Kind::OriginalRead(_)
        | Kind::AdmissionAuthorization(_)
        | Kind::CompletionExecutionRequest(_)
        | Kind::DurableStateAdmission(_)
        | Kind::RecoveryNotAdmitted
        | Kind::StaleIncarnation
        | Kind::RecoveryAlreadyMinted
        | Kind::SchemaMismatch
        | Kind::BranchMismatch
        | Kind::ApplicationBindingGenerationMismatch
        | Kind::OperationMismatch
        | Kind::GovernedInputMismatch
        | Kind::ForeignIdempotencyRead
        | Kind::CorrelationMismatch
        | Kind::CompatibilityGenerationMismatch
        | Kind::Expired
        | Kind::AlreadyTerminal
        | Kind::ForeignPrincipal
        | Kind::ForeignRuntime
        | Kind::ForeignBranchEqualOrdinal
        | Kind::AlreadyCompleted
        | Kind::CompletionPublicationPending
        | Kind::CompletionExecutionDenied { .. }
        | Kind::CompletionExecutionControlStopped { .. }
        | Kind::CompletionAllocationDenied { .. }
        | Kind::CompletionStagingCardinalityOverflow
        | Kind::CompletionInputDirectoryAllocationDenied { .. }
        | Kind::TerminalIndexUnavailable
        | Kind::DispatchOutboxMissing
        | Kind::TransportNotInstalled
        | Kind::DispatchOwnerReadDenied(_)
        | Kind::AttemptAdmissionDenied(_)
        | Kind::CanonicalDerivationDenied
        | Kind::TimeObservationDenied
        | Kind::CompensationNotAdmitted
        | Kind::ReconciliationNotAdmitted
        | Kind::FreshAuthorityDenied
        | Kind::AdmissionCancelled
        | Kind::AdmissionDeadlineExceeded
        | Kind::AdmissionAuthenticationExpired
        | Kind::DisclosureAdmissionRequired
        | Kind::ProvenNoEffect(_)
        | Kind::UnresolvedExternalPosture => kind,
    };
    let refused = (Denial::PermissionDenied, Next::None);
    let moved_on = (Denial::Stale, Next::Refresh);
    let unrecoverable = (Denial::Stale, Next::ContactOperator);
    let settling = (Denial::Unavailable, Next::Retry);
    let server_fault = (Denial::InternalDenied, Next::ContactOperator);
    [
        (Kind::DurableClaimPublication, settling),
        (Kind::RetainedMaterialUnavailable, (Denial::Unavailable, Next::ContactOperator)),
        (Kind::OriginalRead(BankEstateIdempotencyResolutionDenial::PreparationBusy), settling),
        (Kind::OriginalRead(BankEstateIdempotencyResolutionDenial::ProviderUnavailable), settling),
        (Kind::OriginalRead(BankEstateIdempotencyResolutionDenial::ForeignAdmission), server_fault),
        (Kind::OriginalRead(BankEstateIdempotencyResolutionDenial::ExecutionDenied(Err(worth_query_host::facade::primary_graph::WorthQueryProviderSessionControlStopKind::Cancelled))), (Denial::Cancelled, Next::Retry)),
        (Kind::AdmissionAuthorization(bank_server::BankAuthorizationDenialKind::InputNotAdmitted), server_fault),
        (Kind::AdmissionAuthorization(bank_server::BankAuthorizationDenialKind::Cancelled), (Denial::Cancelled, Next::Retry)),
        (Kind::AdmissionAuthorization(bank_server::BankAuthorizationDenialKind::DeadlineExceeded), (Denial::DeadlineExceeded, Next::Retry)),
        (Kind::AdmissionAuthorization(bank_server::BankAuthorizationDenialKind::ExpiredAuthentication), (Denial::Unauthenticated, Next::Authenticate)),
        (Kind::AdmissionAuthorization(bank_server::BankAuthorizationDenialKind::PermissionDenied), (Denial::PermissionDenied, Next::None)),
        (Kind::DurableStateAdmission(worth_query_host::facade::product::WorthQueryDurableStateAdmissionDenial::PolicyUnavailable), server_fault),
        (Kind::DurableStateAdmission(worth_query_host::facade::product::WorthQueryDurableStateAdmissionDenial::UndeclaredSupplierCapabilities), server_fault),
        (Kind::DurableStateAdmission(worth_query_host::facade::product::WorthQueryDurableStateAdmissionDenial::Unsupported {
            family: worth_foundational::facade::DurableStateFamily::RecoveryHandleCustody,
            supported_from_phase: 8,
        }), (Denial::Unavailable, Next::ContactOperator)),
        (Kind::RecoveryNotAdmitted, refused),
        (Kind::CompensationNotAdmitted, refused),
        (Kind::ReconciliationNotAdmitted, refused),
        (Kind::ForeignPrincipal, refused),
        (Kind::RecoveryAlreadyMinted, moved_on),
        (Kind::StaleIncarnation, moved_on),
        (Kind::AlreadyTerminal, moved_on),
        (Kind::Expired, unrecoverable),
        (Kind::SchemaMismatch, unrecoverable),
        (Kind::BranchMismatch, unrecoverable),
        (Kind::ForeignBranchEqualOrdinal, unrecoverable),
        (Kind::ApplicationBindingGenerationMismatch, unrecoverable),
        (Kind::OperationMismatch, unrecoverable),
        (Kind::GovernedInputMismatch, unrecoverable),
        (Kind::CompatibilityGenerationMismatch, unrecoverable),
        (Kind::AdmissionCancelled, (Denial::Cancelled, Next::Retry)),
        (
            Kind::AdmissionDeadlineExceeded,
            (Denial::DeadlineExceeded, Next::Retry),
        ),
        (
            Kind::AdmissionAuthenticationExpired,
            (Denial::Unauthenticated, Next::Authenticate),
        ),
        (Kind::UnresolvedExternalPosture, settling),
        (Kind::ProvenNoEffect(worth_foundational::facade::ExternalEffectNoEffectCause::PayloadRejected), (Denial::Unavailable, Next::ContactOperator)),
        (Kind::ProvenNoEffect(worth_foundational::facade::ExternalEffectNoEffectCause::ContactNotStarted), settling),
        (Kind::CompletionPublicationPending, settling),
        (Kind::CompletionExecutionDenied {
            stage: worth_query_host::facade::primary_graph::WorthQueryApplicationCommitDenialStage::ProviderCommit,
            kind: worth_query_host::facade::installed::provider_session::WorthQueryProviderSessionDenialKind::ExecutionWorkerPanicked { partition_identity: Some(1) },
        }, settling),
        (Kind::CompletionExecutionControlStopped {
            stage: worth_query_host::facade::primary_graph::WorthQueryApplicationCommitDenialStage::ProviderCommit,
            kind: worth_query_host::facade::installed::provider_session::WorthQueryProviderSessionControlStopKind::Cancelled,
        }, settling),
        (Kind::CompletionAllocationDenied {
            stage: worth_query_host::facade::primary_graph::WorthQueryApplicationCommitDenialStage::ProviderCommit,
            kind: worth_query_host::facade::runtime::ExecutionAllocationDenialClass::Physical(worth_query_host::facade::runtime::ExecutionPhysicalAllocationDenialKind::Allocator),
            requested_payload_bytes: Some(17),
        }, settling),
        (Kind::CompletionExecutionDenied { stage: worth_query_host::facade::primary_graph::WorthQueryApplicationCommitDenialStage::ProviderCommit, kind: worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind::ContentInconsistency }, server_fault),
        (Kind::CompletionExecutionDenied { stage: worth_query_host::facade::primary_graph::WorthQueryApplicationCommitDenialStage::ProviderCommit, kind: worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind::Content(worth_query_host::facade::primary_graph::RelationalContentRejection::SizeLimit { observed: 9, maximum: 3 }) }, (Denial::ResourceExhausted, Next::NarrowRequest)),
        (Kind::CompletionStagingCardinalityOverflow, settling),
        (Kind::CompletionInputDirectoryAllocationDenied { requested_batches: 3 }, settling),
        (Kind::TerminalIndexUnavailable, settling),
        (Kind::TimeObservationDenied, settling),
        (
            Kind::TransportNotInstalled,
            (Denial::Unavailable, Next::ContactOperator),
        ),
        (Kind::DispatchOutboxMissing, (Denial::NotFound, Next::None)),
        (Kind::ForeignRuntime, server_fault),
        (Kind::ForeignIdempotencyRead, server_fault),
        (Kind::CorrelationMismatch, server_fault),
        (Kind::FreshAuthorityDenied, server_fault),
        (Kind::DisclosureAdmissionRequired, server_fault),
        (Kind::AlreadyCompleted, server_fault),
        (Kind::CanonicalDerivationDenied, server_fault),
    ]
    .into_iter()
    .map(|(kind, (denial, next))| (listed(kind), denial, next))
    .chain(
        OWNER_READS.into_iter().map(|(read, denial, next)| {
            (listed(Kind::DispatchOwnerReadDenied(read)), denial, next)
        }),
    )
    .chain(ATTEMPTS.into_iter().map(|(attempt, denial, next)| {
        (listed(Kind::AttemptAdmissionDenied(attempt)), denial, next)
    }))
    .collect()
}
