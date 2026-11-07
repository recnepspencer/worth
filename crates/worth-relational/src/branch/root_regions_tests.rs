use std::collections::{BTreeMap, BTreeSet};

use super::{RelationalPersistentRegionAllocationKind, RelationalPersistentRegionSet};
use crate::branch::root::RelationalBranchRootIdentityIssuer;
use crate::identity::data::PartitionId;

#[test]
fn persistent_index_carries_explicit_set_and_removal_allocations() {
    let issuer = RelationalBranchRootIdentityIssuer::default();
    let initial = RelationalPersistentRegionSet::initial(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        1,
        BTreeMap::new(),
        &issuer,
    )
    .expect("empty owner index builds");
    let removed = [PartitionId(7), PartitionId(11)]
        .into_iter()
        .collect::<BTreeSet<_>>();
    let replaced =
        RelationalPersistentRegionSet::replace(2, &initial, BTreeMap::new(), removed, &issuer)
            .expect("removal paths build");
    let observations = replaced.allocation_observations();
    assert!(observations.iter().any(|allocation| {
        allocation.allocation_kind == RelationalPersistentRegionAllocationKind::SetObject
    }));
    assert_eq!(
        observations
            .iter()
            .filter(|allocation| {
                allocation.allocation_kind
                    == RelationalPersistentRegionAllocationKind::RemovalStorage
            })
            .count(),
        2
    );
}

#[test]
fn keyed_merkle_commitment_rejects_compensating_leaf_substitution() {
    let issuer = RelationalBranchRootIdentityIssuer::default();
    let initial = RelationalPersistentRegionSet::initial(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        1,
        BTreeMap::new(),
        &issuer,
    )
    .expect("empty owner index builds");
    let left = RelationalPersistentRegionSet::replace(
        2,
        &initial,
        BTreeMap::new(),
        [PartitionId(7), PartitionId(11)].into_iter().collect(),
        &issuer,
    )
    .expect("first keyed removal shape builds");
    let right = RelationalPersistentRegionSet::replace(
        3,
        &initial,
        BTreeMap::new(),
        [PartitionId(8), PartitionId(10)].into_iter().collect(),
        &issuer,
    )
    .expect("compensating keyed removal shape builds");

    assert_eq!(left.len(), right.len());
    assert_ne!(left.commitment(), right.commitment());
}
