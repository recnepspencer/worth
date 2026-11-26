use std::num::{NonZeroU32, NonZeroU64};

use worth_store_physical_format::PhysicalCheckpointIdentity;

use super::*;
use crate::physical_runtime::durability::mutation::idempotency::persisted_binding::tests::fixture::single_member;
use crate::physical_runtime::durability::mutation::idempotency::persisted_binding::PhysicalBindingDecodingContext;
use crate::physical_runtime::durability::mutation::idempotency::registry::{
    PhysicalMutationIdempotencyRegistryAdmission as Admission,
    PhysicalMutationIdempotencyRegistryDenial, PhysicalMutationWalBindingDenial,
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

/// Issue #423: a session that completed more WAL members than the pending
/// bound reopened that committed history as pending and refused ServingOpen.
#[test]
fn committed_wal_history_beyond_the_pending_bound_reopens_with_both_bounds_intact() {
    let mut fixture = fixture(4);
    let keys = (40..46_u8)
        .map(|material| {
            fixture
                .registry
                .issue_key(PhysicalMutationIdempotencyMaterial::new([material; 32]))
                .unwrap()
        })
        .collect::<Vec<_>>();
    let members = (40..46_u8)
        .zip(&keys)
        .map(|(payload, key)| {
            let start = u64::from(payload) - 39;
            let binding = single_member(&fixture, key.clone(), payload, payload.into(), start);
            ReopenedPhysicalWalMember::from_verified_parts(
                binding.member().lsn_range(),
                binding.bytes(),
                binding.redo_digest(),
            )
        })
        .collect::<Vec<_>>();
    // The first attempt was compacted before WAL binding; its retained member
    // upgrades it, releasing the only pending slot.
    assert!(matches!(
        fixture.registry.admit_unallocated(
            keys[0].clone(),
            fingerprint(&fixture, 40),
            mutation(&fixture, 40)
        ),
        Ok(Admission::Fresh(_))
    ));
    let records = compaction_records(&fixture);
    let mut reopened = rebuilder(&fixture, idempotency_policy(1, 16));
    reopened.consume_compaction_record(&records[0]).unwrap();
    assert_eq!(reopened.pending_count, 1);
    for member in members.clone() {
        reopened.consume_wal_member(member).unwrap();
    }
    reopened.validate_groups().unwrap();
    assert_eq!(reopened.pending_count, 0);
    let mut registry = reopened.finish(checkpoint_compaction::admit_generation_zero());

    // Exact retry joins the durable attempt; a substituted payload conflicts.
    assert!(matches!(
        registry.admit_unallocated(keys[5].clone(), fingerprint(&fixture, 45), mutation(&fixture, 90)),
        Ok(Admission::DuplicateUnresolved(observation))
            if observation.mutation() == mutation(&fixture, 45)
    ));
    assert!(matches!(
        registry.admit_unallocated(
            keys[5].clone(),
            fingerprint(&fixture, 99),
            mutation(&fixture, 91)
        ),
        Err(PhysicalMutationIdempotencyRegistryDenial::Conflict)
    ));
    // The genuine in-flight bound still admits one and refuses the next.
    let issue = |material| {
        registry
            .issue_key(PhysicalMutationIdempotencyMaterial::new([material; 32]))
            .unwrap()
    };
    let (fresh, next) = (issue(80), issue(81));
    assert!(matches!(
        registry.admit_unallocated(fresh, fingerprint(&fixture, 80), mutation(&fixture, 80)),
        Ok(Admission::Fresh(_))
    ));
    assert!(matches!(
        registry.admit_unallocated(next, fingerprint(&fixture, 81), mutation(&fixture, 81)),
        Err(PhysicalMutationIdempotencyRegistryDenial::PendingUnresolvedLimitReached)
    ));

    // Committed history still counts against the total live-binding bound.
    let mut live_bounded = rebuilder(&fixture, idempotency_policy(1, 4));
    let refused = members
        .into_iter()
        .map(|member| live_bounded.consume_wal_member(member))
        .find(Result::is_err);
    assert_eq!(
        refused,
        Some(Err(
            PhysicalIdempotencyReopenFailure::LiveBindingLimitExceeded
        ))
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
            Ok(Admission::Fresh(_))
        ));
    }
    compaction_records(fixture)
}

fn compaction_records(fixture: &RegistryFixture) -> Vec<Vec<u8>> {
    let pending = fixture
        .registry
        .prepare_binding_compaction(
            PhysicalCheckpointIdentity::new(fixture.store, NonZeroU64::new(1).unwrap()),
            1,
            crate::physical_runtime::record_serving::SelectedBlobManifestPins::empty_for_test(),
            u64::MAX,
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
