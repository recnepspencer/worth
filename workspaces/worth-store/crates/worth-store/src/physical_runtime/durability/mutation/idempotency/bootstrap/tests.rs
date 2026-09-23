use std::num::{NonZeroU32, NonZeroU64};

use worth_store_physical_format::PhysicalCheckpointIdentity;

use super::*;
use crate::physical_runtime::durability::mutation::idempotency::persisted_binding::PhysicalBindingDecodingContext;
use crate::physical_runtime::durability::mutation::idempotency::registry::{
    PhysicalMutationIdempotencyRegistryAdmission, PhysicalMutationWalBindingDenial,
};
use crate::physical_runtime::durability::mutation::idempotency::test_support::{
    fingerprint, fixture, idempotency_policy, mutation, RegistryFixture,
};
use crate::physical_runtime::{
    PhysicalDurabilityGroupIdentity, PhysicalMutationIdempotencyMaterial,
};

#[test]
fn reopen_accepts_only_new_or_exactly_repeated_wal_bindings() {
    assert_eq!(classify_reopened_wal_binding(Ok(())), Ok(()));
    if classify_reopened_wal_binding(Err(PhysicalMutationWalBindingDenial::AlreadyWalBound))
        .is_err()
    {
        panic!("MUTANT_PREDICATE:c7-reopen-exact-wal-overlap-rejected");
    }
    for denial in [
        PhysicalMutationWalBindingDenial::BindingMismatch,
        PhysicalMutationWalBindingDenial::ProvenNoEffect,
    ] {
        assert_eq!(
            classify_reopened_wal_binding(Err(denial)),
            Err(PhysicalIdempotencyReopenFailure::WalBindingConflict)
        );
    }
}

#[test]
fn compaction_rebuild_rejects_reordered_records_and_independent_bounds() {
    let mut fixture = fixture(4);
    let mut records = compacted_unsealed_records(&mut fixture, 2);
    records.reverse();
    let mut reordered = rebuilder(&fixture, fixture.idempotency);
    reordered.consume_compaction_record(&records[0]).unwrap();
    assert_eq!(
        reordered.consume_compaction_record(&records[1]),
        Err(PhysicalIdempotencyReopenFailure::NonCanonicalCompactionOrder)
    );

    records.reverse();
    let live_one = idempotency_policy(4, 1);
    let mut live_bounded = rebuilder(&fixture, live_one);
    live_bounded.consume_compaction_record(&records[0]).unwrap();
    assert_eq!(
        live_bounded.consume_compaction_record(&records[1]),
        Err(PhysicalIdempotencyReopenFailure::LiveBindingLimitExceeded)
    );

    let pending_one = idempotency_policy(1, 4);
    let mut pending_bounded = rebuilder(&fixture, pending_one);
    pending_bounded
        .consume_compaction_record(&records[0])
        .unwrap();
    assert_eq!(
        pending_bounded.consume_compaction_record(&records[1]),
        Err(PhysicalIdempotencyReopenFailure::PendingUnresolvedLimitExceeded)
    );
    fixture.media.close();
}

#[test]
fn group_validation_rejects_missing_members_and_substituted_membership_digest() {
    let fixture = fixture(4);
    let first_key = fixture
        .registry
        .issue_key(PhysicalMutationIdempotencyMaterial::new([71; 32]))
        .unwrap();
    let second_key = fixture
        .registry
        .issue_key(PhysicalMutationIdempotencyMaterial::new([72; 32]))
        .unwrap();
    let first_mutation = mutation(&fixture, 11);
    let second_mutation = mutation(&fixture, 12);
    let group = PhysicalDurabilityGroupIdentity::from_reopened([73; 32]);
    let count = NonZeroU32::new(2).unwrap();
    let first_member =
        crate::physical_runtime::PhysicalWalMemberIdentity::for_mutation(first_mutation);
    let second_member =
        crate::physical_runtime::PhysicalWalMemberIdentity::for_mutation(second_mutation);
    let first_binding =
        crate::physical_runtime::PhysicalDurabilityGroupMemberBinding::from_reopened(
            group,
            first_member,
            NonZeroU32::new(1).unwrap(),
            count,
            [74; 32],
        )
        .unwrap();
    let second_binding =
        crate::physical_runtime::PhysicalDurabilityGroupMemberBinding::from_reopened(
            group,
            second_member,
            NonZeroU32::new(2).unwrap(),
            count,
            [74; 32],
        )
        .unwrap();
    let first = ReopenedGroupMember::new(first_key.identity(), first_mutation, first_binding);
    assert_eq!(
        validate_group(&mut [first]),
        Err(PhysicalIdempotencyReopenFailure::GroupMemberCountMismatch)
    );
    let first = ReopenedGroupMember::new(first_key.identity(), first_mutation, first_binding);
    let second = ReopenedGroupMember::new(second_key.identity(), second_mutation, second_binding);
    assert_eq!(
        validate_group(&mut [first, second]),
        Err(PhysicalIdempotencyReopenFailure::GroupMembershipMismatch)
    );
    fixture.media.close();
}

fn compacted_unsealed_records(fixture: &mut RegistryFixture, count: u8) -> Vec<Vec<u8>> {
    for index in 0..count {
        let key = fixture
            .registry
            .issue_key(PhysicalMutationIdempotencyMaterial::new([index + 1; 32]))
            .unwrap();
        assert!(matches!(
            fixture.registry.admit_unallocated(
                key,
                fingerprint(fixture, index + 1),
                mutation(fixture, u64::from(index) + 1),
            ),
            Ok(PhysicalMutationIdempotencyRegistryAdmission::Fresh(_))
        ));
    }
    let pending = fixture
        .registry
        .prepare_binding_compaction(
            PhysicalCheckpointIdentity::new(fixture.store, NonZeroU64::new(1).unwrap()),
            1,
        )
        .unwrap();
    let mut records = Vec::new();
    pending
        .for_each_record(&fixture.registry, |record| {
            records.push(record.to_vec());
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap();
    records
}

fn rebuilder(
    fixture: &RegistryFixture,
    idempotency: crate::physical_runtime::PhysicalIdempotencyPolicy,
) -> PhysicalIdempotencyRegistryRebuilder {
    PhysicalIdempotencyRegistryRebuilder::new(
        fixture.store,
        fixture.runtime,
        fixture.policy,
        idempotency,
        PhysicalNamespaceDurableCheckpointGeneration::from_namespace_durable_checkpoint(1),
        PhysicalBindingDecodingContext::new(fixture.store, fixture.policy, idempotency),
        Some(1),
        Vec::new(),
    )
}
