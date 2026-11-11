use worth_proof::ProofOutcome;

use super::PreparedPhysicalMutation;
use crate::physical_runtime::{
    CompletedPhysicalMutation, IndeterminatePhysicalMutation, ProvenNoEffectPhysicalMutation,
    RecordAppendDenial, RecordStreamFailure,
};

pub enum PhysicalMutationPreparationSuccess {
    Prepared(PreparedPhysicalMutation),
    Completed(CompletedPhysicalMutation),
    ProvenNoEffect(ProvenNoEffectPhysicalMutation),
    Indeterminate(IndeterminatePhysicalMutation),
}

pub type PhysicalMutationPreparationOutcome = ProofOutcome<
    PhysicalMutationPreparationSuccess,
    PhysicalMutationPreparationDenial,
    PhysicalMutationPreparationDeferred,
    PhysicalMutationPreparationStale,
    PhysicalMutationPreparationRebindRequired,
    PhysicalMutationPreparationFailure,
>;

/// A preparation that issued no successful result. Record-door failures carry
/// only these outcomes, never a prepared mutation or a terminal success.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalMutationPreparationRefusal {
    Denied(PhysicalMutationPreparationDenial),
    Deferred(PhysicalMutationPreparationDeferred),
    Stale(PhysicalMutationPreparationStale),
    RebindRequired(PhysicalMutationPreparationRebindRequired),
    Failed(PhysicalMutationPreparationFailure),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalMutationPreparationDenial {
    RecordAppend(RecordAppendDenial),
    IdempotencyConflict,
    IdempotencyExpired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalMutationPreparationDeferred {
    PreparedRecordSlots { required_records: u32 },
    PreparedPayloadBytes { required_bytes: u64 },
    PendingUnresolvedLimitReached,
    LiveBindingLimitReached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalMutationPreparationStale {
    PublicationAuthorityReleased,
    DurabilityAuthorityReleased,
    WorkOwnerReleased,
    LifecycleGenerationAdvanced,
    AdmissionStopped,
    SignalOwnerUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalMutationPreparationRebindRequired {
    ForeignStore,
    ForeignRuntime,
    ForeignDurabilityPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalMutationPreparationFailure {
    Stream(RecordStreamFailure),
    CanonicalRequestRejected,
    OperationIdentityExhausted,
}
