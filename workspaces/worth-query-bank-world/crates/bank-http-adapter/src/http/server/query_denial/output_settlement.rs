//! Original output-recovery causes keep their owner-specific next action.
use super::*;
use crate::http::server::{
    advancement_denial, mutation_application, relational_execution_denial, world_denial,
};
use worth_query_host::facade::primary_graph::{
    BoundedEntityFieldLookupDenialKind, IndexedSelectionReobserveDenial, RebaseVerificationReason,
    WorthQueryApplicationIdempotencyResolutionDenialKind,
    WorthQueryOutputSettlementProvenanceDenial,
};
pub(super) fn output_settlement(kind: BankApplicationOutputSettlementDenialKind) -> BankHttpDenial {
    use BankApplicationOutputSettlementDenialKind as Settlement;
    match kind {
        Settlement::ObligationPublication(cause) => obligation_publication(cause),
        Settlement::WorldRecovery(cause) => world_denial::recovery(cause),
        Settlement::RecoveryPublication(cause) => world_denial::publication(cause),
        Settlement::PublicationReceipt(cause) => original_receipt(cause),
        Settlement::DurableStateAdmission(cause) => mutation_application::request_mutation_denial(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenialKind::DurableStateAdmission(cause),
        ),
        Settlement::CurrentControlBusy => unavailable(),
        Settlement::PredecessorRead(cause) => predecessor_read::denial(*cause),
        Settlement::CurrentPublication(cause) => publication_read::denial(*cause),
        Settlement::RetainedVerificationFactUnavailable(cause) => verification(*cause),
        Settlement::SettlementProvenance(cause) => match cause {
            WorthQueryOutputSettlementProvenanceDenial::OriginalSettlementMissing
            | WorthQueryOutputSettlementProvenanceDenial::ReceiptSettlementMissing
            | WorthQueryOutputSettlementProvenanceDenial::ForeignSettlementOwner
            | WorthQueryOutputSettlementProvenanceDenial::OriginalDisclosureMissing
            | WorthQueryOutputSettlementProvenanceDenial::ReceiptDisclosureMissing
            | WorthQueryOutputSettlementProvenanceDenial::ForeignDisclosureOwner
            | WorthQueryOutputSettlementProvenanceDenial::PublicationMismatch
            | WorthQueryOutputSettlementProvenanceDenial::ReusedSettlementOwnerMismatch
            | WorthQueryOutputSettlementProvenanceDenial::RestoredCheckpointMismatch => operator(),
        },

        Settlement::ManagedCleanupRequired(_) => {
            BankHttpDenial::new(Kind::ManagedCleanupRequired, Next::ContactOperator)
        }
        Settlement::BridgeConditional(_)
        | Settlement::CorrespondenceDelivery(_)
        | Settlement::ProductDelivery(_) => unavailable(),
        Settlement::ExecutionRequest(cause) => {
            crate::http::server::advancement_denial::advancement(cause)
        }
        Settlement::RelationalExecution(cause) => {
            crate::http::server::relational_execution_denial::denial(cause)
        }
        Settlement::ManagedIndexSourceDenied | Settlement::SourceQueryInstallation(_) => {
            unavailable()
        }
        Settlement::SourcePrincipal(_) => stale(),
        Settlement::SourceScope(kind) => entity(kind),
        Settlement::SourceQueryAdmission(kind) => admission(kind),
        Settlement::SourceQueryExecution(kind) => execution(kind),
        Settlement::RequestAuthorization(kind) => authorization_denial(kind),
        Settlement::Cancelled => cancelled(),
        Settlement::TimedOut => deadline(),
        Settlement::ProducerDomainDenied | Settlement::DiscoveryConnection(_) => malformed(),
        Settlement::ConflictingIntent { .. } | Settlement::AuthoredStageRefused { .. } => operator(),
        Settlement::Superseded
        | Settlement::PublicationStale
        | Settlement::ForeignSource
        | Settlement::ForeignDemand
        | Settlement::ForeignSettlement
        | Settlement::RetainedBasisUnavailable
        | Settlement::Closed => stale(),
        Settlement::ProducerRouteRoundBudgetExceeded { .. }
        | Settlement::ProducerAttemptRoundBudgetExceeded { .. }
        | Settlement::WorkBudgetExceeded
        | Settlement::RetentionBudgetExceeded
        | Settlement::PublicationCapacityExceeded => exhausted(),
        Settlement::MissingApplicableProducer
        | Settlement::AmbiguousApplicableProducer
        | Settlement::ProducerUnavailable
        | Settlement::SchedulingRejected
        | Settlement::SchedulingDeferred
        | Settlement::NoEffect => unavailable(),
        Settlement::ProductSelection(kind) => product_selection(kind),
        Settlement::CyclicRequiredOutputOccurrences { .. }
        | Settlement::RequiredOutputParentUnavailable { .. }
        | Settlement::RequiredSuccessorStateUnavailable
        | Settlement::DuplicatePerformedSource
        | Settlement::IncompleteDependencyCoverage
        | Settlement::ManagedSourceExpansionUnsupported
        | Settlement::UnobservedWrite(_) => internal_denied(),
    }
}

fn obligation_publication(
    cause: worth_query_host::facade::primary_graph::QueryObligationDenial,
) -> BankHttpDenial {
    use worth_query_host::facade::primary_graph::QueryObligationDenial as D;
    match cause {
        D::Execution(cause) => advancement_denial::request(cause),
        D::ConditionalCleanupPending { .. }
        | D::ManagedRunPending { .. }
        | D::OwnedAsyncPending { .. }
        | D::OwnedAsyncQueuePending { .. }
        | D::PublicationPending
        | D::PublicationNoEffect
        | D::PublicationUnresolved
        | D::Storage(_) => unavailable(),
        D::StaleIncarnation => stale(),
        D::UnsupportedVersion(_)
        | D::MalformedFrame
        | D::Checksum
        | D::ForeignHome
        | D::PrefixMismatch
        | D::IdentityExhausted
        | D::DuplicateProgram
        | D::UnrosteredProgram(_)
        | D::StatusMismatch
        | D::IllegalStatusTransition
        | D::ObligationIdentityMismatch
        | D::ObligationSubjectMismatch
        | D::SupportUnavailable
        | D::UnknownObligation
        | D::DuplicateObligation
        | D::AdoptionReadmissionRequired
        | D::CleanupReleaseUnrecorded { .. }
        | D::OwnedAsyncMandatory { .. }
        | D::CleanupInconsistent { .. }
        | D::MissingOriginalInterpretationSource { .. }
        | D::OriginalInterpretationProgramMismatch { .. }
        | D::OwnerUnavailable
        | D::PublicationGenerationMismatch => operator(),
    }
}

fn original_receipt(cause: WorthQueryApplicationIdempotencyResolutionDenialKind) -> BankHttpDenial {
    use WorthQueryApplicationIdempotencyResolutionDenialKind as D;
    match cause {
        D::OriginalEffectReleased => {
            BankHttpDenial::new(Kind::OriginalEffectReleased, Next::ContactOperator)
        }
        D::ExecutionDenied(cause) => mutation_application::pending_execution_stop(cause),
        D::Authorization(cause) => authorization_denial(cause.into()),
        D::PreparationBusy
        | D::ActiveSnapshotCapacityExhausted { .. }
        | D::RetentionCapacityExhausted
        | D::ProviderUnavailable => unavailable(),
        D::RetentionIdentityExhausted | D::SnapshotIdentityExhausted => {
            BankHttpDenial::new(Kind::Unavailable, Next::ContactOperator)
        }
        D::ForeignAdmission | D::RecordedIntentUnverifiable => operator(),
    }
}

fn verification(cause: RebaseVerificationReason) -> BankHttpDenial {
    use worth_query_host::facade::runtime::ExecutionAllocationDenialClass as Allocation;
    use RebaseVerificationReason as D;
    match cause {
        D::ManagedIndex(cause) => managed_index::work(&cause),
        D::Execution(cause) => relational_execution_denial::denial(cause),
        D::AdmissionDenied(cause) => managed_index::native_admission(&cause),
        D::NativePayloadStorage {
            class: Allocation::Request(cause),
            ..
        } => advancement_denial::request(cause),
        D::NativePayloadStorage {
            class: Allocation::Physical(cause),
            ..
        } => advancement_denial::physical_allocation(cause),
        D::IndexedSelectionDenied(cause) | D::IndexedSelectionFactDenied(cause) => indexed(cause),
        D::NativeRevisionUnavailable | D::NativeFactRevisionUnavailable => {
            BankHttpDenial::new(Kind::Unavailable, Next::ContactOperator)
        }
        D::UnsupportedDecisionFact => operator(),
    }
}

fn indexed(cause: IndexedSelectionReobserveDenial) -> BankHttpDenial {
    use IndexedSelectionReobserveDenial as D;
    match cause {
        D::WorkBudgetExceeded => exhausted(),
        D::UnexpectedFact | D::Overflow => operator(),
        D::Lookup(cause) => match cause {
            BoundedEntityFieldLookupDenialKind::Execution(cause) => {
                relational_execution_denial::denial(cause)
            }
            BoundedEntityFieldLookupDenialKind::SnapshotUnavailable
            | BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable => unavailable(),
            BoundedEntityFieldLookupDenialKind::InvalidCandidateLimit
            | BoundedEntityFieldLookupDenialKind::IndexNotInstalled
            | BoundedEntityFieldLookupDenialKind::WrongIndexKind
            | BoundedEntityFieldLookupDenialKind::CorruptIndexEntries
            | BoundedEntityFieldLookupDenialKind::StorageParityMismatch => operator(),
        },
    }
}
fn operator() -> BankHttpDenial {
    BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator)
}
