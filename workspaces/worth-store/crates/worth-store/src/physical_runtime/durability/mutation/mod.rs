mod admission;
mod handle;
mod idempotency;
mod identity;
mod outcome;
mod progression;
mod request;
mod request_fingerprint;

#[cfg(all(
    test,
    feature = "recovery-runtime-owner",
    feature = "certification-test-authority"
))]
pub(in crate::physical_runtime) use idempotency::with_conflicting_binding_history;

pub(in crate::physical_runtime) use handle::PhysicalMutationAttempt;
pub use handle::PhysicalMutationHandle;
pub use idempotency::{
    PhysicalIdempotencyReopenFailure, PhysicalMutationBindingCompaction,
    PhysicalMutationIdempotencyIssuanceDenial, PhysicalMutationIdempotencyKey,
    PhysicalMutationIdempotencyKeyIdentity, PhysicalMutationIdempotencyLease,
    PhysicalMutationIdempotencyMaterial, PhysicalNamespaceDurableCheckpointGeneration,
};
pub use identity::PhysicalMutationIdentity;
pub(in crate::physical_runtime) use outcome::PhysicalMutationTerminalFact;
pub use outcome::{
    PhysicalMutationCancellationOutcome, PhysicalMutationOutcome, PhysicalMutationPoll,
    PhysicalMutationProgress, PhysicalMutationProgressPhase, PhysicalMutationTerminalObservation,
};
pub use progression::{
    CompletedPhysicalMutation, DataDispatchedPhysicalMutation, DataSettledPhysicalMutation,
    RootNamespaceDurablePhysicalMutationMembers, RootPublicationPhysicalMutationMember,
    RootPublicationPreparedPhysicalMutationMembers, RootReplacedPhysicalMutationMembers,
    WalAppendedPhysicalMutation, WalDurablePhysicalMutation, WalRangeReservedPhysicalMutation,
};
pub(in crate::physical_runtime) use progression::{
    CompletedPhysicalMutationFact, RootPublicationPreparedCore, SettledPhysicalMutationBasis,
    WalRangeReservedPhysicalMutationBasis,
};
pub use request::{PhysicalMutationDeadline, PhysicalMutationRequest};
pub use request_fingerprint::PhysicalMutationRequestFingerprint;

pub(in crate::physical_runtime) use admission::AdmittedPhysicalMutation;
pub(in crate::physical_runtime) use idempotency::PhysicalBindingDecodingContext;
pub(in crate::physical_runtime::durability) use idempotency::PhysicalMutationIdempotencyRegistry;
pub(in crate::physical_runtime) use idempotency::{
    rebuild_idempotency, AllocatedPhysicalMutationAttemptBinding,
    PersistedPhysicalMutationAttemptBinding, PhysicalMutationBindingCompactionCutover,
    PhysicalMutationBindingCompactionRuntimeAuthority, PhysicalMutationGroupSealingBinding,
    PhysicalMutationIdempotencyGroupSealDenial, PhysicalMutationIdempotencyRegistryAdmission,
    PhysicalMutationIdempotencyRegistryAdmissionError, PhysicalMutationIdempotencyRegistryDenial,
    PhysicalMutationIdempotencyRuntimeAuthority, PhysicalMutationIdempotencyRuntimeOwner,
    PhysicalMutationPreSealCancellationDenial, PhysicalMutationTerminalizationDenial,
    PhysicalMutationUnresolvedBindingObservation, PhysicalOriginalDropCompleted,
    PhysicalOriginalDropNoEffect, PhysicalRecoveredOriginalDropNoDurableEffect,
    RebuiltPhysicalMutationIdempotency, UnallocatedPhysicalMutationAttemptBinding,
};
#[cfg(feature = "recovery-runtime-owner")]
pub(in crate::physical_runtime) use idempotency::{
    DecodedPhysicalMutationBindingRecord, PersistedPhysicalMutationFate,
    PhysicalBindingCompactionRecordDecodeDenial, PhysicalPersistedBindingDecodeDenial,
};
pub(in crate::physical_runtime) use idempotency::{
    TerminalHeadNoRetryClaim, TerminalHeadRetryClaimDenial,
};
pub(in crate::physical_runtime) use request::PhysicalMutationDurabilityRequest;
pub(in crate::physical_runtime) use request_fingerprint::{
    PhysicalMutationFingerprintInput, PhysicalMutationOperationFamily,
    PhysicalMutationPayloadDigest, PhysicalMutationRequestScope, PhysicalMutationSecurityBasis,
};
