//! Preserve request stops while refusing incomplete original-owner evidence.
use super::{idempotency_read_stop, WorthQueryProviderIdempotencyResolutionDenial as Denial};
use worth_relational::facade::durability::{
    RelationalAssociatedEffectRefusal as Native, RelationalRecoveryStop,
};

pub(super) fn native(cause: Native) -> Denial {
    match cause {
        Native::Recovery(RelationalRecoveryStop::Request(cause)) => idempotency_read_stop(cause),
        Native::Recovery(RelationalRecoveryStop::Allocation(cause)) => allocation(cause),
        Native::OwnerUnavailable
        | Native::Recovery(RelationalRecoveryStop::Durability(_))
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
        Component::Source(RelationalRecoveryStop::Allocation(cause)) => allocation(cause),
        Component::Basis(_)
        | Component::Frame(_)
        | Component::Source(RelationalRecoveryStop::Durability(_))
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
        RuntimeWorldServiceDenial, WorldReadmissionDenial, WorldReadmittedPublicationDenial,
    };
    if let RuntimeWorldServiceDenial::Denied(WorldReadmittedPublicationDenial::Request(cause)) =
        &cause
    {
        return idempotency_read_stop((*cause).into());
    }
    if let RuntimeWorldServiceDenial::Denied(WorldReadmittedPublicationDenial::Source(cause)) =
        cause
    {
        if let WorldReadmissionDenial::Medium(cause) = *cause {
            return medium(cause);
        }
    }
    Denial::Unavailable
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
        Medium::Frame(_)
        | Medium::SignalDescription(_)
        | Medium::BridgeDescription(_)
        | Medium::Unavailable
        | Medium::PublicationBusy
        | Medium::PublicationUnresolved
        | Medium::StaleCheckpoint
        | Medium::CheckpointSelectionRequired
        | Medium::Storage(_) => Denial::Unavailable,
    }
}
