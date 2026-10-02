//! Real native-pool backing; inline evidence is local merge input, not custody.

use std::num::{NonZeroU32, NonZeroU64};

use worth_proof::TransitionOutcome;
use worth_store_physical_backend::FilesystemAccessPosture;
use worth_store_physical_format::store_namespace::StableStoreIdentity;

use super::{Fate, StoreRecoveryBindingFreshness, StoreRecoveryOperationEvidence};
use crate::physical_runtime::{
    durability::{
        PhysicalMutationDurabilityRequest, PhysicalMutationFingerprintInput,
        PhysicalMutationOperationFamily, PhysicalMutationPayloadDigest,
        PhysicalMutationRequestScope, PhysicalMutationSecurityBasis,
    },
    instance::PhysicalResidencyOwner,
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy, CheckpointMemoryLimit,
    FilesystemMediaAdmission, GroupCommitDelay, GroupCommitLimit, IdempotencyRetentionGenerations,
    LifecycleGeneration, LiveIdempotencyBindingLimit, MediaOwnedPhysicalRuntime,
    PendingUnresolvedMutationLimit, PhysicalCheckpointPolicy, PhysicalDurabilityDeclaration,
    PhysicalDurabilityPolicyIdentity, PhysicalIdempotencyPolicy, PhysicalMutationIdentity,
    PhysicalMutationRequestFingerprint, PhysicalOperationIdentity, PhysicalRuntimeAdmission,
    PhysicalStore, PhysicalWalPolicy, PhysicalWorkGeneration, PhysicalWorkIdentity,
    RetainedWalTailLimit, RuntimeIdentity, WalSegmentByteLimit, WalSegmentInventoryLimit,
};

pub(super) struct Fixture {
    _root: tempfile::TempDir,
    pub(super) media: MediaOwnedPhysicalRuntime,
    store: StableStoreIdentity,
    runtime: RuntimeIdentity,
    generation: LifecycleGeneration,
    policy: PhysicalDurabilityPolicyIdentity,
}

pub(super) fn world() -> (Fixture, PhysicalResidencyOwner) {
    let root = tempfile::tempdir().unwrap();
    let runtime =
        PhysicalStore::admit(PhysicalRuntimeAdmission::new(root.path()).unwrap()).unwrap();
    let media = match runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("native index fixture must admit filesystem media"),
    };
    let store = media.store_identity();
    let runtime = media.runtime_identity();
    let generation = media.observer().snapshot().unwrap().generation();
    let policy = match PhysicalDurabilityDeclaration::builder()
        .group_commit(
            GroupCommitLimit::new(NonZeroU32::new(32).unwrap()),
            GroupCommitDelay::new(NonZeroU64::new(1).unwrap()),
        )
        .wal(PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(NonZeroU64::new(1024).unwrap()),
            WalSegmentInventoryLimit::new(NonZeroU32::new(64).unwrap()),
        ))
        .idempotency(PhysicalIdempotencyPolicy::new(
            IdempotencyRetentionGenerations::new(NonZeroU64::new(4).unwrap()),
            PendingUnresolvedMutationLimit::new(NonZeroU32::new(2).unwrap()),
            LiveIdempotencyBindingLimit::new(NonZeroU32::new(16).unwrap()),
        ))
        .checkpoint(PhysicalCheckpointPolicy::fuzzy(
            CheckpointMemoryLimit::new(NonZeroU64::new(1024).unwrap()),
            RetainedWalTailLimit::new(NonZeroU64::new(4096).unwrap()),
        ))
        .admit(media.physical_durability_admission_basis().unwrap())
        .into_raw()
    {
        TransitionOutcome::Success(policy) => policy.identity(),
        _ => panic!("native index fixture must admit durability policy"),
    };
    let format = AdmittedPhysicalRecordFormat::admit(
        worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let owner = PhysicalResidencyOwner::admit(
        store,
        AdmittedPhysicalRecordResidencyPolicy::canonical(format),
    )
    .unwrap();
    (
        Fixture {
            _root: root,
            media,
            store,
            runtime,
            generation,
            policy,
        },
        owner,
    )
}

pub(super) fn fingerprint(fixture: &Fixture, payload: u8) -> PhysicalMutationRequestFingerprint {
    PhysicalMutationRequestFingerprint::derive(PhysicalMutationFingerprintInput {
        store: fixture.store,
        durability_policy: fixture.policy,
        scope: PhysicalMutationRequestScope::record_append([3; 32]),
        payload: PhysicalMutationPayloadDigest::from_validated_payload([payload; 32]),
        durability_request: PhysicalMutationDurabilityRequest::PlatformDurable,
        operation_family: PhysicalMutationOperationFamily::RecordAppend,
        security_bases: &[PhysicalMutationSecurityBasis::from_admitted_security(
            [5; 32],
        )],
    })
    .unwrap()
}

pub(super) fn evidence(fixture: &Fixture, ordinal: u8) -> StoreRecoveryOperationEvidence {
    let mutation = PhysicalMutationIdentity::from_reserved_operation(
        PhysicalWorkIdentity::from_instance_owner(
            fixture.store,
            fixture.runtime,
            PhysicalWorkGeneration::from_lifecycle(fixture.generation),
            PhysicalOperationIdentity::from_owner_sequence(
                NonZeroU64::new(u64::from(ordinal)).unwrap(),
            ),
        ),
    );
    StoreRecoveryOperationEvidence {
        // The lookup only compares these inline sample keys. They issue no lease
        // or permission; production decoding and native-owner tests own that join.
        idempotency_identity: [ordinal; 32],
        mutation,
        request_fingerprint: fingerprint(fixture, ordinal),
        lease_issuance_generation: 0,
        lease_expiry_generation: 4,
        freshness: StoreRecoveryBindingFreshness::Retained,
        fate: Fate::Indeterminate,
        attempt_binding_identity: None,
    }
}
