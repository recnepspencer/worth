//! Owner walks over immutable radix/Cold inputs, without runtime construction.
use super::*;
use crate::branch::{RelationalPersistentRegionSet, RelationalRootRegionObservation};
use crate::durability::medium::{
    RelationalDurableColdSource, RelationalDurableMemberCustody, RelationalDurableMemberLocator,
};
use crate::identity::data::PartitionId;
use crate::storage::{
    data::{RelationalStorageUnavailable, RelationalStorageUnavailableCause as Cause},
    overlay::PartitionState,
    residency::{RelationalColdRegion, RelationalRegionKindCommitment, RelationalRegionResidency},
};
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

#[derive(Default)]
struct UnionWalk {
    allocations: BTreeMap<(RelationalRootAuthoritativeAllocationKind, u64), u64>,
}
impl RelationalRootAllocationVisitor for UnionWalk {
    type Error = ();
    fn allocation(&mut self, value: RelationalRootAuthoritativeAllocationObservation) -> bool {
        self.allocations
            .insert((value.kind, value.owner_id), value.authoritative_bytes)
            .is_none()
    }
    fn region(
        &mut self,
        _: RelationalRootRegionObservation,
        partition: Result<Option<&PartitionState>, RelationalStorageUnavailable>,
    ) -> Result<(), ()> {
        partition.map_err(|_| ())?.ok_or(())?;
        Ok(())
    }
}

#[test]
fn shared_radix_union_matches_independent_full_walks_in_both_orders() {
    let issuer = super::super::super::RelationalBranchRootIdentityIssuer::default();
    let first = RelationalPersistentRegionSet::initial(101, BTreeMap::new(), &issuer).unwrap();
    let second = RelationalPersistentRegionSet::replace(
        102,
        &first,
        BTreeMap::new(),
        [PartitionId(7), PartitionId(11)].into_iter().collect(),
        &issuer,
    )
    .unwrap();
    let third = RelationalPersistentRegionSet::replace(
        103,
        &second,
        BTreeMap::new(),
        [PartitionId(17)].into_iter().collect(),
        &issuer,
    )
    .unwrap();
    let expected = [&first, &second, &third]
        .into_iter()
        .flat_map(|set| set.allocation_observations())
        .map(RelationalRootAuthoritativeAllocationObservation::from)
        .map(|row| ((row.kind, row.owner_id), row.authoritative_bytes))
        .collect::<BTreeMap<_, _>>();
    for roots in [[&first, &second, &third], [&third, &second, &first]] {
        let mut union = UnionWalk::default();
        for root in roots {
            root.try_visit_allocation_union(&mut union).unwrap();
        }
        assert_eq!(union.allocations, expected);
        let before = union.allocations.clone();
        third.try_visit_allocation_union(&mut union).unwrap();
        assert_eq!(union.allocations, before);
    }
}

#[derive(Debug, Default)]
struct UnreadSource(AtomicUsize);
impl RelationalDurableColdSource for UnreadSource {
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        Ok(RelationalDurableMemberCustody::new(()))
    }
    fn read_member(&self, _: RelationalDurableMemberLocator, _: u64) -> Result<Vec<u8>, Cause> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err(Cause::SourceUnavailable)
    }
    fn read_commit(
        &self,
        _: crate::publication::patch::data::PatchStreamPosition,
        _: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err(Cause::SourceUnavailable)
    }
}

#[test]
fn cold_region_union_refuses_without_demanding_any_payload() {
    // The unavailable source and its weak member labels are denial inputs,
    // never a claim that a durable payload or application was admitted.
    let source = Arc::new(UnreadSource::default());
    let locator = RelationalDurableMemberLocator {
        sha256: [0; 32],
        length: 0,
    };
    let kind_authority = RelationalRegionKindCommitment {
        head: [0; 32],
        history: [0; 32],
    };
    let cell = RelationalRegionResidency::cold(RelationalColdRegion {
        source: source.clone(),
        partition_id: PartitionId(7),
        head: locator,
        history: locator,
        history_members: Arc::new(Default::default()),
        kind_authority,
        schema: super::super::super::RelationalBranchRootSchemaAuthority::empty(),
        schema_carrier_digest: [0; 32],
        symbols: Default::default(),
        content_digest: [0; 32],
        entity_slots: 0,
        relation_slots: 0,
        unavailable_records: 0,
    })
    .unwrap();
    let region = Arc::new(
        crate::branch::root_region::RelationalRootRegion::from_checkpoint(
            1,
            1,
            crate::branch::RelationalRootRegionCheckpoint {
                partition_id: PartitionId(7),
                cell,
                kind_authority: None,
                kind_commitment: kind_authority,
                content_digest: [0; 32],
                entity_slots: 0,
                relation_slots: 0,
                unavailable_records: 0,
            },
        ),
    );
    let issuer = super::super::super::RelationalBranchRootIdentityIssuer::default();
    let set = RelationalPersistentRegionSet::initial(
        101,
        [(PartitionId(7), region)].into_iter().collect(),
        &issuer,
    )
    .unwrap();
    let mut walk = UnionWalk::default();
    assert_eq!(set.try_visit_allocation_union(&mut walk), Err(()));
    assert_eq!(source.0.load(Ordering::SeqCst), 0);
    assert!(walk.allocations.keys().any(|(kind, _)| *kind
        == RelationalRootAuthoritativeAllocationKind::PersistentRegionReplacementStorage));
}
