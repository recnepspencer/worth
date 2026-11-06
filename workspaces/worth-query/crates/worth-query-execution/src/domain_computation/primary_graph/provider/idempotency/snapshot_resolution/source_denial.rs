//! Preserve request stops while refusing incomplete original-owner evidence.
use super::{idempotency_read_stop, WorthQueryProviderIdempotencyResolutionDenial as Denial};
use worth_relational::facade::durability::{
    RelationalAssociatedEffectRefusal as Native, RelationalRecoveryStop,
};

pub(super) fn native(cause: Native) -> Denial {
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

pub(super) fn world(
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
        Source::NativeCustody { cause, custody: _ } => world_source(*cause),
        Source::SignalInitialRoot(cause) => signal(cause),
        Source::BridgeBaseline(cause) => bridge(cause),
        Source::NativeFreshSource(RelationalRecoveryStop::Request(cause)) => {
            idempotency_read_stop(cause)
        }
        Source::NativeFreshSource(RelationalRecoveryStop::Content(cause)) => content(cause),
        Source::NativeFreshSource(RelationalRecoveryStop::Allocation(cause)) => allocation(cause),
        Source::NativeFreshSource(RelationalRecoveryStop::Durability(_))
        | Source::NativeFreshSource(RelationalRecoveryStop::AdmissionImage(_))
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
        Medium::SignalDescription(cause) => signal(cause),
        Medium::BridgeDescription(cause) => bridge(cause),
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

fn signal(cause: worth_signal::facade::branch::SignalInitialEmptyRootRefusal) -> Denial {
    use worth_signal::facade::branch::SignalInitialEmptyRootRefusal as Signal;
    match cause {
        Signal::Request(cause) => idempotency_read_stop(cause.into()),
        Signal::Description
        | Signal::UnsupportedVersion { .. }
        | Signal::OwnerUnavailable
        | Signal::InitialRootRequired
        | Signal::RecordedMeaningMismatch
        | Signal::CurrentBasisRequired
        | Signal::OwnerAdmissionRequired
        | Signal::RetentionRequired => Denial::Unavailable,
    }
}
fn bridge(cause: worth_runtime_bridge::facade::RuntimeWorldBaselineRefusal) -> Denial {
    use worth_runtime_bridge::facade::RuntimeWorldBaselineRefusal as Bridge;
    match cause {
        Bridge::Request(cause) => idempotency_read_stop(cause.into()),
        Bridge::Description
        | Bridge::UnsupportedVersion { .. }
        | Bridge::RegisteredSourceRequired
        | Bridge::UninstalledBaselineRequired
        | Bridge::RecordedMeaningMismatch
        | Bridge::SignalInitialRootRequired
        | Bridge::CurrentBasisRequired => Denial::Unavailable,
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
    fn signal_description_cancellation_keeps_the_original_control_stop() {
        assert_eq!(
            source(WorldDurableMediumDenial::SignalDescription(
                worth_signal::facade::branch::SignalInitialEmptyRootRefusal::Request(
                    WorkCeilingDenial::Stopped(KernelStop::Cancelled)
                )
            )),
            idempotency_read_stop(
                worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled
            )
        );
    }
    #[test]
    fn bridge_description_work_exhaustion_keeps_the_original_resource_stop() {
        assert_eq!(source(WorldDurableMediumDenial::BridgeDescription(
            worth_runtime_bridge::facade::RuntimeWorldBaselineRefusal::Request(WorkCeilingDenial::Stopped(KernelStop::WorkCeiling)))),
            idempotency_read_stop(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted));
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
