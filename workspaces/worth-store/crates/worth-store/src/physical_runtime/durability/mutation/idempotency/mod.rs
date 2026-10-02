mod attempt_binding;
mod binding_compaction;
mod bootstrap;
mod canonical_encoding;
mod fate;
mod key;
mod lease;
mod original_drop_no_effect;
mod persisted_binding;
mod registry;
mod runtime_owner;
#[cfg(test)]
mod test_support;

#[cfg(all(
    test,
    feature = "recovery-runtime-owner",
    feature = "certification-test-authority"
))]
pub(in crate::physical_runtime) use binding_compaction::with_conflicting_binding_history;

pub(in crate::physical_runtime) use attempt_binding::{
    AllocatedPhysicalMutationAttemptBinding, UnallocatedPhysicalMutationAttemptBinding,
};
pub use binding_compaction::PhysicalMutationBindingCompaction;
#[cfg(feature = "recovery-runtime-owner")]
pub(in crate::physical_runtime) use binding_compaction::{
    DecodedPhysicalMutationBindingRecord, PhysicalBindingCompactionRecordDecodeDenial,
};
pub use bootstrap::PhysicalIdempotencyReopenFailure;
pub(in crate::physical_runtime) use bootstrap::{
    rebuild_idempotency, RebuiltPhysicalMutationIdempotency,
};
#[cfg(feature = "recovery-runtime-owner")]
pub(in crate::physical_runtime) use fate::PersistedPhysicalMutationFate;
pub use key::{
    PhysicalMutationIdempotencyKey, PhysicalMutationIdempotencyKeyIdentity,
    PhysicalMutationIdempotencyMaterial,
};
pub use lease::{PhysicalMutationIdempotencyLease, PhysicalNamespaceDurableCheckpointGeneration};
pub(in crate::physical_runtime) use original_drop_no_effect::{
    PhysicalOriginalDropCompleted, PhysicalOriginalDropNoEffect,
    PhysicalRecoveredOriginalDropNoDurableEffect,
};
pub(in crate::physical_runtime) use persisted_binding::PersistedPhysicalMutationAttemptBinding;
pub(in crate::physical_runtime) use persisted_binding::PhysicalBindingDecodingContext;
#[cfg(feature = "recovery-runtime-owner")]
pub(in crate::physical_runtime) use persisted_binding::PhysicalPersistedBindingDecodeDenial;
pub(in crate::physical_runtime::durability) use registry::PhysicalMutationIdempotencyRegistry;
pub(in crate::physical_runtime) use registry::{
    PhysicalMutationGroupSealingBinding, PhysicalMutationIdempotencyGroupSealDenial,
    PhysicalMutationIdempotencyRegistryAdmission,
    PhysicalMutationIdempotencyRegistryAdmissionError, PhysicalMutationIdempotencyRegistryDenial,
    PhysicalMutationPreSealCancellationDenial, PhysicalMutationTerminalizationDenial,
    PhysicalMutationUnresolvedBindingObservation,
};
pub use runtime_owner::PhysicalMutationIdempotencyIssuanceDenial;
pub(in crate::physical_runtime) use runtime_owner::{
    PhysicalMutationBindingCompactionCutover, PhysicalMutationBindingCompactionRuntimeAuthority,
    PhysicalMutationIdempotencyRuntimeAuthority, PhysicalMutationIdempotencyRuntimeOwner,
};
