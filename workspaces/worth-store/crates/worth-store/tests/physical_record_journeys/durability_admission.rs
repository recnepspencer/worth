use std::num::NonZeroU32;

use super::{configuration, durability_with_group_limit, media, success};
use worth_store::physical_runtime::{
    PhysicalMutationIdempotencyIssuanceDenial, PhysicalMutationIdempotencyMaterial,
    PhysicalRecordInitialization,
};

#[path = "durability_admission/checkpoint_capture.rs"]
mod checkpoint_capture;
#[path = "durability_admission/checkpoint_lifecycle.rs"]
mod checkpoint_lifecycle;
#[path = "durability_admission/checkpoint_pressure.rs"]
mod checkpoint_pressure;
#[path = "durability_admission/checkpoint_retained_wal_tail.rs"]
mod checkpoint_retained_wal_tail;
#[path = "durability_admission/checkpoint_wal_reclamation.rs"]
mod checkpoint_wal_reclamation;
#[path = "durability_admission/closeout_handoff.rs"]
mod closeout_handoff;
#[path = "durability_admission/data_durability.rs"]
mod data_durability;
#[path = "durability_admission/group_commit.rs"]
mod group_commit;
#[path = "durability_admission/idempotency_reopen.rs"]
mod idempotency_reopen;
#[path = "durability_admission/independent_wal_oracle.rs"]
mod independent_wal_oracle;
#[path = "durability_admission/managed_mutation.rs"]
mod managed_mutation;
#[path = "durability_admission/mutation_preparation.rs"]
mod mutation_preparation;
#[path = "durability_admission/stream_failure.rs"]
mod stream_failure;
#[path = "durability_admission/wal_append.rs"]
mod wal_append;
#[path = "durability_admission/wal_attempt_binding_inspection.rs"]
mod wal_attempt_binding_inspection;
#[path = "durability_admission/wal_barrier.rs"]
mod wal_barrier;
#[path = "durability_admission/wal_group_continuation.rs"]
mod wal_group_continuation;
#[path = "durability_admission/wal_preparation_authority.rs"]
mod wal_preparation_authority;
#[path = "durability_admission/wal_reopen.rs"]
mod wal_reopen;
#[path = "durability_admission/wal_rotation.rs"]
mod wal_rotation;
#[path = "durability_admission/wal_submission_admission.rs"]
mod wal_submission_admission;

#[test]
fn admitted_limits_and_identity_are_observable_without_exposing_policy_authority() {
    let parent = tempfile::tempdir().unwrap();
    let media = media(&parent.path().join("store"));
    let ordinary = durability_with_group_limit(&media, NonZeroU32::new(32).unwrap());
    let ordinary_identity = ordinary.identity();
    {
        let changed = durability_with_group_limit(&media, NonZeroU32::new(64).unwrap());
        assert_ne!(ordinary_identity, changed.identity());
        assert_eq!(
            ordinary.admission_basis_identity(),
            changed.admission_basis_identity(),
        );
    }

    let (format, placement, access) = configuration();
    let serving = success(
        media.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, ordinary,
        )),
    );
    let observed = serving.durability_observation();
    assert_eq!(observed.store_identity(), serving.store_identity());
    assert_eq!(observed.runtime_identity(), serving.runtime_identity());
    assert_eq!(observed.policy_identity(), ordinary_identity);
    assert_eq!(observed.group_commit_limit().get().get(), 32);
    assert_eq!(observed.group_commit_delay().signal_duration().get(), 1);
    assert_eq!(observed.idempotency_policy().retention().get().get(), 4,);
    assert_eq!(
        observed
            .idempotency_policy()
            .pending_unresolved_limit()
            .get()
            .get(),
        1_024,
    );
    assert_eq!(
        observed.checkpoint_policy().memory_limit().get().get(),
        16 * 1024 * 1024,
    );
    assert_eq!(
        observed
            .checkpoint_policy()
            .retained_wal_tail_limit()
            .get()
            .get(),
        64 * 1024 * 1024,
    );
    serving.close();
}

#[test]
fn serving_issues_policy_bound_generation_leases_until_publication_shutdown() {
    let parent = tempfile::tempdir().unwrap();
    let media = media(&parent.path().join("store"));
    let policy = durability_with_group_limit(&media, NonZeroU32::new(32).unwrap());
    let policy_identity = policy.identity();
    let (format, placement, access) = configuration();
    let serving = success(
        media.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, policy,
        )),
    );
    let store_identity = serving.store_identity();
    let submission = serving.record_submission();

    let first = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([7; 32]))
        .unwrap();
    let retry = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([7; 32]))
        .unwrap();
    let distinct = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([8; 32]))
        .unwrap();

    assert_eq!(first.identity(), retry.identity());
    assert_eq!(first.lease(), retry.lease());
    assert_ne!(first.identity(), distinct.identity());
    assert_eq!(first.lease().store_identity(), store_identity);
    assert_eq!(first.lease().policy_identity(), policy_identity);
    assert_eq!(first.lease().issuance_generation().get(), 0);
    assert_eq!(first.lease().expiry_generation().get(), 4);
    assert!(!first
        .lease()
        .is_expired_at(first.lease().issuance_generation()));

    serving.close();
    assert_eq!(
        submission.issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([9; 32])),
        Err(PhysicalMutationIdempotencyIssuanceDenial::DurabilityAuthorityReleased),
    );
}
