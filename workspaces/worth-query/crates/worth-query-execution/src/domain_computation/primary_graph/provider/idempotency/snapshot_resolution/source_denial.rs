//! Preserve request stops while refusing incomplete original-owner evidence.
use super::{idempotency_read_stop, WorthQueryProviderIdempotencyResolutionDenial as Denial};
use worth_relational::facade::durability::{
    RelationalAssociatedEffectRefusal as Native, RelationalRecoveryStop,
};

pub(in crate::domain_computation::primary_graph::provider) fn native(cause: Native) -> Denial {
    match cause {
        Native::Recovery(RelationalRecoveryStop::Content(cause)) => content(cause),
        Native::Recovery(RelationalRecoveryStop::Request(cause)) => idempotency_read_stop(cause),
        Native::Recovery(RelationalRecoveryStop::Allocation(cause)) => allocation(cause),
        Native::OwnerUnavailable
        | Native::Recovery(RelationalRecoveryStop::Durability(_))
        | Native::Recovery(RelationalRecoveryStop::AdmissionImage(_))
        | Native::AssociatedMemory { .. }
        | Native::MissingDiscovery { .. }
        | Native::ForeignRuntime
        | Native::CatalogSubjectChanged
        | Native::MissingSettlementEvidence
        | Native::SubjectMismatch => Denial::Unavailable,
    }
}

pub(super) fn component(
    cause: worth_relational::facade::branch::RelationalRecordedComponentRefusal,
) -> Denial {
    use worth_relational::facade::branch::RelationalRecordedComponentRefusal as Component;
    match cause {
        Component::Request(cause) | Component::Source(RelationalRecoveryStop::Request(cause)) => {
            idempotency_read_stop(cause)
        }
        Component::Source(RelationalRecoveryStop::Content(cause)) => content(cause),
        Component::Source(RelationalRecoveryStop::Allocation(cause)) => allocation(cause),
        Component::PayloadBacking(worth_execution::ExecutionRequestArrayDenial::Request(cause)) => {
            idempotency_read_stop(cause.into())
        }
        Component::PayloadBacking(worth_execution::ExecutionRequestArrayDenial::Allocation(
            cause,
        )) => allocation(cause),
        Component::Basis(_)
        | Component::Frame(_)
        | Component::Source(RelationalRecoveryStop::Durability(_))
        | Component::Source(RelationalRecoveryStop::AdmissionImage(_))
        | Component::MissingPerformedCommit { .. }
        | Component::CanonicalMismatch { .. }
        | Component::PayloadEncoding { .. } => Denial::Unavailable,
    }
}

pub(in crate::domain_computation::primary_graph::provider) fn world(
    cause: worth_runtime_world::facade::RuntimeWorldServiceDenial<
        worth_runtime_world::facade::WorldReadmittedPublicationDenial,
    >,
) -> Denial {
    use worth_runtime_world::facade::{
        RuntimeWorldServiceDenial, WorldReadmittedPublicationDenial,
    };
    match cause {
        RuntimeWorldServiceDenial::OwnerUnavailable(_) => Denial::Unavailable,
        RuntimeWorldServiceDenial::Denied(cause) => match cause {
            WorldReadmittedPublicationDenial::Request(cause) => idempotency_read_stop(cause.into()),
            WorldReadmittedPublicationDenial::Source(cause) => world_source(*cause),
            WorldReadmittedPublicationDenial::ForeignHome
            | WorldReadmittedPublicationDenial::OwnerUnavailable
            | WorldReadmittedPublicationDenial::History(_)
            | WorldReadmittedPublicationDenial::NotReadmitted
            | WorldReadmittedPublicationDenial::NativeMismatch => Denial::Unavailable,
        },
    }
}

fn world_source(cause: worth_runtime_world::facade::WorldReadmissionDenial) -> Denial {
    use worth_runtime_world::facade::WorldReadmissionDenial as Source;
    match cause {
        Source::Medium(cause) => medium(cause),
        Source::NativeRecorded(cause) => component(cause),
        Source::NativeDiscovery(cause) => native(cause),
        Source::NativeForkDiscovery(cause) => fork_source(&cause),
        Source::NativeFork(cause) => fork_source(cause.cause()),
        Source::NativeForkRetirement(cause) => match cause {
            worth_relational::facade::branch::RelationalBranchBasisRetirementDenial::Execution(cause) => idempotency_read_stop(cause.into()),
            worth_relational::facade::branch::RelationalBranchBasisRetirementDenial::ForeignOwner
            | worth_relational::facade::branch::RelationalBranchBasisRetirementDenial::Busy => Denial::Unavailable,
        },
        Source::NativeCustody { cause, custody: _ } => world_source(*cause),
        Source::SignalSource(_)
        | Source::SignalDiscovery(_)
        | Source::UncleanRecoveryUnsupported(_)
        | Source::AuthoredSignalRequired => Denial::Unavailable,
        Source::BridgeBaseline(cause) => bridge(cause),
        Source::NativeFreshSource(RelationalRecoveryStop::Request(cause)) => {
            idempotency_read_stop(cause)
        }
        Source::NativeFreshSource(RelationalRecoveryStop::Content(cause)) => content(cause),
        Source::NativeFreshSource(RelationalRecoveryStop::Allocation(cause)) => allocation(cause),
        Source::NativeFreshSource(RelationalRecoveryStop::Durability(_))
        | Source::NativeFreshSource(RelationalRecoveryStop::AdmissionImage(_))
        | Source::AuthoredAttachment(_)
        | Source::JournalSourceMismatch
        | Source::OccurrenceEvidenceMismatch
        | Source::HistoryAdmission(_)
        | Source::HistoryRetention(_)
        | Source::NativeComponent(_)
        | Source::NativePublication(_)
        | Source::NativeSuccessor(_)
        | Source::NativeSettlementRequired
        | Source::OrphanNativeAssociation
        | Source::DuplicateNativeEffect
        | Source::MalformedNativeAssociation
        | Source::DispositionEvidenceMismatch
        | Source::ProgramRevisionMismatch
        | Source::OccurrenceReadmissionRequired
        | Source::RecoveryCapacity
        | Source::ExecutionInstallation(_)
        | Source::IdentityExhaustion(_)
        | Source::WrongJournal
        | Source::ForeignOwner
        | Source::DuplicateIdentity
        | Source::MissingCreationIntent
        | Source::ConflictingCreationIntent
        | Source::CreationEvidenceMismatch
        | Source::MissingOpenAttempt
        | Source::ConflictingOpenAttempt
        | Source::ConflictingDisposition
        | Source::MissingPredecessor
        | Source::BranchLineage
        | Source::FutureIncarnation
        | Source::NativeReadmissionRequired
        | Source::RootPreparation(_)
        | Source::RootAdmission(_)
        | Source::FreshWorldSourceNotEmpty => Denial::Unavailable,
    }
}

fn fork_source(
    cause: &worth_relational::facade::durability::RelationalForkReadmissionCause,
) -> Denial {
    use worth_relational::facade::durability::RelationalForkReadmissionCause as Fork;
    match cause {
        Fork::Request(cause) => idempotency_read_stop(*cause),
        Fork::ForeignRuntime
        | Fork::OwnerUnavailable
        | Fork::MissingOriginal(_)
        | Fork::AssociationMismatch
        | Fork::ArtifactDigestMismatch
        | Fork::NotFork
        | Fork::SourceMismatch
        | Fork::TargetMismatch
        | Fork::Component(_) => Denial::Unavailable,
    }
}

fn allocation(cause: worth_execution::ExecutionAllocationDenial) -> Denial {
    match cause.classify() {
        worth_execution::ExecutionAllocationDenialClass::Request(cause) => {
            idempotency_read_stop(cause.into())
        }
        worth_execution::ExecutionAllocationDenialClass::Physical(_) => Denial::ExecutionDenied(
            Ok(crate::domain_computation::WorthQueryProviderSessionDenialKind::AllocationDenied),
        ),
    }
}

fn medium(cause: worth_runtime_world::facade::WorldDurableMediumDenial) -> Denial {
    use worth_execution::{
        ExecutionOrderedMapDenial as Ordered, ExecutionRequestArrayDenial as Backing,
    };
    use worth_runtime_world::facade::WorldDurableMediumDenial as Medium;
    match cause {
        Medium::Request(cause)
        | Medium::Backing(Backing::Request(cause))
        | Medium::Ordered(Ordered::Request(cause)) => idempotency_read_stop(cause.into()),
        Medium::Allocation(cause)
        | Medium::Backing(Backing::Allocation(cause))
        | Medium::Ordered(Ordered::Allocation(cause)) => allocation(cause),
        Medium::Ordered(Ordered::CapacityOverflow) => idempotency_read_stop(
            worth_relational::facade::transactions::RelationalExecutionDenialCause::MemoryOverflow,
        ),
        Medium::NativeDescription(cause) => component(*cause),
        Medium::NameAllocation
        | Medium::OccurrenceReopenRequired
        | Medium::Frame(_)
        | Medium::Unavailable
        | Medium::PublicationBusy
        | Medium::PublicationUnresolved
        | Medium::StaleCheckpoint
        | Medium::CheckpointOriginRetention
        | Medium::CheckpointSelectionRequired
        | Medium::Storage(_) => Denial::Unavailable,
    }
}

fn bridge(cause: worth_runtime_bridge::facade::RuntimeWorldBaselineRefusal) -> Denial {
    use worth_runtime_bridge::facade::RuntimeWorldBaselineRefusal as Bridge;
    match cause {
        Bridge::Request(cause) => idempotency_read_stop(cause.into()),
        Bridge::Description
        | Bridge::UnsupportedVersion { .. }
        | Bridge::RegisteredSourceRequired
        | Bridge::BaselineCorrespondenceRequired
        | Bridge::RecordedMeaningMismatch
        | Bridge::SignalGraphBasisRequired
        | Bridge::CurrentBasisRequired => Denial::Unavailable,
    }
}

fn content(cause: worth_relational::facade::content::RelationalContentDenial) -> Denial {
    use worth_relational::facade::content::RelationalContentDenial as Content;
    match cause {
        Content::Execution(cause) => idempotency_read_stop(cause),
        Content::MissingContent
        | Content::BackingUnavailable { .. }
        | Content::FormatBindingMissing
        | Content::FormatMismatch
        | Content::UnsupportedVersion
        | Content::InvalidFormat
        | Content::NonCanonicalContent
        | Content::IntegrityMismatch
        | Content::ContentIdentityConflict
        | Content::EncodingInconsistency
        | Content::InputTypeMismatch
        | Content::DuplicateContentFormat
        | Content::SizeLimit { .. }
        | Content::ArithmeticOverflow
        | Content::HistoryNotResident
        | Content::UnreferencedAttachment { .. } => Denial::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_execution::{MapKernelStop as KernelStop, WorkCeilingDenial};
    use worth_runtime_world::facade::{
        RuntimeWorldServiceDenial, WorldDurableMediumDenial, WorldReadmissionDenial,
        WorldReadmittedPublicationDenial,
    };
    fn source(cause: WorldDurableMediumDenial) -> Denial {
        world(RuntimeWorldServiceDenial::Denied(
            WorldReadmittedPublicationDenial::Source(Box::new(WorldReadmissionDenial::Medium(
                cause,
            ))),
        ))
    }
    #[test]
    fn component_decoder_cancellation_keeps_the_original_control_stop() {
        assert_eq!(
            source(WorldDurableMediumDenial::Request(
                WorkCeilingDenial::Stopped(KernelStop::Cancelled),
            )),
            idempotency_read_stop(
                worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled,
            ),
        );
    }
    #[test]
    fn bridge_baseline_work_exhaustion_keeps_the_original_resource_stop() {
        assert_eq!(
            world_source(WorldReadmissionDenial::BridgeBaseline(
                worth_runtime_bridge::facade::RuntimeWorldBaselineRefusal::Request(
                    WorkCeilingDenial::Stopped(KernelStop::WorkCeiling),
                ),
            )),
            idempotency_read_stop(
                worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted,
            ),
        );
    }
    #[test]
    fn native_fork_discovery_cancellation_keeps_the_original_control_stop() {
        let cancelled =
            worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled;
        assert_eq!(
            world_source(WorldReadmissionDenial::NativeForkDiscovery(
                worth_relational::facade::durability::RelationalForkReadmissionCause::Request(
                    cancelled
                )
            )),
            idempotency_read_stop(cancelled)
        );
    }
    #[test]
    fn native_fork_retirement_exhaustion_keeps_the_original_resource_stop() {
        assert_eq!(world_source(WorldReadmissionDenial::NativeForkRetirement(
            worth_relational::facade::branch::RelationalBranchBasisRetirementDenial::Execution(
                WorkCeilingDenial::Stopped(KernelStop::WorkCeiling)))),
            idempotency_read_stop(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted));
    }
}
