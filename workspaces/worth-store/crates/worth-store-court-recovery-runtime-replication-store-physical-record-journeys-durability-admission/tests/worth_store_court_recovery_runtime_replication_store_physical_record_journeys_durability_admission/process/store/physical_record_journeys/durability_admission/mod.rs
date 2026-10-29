use {std::num::NonZeroU32};

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_with_group_limit, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::success};
use {worth_store::physical_runtime::PhysicalMutationIdempotencyIssuanceDenial, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalRecordInitialization};











pub(super) use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::rewrite_redo::produced_rewrite_payloads};













#[test]
fn admitted_limits_and_identity_are_observable_without_exposing_policy_authority() {
    let parent = tempfile::tempdir().unwrap();
    let media = media(&parent.path().join("store"));
    let ordinary = durability_with_group_limit(&media, std::num::NonZeroU32::new(32).unwrap());
    let ordinary_identity = ordinary.identity();
    {
        let changed = durability_with_group_limit(&media, std::num::NonZeroU32::new(64).unwrap());
        assert_ne!(ordinary_identity, changed.identity());
        assert_eq!(
            ordinary.admission_basis_identity(),
            changed.admission_basis_identity(),
        );
    }

    let (format, placement, access) = configuration();
    let serving = success(
        media.initialize_record_store(worth_store::physical_runtime::PhysicalRecordInitialization::new(
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
    let policy = durability_with_group_limit(&media, std::num::NonZeroU32::new(32).unwrap());
    let policy_identity = policy.identity();
    let (format, placement, access) = configuration();
    let serving = success(
        media.initialize_record_store(worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, policy,
        )),
    );
    let store_identity = serving.store_identity();
    let submission = serving.record_submission();

    let first = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([7; 32]))
        .unwrap();
    let retry = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([7; 32]))
        .unwrap();
    let distinct = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([8; 32]))
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
        submission.issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([9; 32])),
        Err(worth_store::physical_runtime::PhysicalMutationIdempotencyIssuanceDenial::DurabilityAuthorityReleased),
    );
}
#[path = "checkpoint_wal_reclamation/mod.rs"]
pub(crate) mod checkpoint_wal_reclamation;
#[path = "closeout_handoff/mod.rs"]
pub(crate) mod closeout_handoff;
#[path = "managed_mutation/mod.rs"]
pub(crate) mod managed_mutation;
#[path = "wal_reopen/mod.rs"]
pub(crate) mod wal_reopen;
