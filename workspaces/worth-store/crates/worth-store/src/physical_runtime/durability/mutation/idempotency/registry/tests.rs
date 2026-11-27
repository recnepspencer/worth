use std::num::{NonZeroU32, NonZeroU64};

use sha2::{Digest, Sha256};
use worth_proof::TransitionOutcome;
use worth_store_physical_backend::FilesystemAccessPosture;
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};

use super::*;
use crate::physical_runtime::durability::mutation::idempotency::test_support::RegistryFixture;
use crate::physical_runtime::{
    durability::mutation::{
        request::PhysicalMutationDurabilityRequest, PhysicalMutationFingerprintInput,
        PhysicalMutationOperationFamily, PhysicalMutationPayloadDigest,
        PhysicalMutationRequestScope, PhysicalMutationSecurityBasis,
    },
    CheckpointMemoryLimit, FilesystemMediaAdmission, GroupCommitDelay, GroupCommitLimit,
    IdempotencyRetentionGenerations, LiveIdempotencyBindingLimit, PhysicalCheckpointPolicy,
    PhysicalDurabilityDeclaration, PhysicalDurabilityPolicyIdentity, PhysicalIdempotencyPolicy,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationIdentity, PhysicalOperationIdentity,
    PhysicalRuntimeAdmission, PhysicalStore, PhysicalWalPolicy, PhysicalWorkGeneration,
    PhysicalWorkIdentity, RetainedWalTailLimit, RuntimeIdentity, WalSegmentByteLimit,
    WalSegmentInventoryLimit,
};

#[test]
fn unresolved_duplicate_and_conflict_remain_authoritative_after_expiry() {
    let mut fixture = fixture(2);
    let key = fixture
        .registry
        .issue_key(PhysicalMutationIdempotencyMaterial::new([1; 32]))
        .unwrap();
    let first_fingerprint = fingerprint(&fixture, 1);
    let first_mutation = mutation(&fixture, 1);
    let retry_mutation = mutation(&fixture, 2);
    assert!(matches!(
        fixture
            .registry
            .admit_unallocated(key.clone(), first_fingerprint, first_mutation),
        Ok(PhysicalMutationIdempotencyRegistryAdmission::Fresh(binding))
            if binding.key().identity() == key.identity()
                && binding.fingerprint() == first_fingerprint
                && binding.mutation_identity() == first_mutation
    ));

    fixture
        .registry
        .set_namespace_durable_generation_for_test(4);
    assert!(matches!(
        fixture
            .registry
            .admit_unallocated(key.clone(), first_fingerprint, retry_mutation),
        Ok(PhysicalMutationIdempotencyRegistryAdmission::DuplicateUnresolved(existing))
            if existing.key() == key.identity()
                && existing.fingerprint() == first_fingerprint
                && existing.mutation() == first_mutation
    ));
    let conflicting_fingerprint = fingerprint(&fixture, 2);
    assert!(matches!(
        fixture
            .registry
            .admit_unallocated(key, conflicting_fingerprint, retry_mutation),
        Err(PhysicalMutationIdempotencyRegistryDenial::Conflict)
    ));
    fixture.media.close();
}

#[test]
fn unseen_expired_key_is_denied_instead_of_silently_reissued() {
    let mut fixture = fixture(1);
    let key = fixture
        .registry
        .issue_key(PhysicalMutationIdempotencyMaterial::new([2; 32]))
        .unwrap();
    fixture
        .registry
        .set_namespace_durable_generation_for_test(4);
    let unseen_fingerprint = fingerprint(&fixture, 1);
    let unseen_mutation = mutation(&fixture, 1);
    assert!(matches!(
        fixture
            .registry
            .admit_unallocated(key, unseen_fingerprint, unseen_mutation),
        Err(PhysicalMutationIdempotencyRegistryDenial::Expired)
    ));
    fixture.media.close();
}

#[test]
fn pending_bound_and_foreign_mutation_identities_deny_before_insertion() {
    let mut fixture = fixture(1);
    let first = fixture
        .registry
        .issue_key(PhysicalMutationIdempotencyMaterial::new([3; 32]))
        .unwrap();
    let second = fixture
        .registry
        .issue_key(PhysicalMutationIdempotencyMaterial::new([4; 32]))
        .unwrap();
    let foreign_store = published_store([9; 16]);
    let first_fingerprint = fingerprint(&fixture, 1);
    let foreign_store_mutation = mutation_with(&fixture, foreign_store, fixture.runtime, 1);
    assert!(matches!(
        fixture.registry.admit_unallocated(
            first.clone(),
            first_fingerprint,
            foreign_store_mutation
        ),
        Err(PhysicalMutationIdempotencyRegistryDenial::ForeignMutationStore)
    ));
    let foreign_runtime_mutation = mutation_with(
        &fixture,
        fixture.store,
        RuntimeIdentity::generate().unwrap(),
        1,
    );
    assert!(matches!(
        fixture.registry.admit_unallocated(
            first.clone(),
            first_fingerprint,
            foreign_runtime_mutation,
        ),
        Err(PhysicalMutationIdempotencyRegistryDenial::ForeignMutationRuntime)
    ));
    let first_mutation = mutation(&fixture, 1);
    assert!(matches!(
        fixture
            .registry
            .admit_unallocated(first, first_fingerprint, first_mutation),
        Ok(PhysicalMutationIdempotencyRegistryAdmission::Fresh(_))
    ));
    let second_fingerprint = fingerprint(&fixture, 2);
    let second_mutation = mutation(&fixture, 2);
    assert!(matches!(
        fixture
            .registry
            .admit_unallocated(second, second_fingerprint, second_mutation),
        Err(PhysicalMutationIdempotencyRegistryDenial::PendingUnresolvedLimitReached)
    ));
    fixture.media.close();
}

#[test]
fn foreign_store_and_policy_leases_deny_before_binding() {
    let mut fixture = fixture(2);
    let foreign_store_key = PhysicalMutationIdempotencyRegistry::generation_zero(
        published_store([8; 16]),
        fixture.runtime,
        fixture.policy,
        idempotency_policy(2),
    )
    .issue_key(PhysicalMutationIdempotencyMaterial::new([5; 32]))
    .unwrap();
    let foreign_policy_key = PhysicalMutationIdempotencyRegistry::generation_zero(
        fixture.store,
        fixture.runtime,
        fixture.foreign_policy,
        idempotency_policy(2),
    )
    .issue_key(PhysicalMutationIdempotencyMaterial::new([6; 32]))
    .unwrap();
    let request_fingerprint = fingerprint(&fixture, 1);
    let request_mutation = mutation(&fixture, 1);
    assert!(matches!(
        fixture.registry.admit_unallocated(
            foreign_store_key,
            request_fingerprint,
            request_mutation
        ),
        Err(PhysicalMutationIdempotencyRegistryDenial::ForeignStore)
    ));
    assert!(matches!(
        fixture.registry.admit_unallocated(
            foreign_policy_key,
            request_fingerprint,
            request_mutation
        ),
        Err(PhysicalMutationIdempotencyRegistryDenial::ForeignPolicy)
    ));
    fixture.media.close();
}

#[test]
fn derived_fingerprint_matches_an_independent_v1_encoder_and_orders_security_bases() {
    let fixture = fixture(2);
    let scope = [3; 32];
    let payload = [4; 32];
    let security = [[6; 32], [5; 32]];
    let derived = PhysicalMutationRequestFingerprint::derive(PhysicalMutationFingerprintInput {
        store: fixture.store,
        durability_policy: fixture.policy,
        scope: PhysicalMutationRequestScope::record_append(scope),
        payload: PhysicalMutationPayloadDigest::from_validated_payload(payload),
        durability_request: PhysicalMutationDurabilityRequest::PlatformDurable,
        operation_family: PhysicalMutationOperationFamily::RecordAppend,
        security_bases: &security.map(PhysicalMutationSecurityBasis::from_admitted_security),
    })
    .unwrap();
    let reversed = PhysicalMutationRequestFingerprint::derive(PhysicalMutationFingerprintInput {
        store: fixture.store,
        durability_policy: fixture.policy,
        scope: PhysicalMutationRequestScope::record_append(scope),
        payload: PhysicalMutationPayloadDigest::from_validated_payload(payload),
        durability_request: PhysicalMutationDurabilityRequest::PlatformDurable,
        operation_family: PhysicalMutationOperationFamily::RecordAppend,
        security_bases: &security
            .into_iter()
            .rev()
            .map(PhysicalMutationSecurityBasis::from_admitted_security)
            .collect::<Vec<_>>(),
    })
    .unwrap();

    assert_eq!(derived, reversed);
    assert_eq!(
        derived.bytes(),
        independent_v1_fingerprint(
            fixture.store.bytes(),
            fixture.policy.bytes(),
            scope,
            payload,
            security,
        )
    );
    fixture.media.close();
}

fn fixture(pending: u32) -> RegistryFixture {
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
        _ => panic!("test media must admit"),
    };
    let store = media.store_identity();
    let runtime = media.runtime_identity();
    let generation = media.observer().snapshot().unwrap().generation();
    let policy = admitted_policy_identity(&media, 32);
    let foreign_policy = admitted_policy_identity(&media, 64);
    let idempotency = idempotency_policy(pending);
    let registry =
        PhysicalMutationIdempotencyRegistry::generation_zero(store, runtime, policy, idempotency);
    RegistryFixture {
        _root: root,
        media,
        registry,
        store,
        runtime,
        generation,
        policy,
        foreign_policy,
        idempotency,
    }
}

fn admitted_policy_identity(
    media: &crate::physical_runtime::MediaOwnedPhysicalRuntime,
    group_limit: u32,
) -> PhysicalDurabilityPolicyIdentity {
    let basis = media.physical_durability_admission_basis().unwrap();
    match PhysicalDurabilityDeclaration::builder()
        .group_commit(
            GroupCommitLimit::new(NonZeroU32::new(group_limit).unwrap()),
            GroupCommitDelay::new(NonZeroU64::new(1).unwrap()),
        )
        .wal(PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(NonZeroU64::new(1024).unwrap()),
            WalSegmentInventoryLimit::new(NonZeroU32::new(64).unwrap()),
        ))
        .idempotency(PhysicalIdempotencyPolicy::new(
            retention(),
            pending_limit(2),
            live_limit(16),
        ))
        .checkpoint(PhysicalCheckpointPolicy::fuzzy(
            CheckpointMemoryLimit::new(NonZeroU64::new(1024).unwrap()),
            RetainedWalTailLimit::new(NonZeroU64::new(4096).unwrap()),
        ))
        .admit(basis)
        .into_raw()
    {
        TransitionOutcome::Success(policy) => policy.identity(),
        _ => panic!("test durability policy must admit"),
    }
}

fn mutation(fixture: &RegistryFixture, operation: u64) -> PhysicalMutationIdentity {
    mutation_with(fixture, fixture.store, fixture.runtime, operation)
}

fn mutation_with(
    fixture: &RegistryFixture,
    store: StableStoreIdentity,
    runtime: RuntimeIdentity,
    operation: u64,
) -> PhysicalMutationIdentity {
    PhysicalMutationIdentity::from_reserved_operation(PhysicalWorkIdentity::from_instance_owner(
        store,
        runtime,
        PhysicalWorkGeneration::from_lifecycle(fixture.generation),
        PhysicalOperationIdentity::from_owner_sequence(NonZeroU64::new(operation).unwrap()),
    ))
}

fn fingerprint(fixture: &RegistryFixture, payload: u8) -> PhysicalMutationRequestFingerprint {
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

fn retention() -> IdempotencyRetentionGenerations {
    IdempotencyRetentionGenerations::new(NonZeroU64::new(4).unwrap())
}

fn pending_limit(value: u32) -> PendingUnresolvedMutationLimit {
    PendingUnresolvedMutationLimit::new(NonZeroU32::new(value).unwrap())
}

fn live_limit(value: u32) -> LiveIdempotencyBindingLimit {
    LiveIdempotencyBindingLimit::new(NonZeroU32::new(value).unwrap())
}

fn idempotency_policy(pending: u32) -> PhysicalIdempotencyPolicy {
    PhysicalIdempotencyPolicy::new(retention(), pending_limit(pending), live_limit(16))
}

fn published_store(bytes: [u8; 16]) -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes(bytes).unwrap(),
    )
    .published_identity()
}

fn independent_v1_fingerprint(
    store: [u8; 16],
    policy: [u8; 32],
    scope: [u8; 32],
    payload: [u8; 32],
    mut security: [[u8; 32]; 2],
) -> [u8; 32] {
    security.sort_unstable();
    let mut digest = Sha256::new();
    for field in [
        b"store.physical.mutation.request-fingerprint.v1".as_slice(),
        &store,
        &policy,
        &[1],
        &scope,
        &payload,
        &[1],
        &[1],
        &2_u32.to_le_bytes(),
        &security[0],
        &security[1],
    ] {
        digest.update((field.len() as u64).to_le_bytes());
        digest.update(field);
    }
    digest.finalize().into()
}

#[test]
fn runtime_owner_binding_counts_split_prior_and_current_generation() {
    use super::super::PhysicalMutationIdempotencyRuntimeOwner as Owner;
    let (mut f, mat) = (fixture(4), PhysicalMutationIdempotencyMaterial::new);
    let k0 = f.registry.issue_key(mat([41; 32])).unwrap();
    f.registry
        .admit_unallocated(k0, fingerprint(&f, 1), mutation(&f, 1))
        .unwrap();
    let (fp2, m2) = (fingerprint(&f, 2), mutation(&f, 2));
    f.registry.set_namespace_durable_generation_for_test(1);
    let owner = Owner::from_rebuilt_registry(f.registry);
    assert_eq!(owner.binding_counts(), (1, 0));
    let auth = Owner::authority(&owner);
    let k1 = auth.issue_key(mat([42; 32])).unwrap();
    assert!(auth
        .admit_unallocated_with(k1, fp2, || Ok::<_, ()>(m2))
        .is_ok());
    assert_eq!(owner.binding_counts(), (2, 1));
}
