use std::collections::BTreeMap;

use worth_foundational::PartitionIdentity;
use worth_proof::{CanonicalUniqueVec, DisjointKeySetFamily};

use crate::{backend::AdmittedBatch, report::ChargedBytes};

use super::{access::access_memory_bytes, ExecutionMap};

/// One dispatched value that declares no access keys.
pub struct KeylessPartition<T> {
    pub value: T,
    pub kernel_scratch_bytes: u64,
    pub max_result_bytes: u64,
}

/// A keyless map's declared bytes do not fit a byte count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapMemoryOverflow;

impl<T, K: Ord + ChargedBytes> ExecutionMap<T, K> {
    /// A map over partitions keyed by identity. A map's identities iterate in
    /// canonical order and sets with no keys cannot overlap or conflict, so
    /// the one refusal is a byte count that does not fit.
    pub fn from_keyless_partitions(
        partitions: BTreeMap<PartitionIdentity, KeylessPartition<T>>,
    ) -> Result<Self, MapMemoryOverflow> {
        let identities = CanonicalUniqueVec::from_btree_set(partitions.keys().copied().collect());
        let write_sets = DisjointKeySetFamily::with_empty_sets(&identities);
        let write_member_capacity = write_sets.members().len();
        let read_sets: Vec<Vec<K>> = partitions.keys().map(|_| Vec::new()).collect();
        let access_memory_bytes = access_memory_bytes(
            &read_sets,
            read_sets.capacity(),
            &write_sets,
            write_member_capacity,
        )
        .ok_or(MapMemoryOverflow)?;
        let (values, capacities): (Vec<_>, Vec<_>) = partitions
            .into_values()
            .map(|partition| {
                (
                    partition.value,
                    (partition.kernel_scratch_bytes, partition.max_result_bytes),
                )
            })
            .unzip();
        let batch =
            AdmittedBatch::admit_canonical(identities, values, capacities, access_memory_bytes)
                .ok_or(MapMemoryOverflow)?;
        Ok(Self {
            batch,
            _read_sets: read_sets,
            _write_sets: write_sets,
        })
    }
}
