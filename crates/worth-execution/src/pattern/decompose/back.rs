use worth_foundational::PartitionIdentity;
use worth_proof::CanonicalUniqueVec;

use crate::report::ChargedBytes;

use super::{BackInput, ExecutionMap, MapDenial, MapPartition};

pub(super) fn admit_back_map<I: Clone + ChargedBytes, B: Clone + ChargedBytes>(
    all_identities: &CanonicalUniqueVec<PartitionIdentity>,
    changed_identities: &[PartitionIdentity],
    interiors: &[I],
    slices: &[B],
    max_result_bytes: u64,
) -> Result<ExecutionMap<BackInput<I, B>, PartitionIdentity>, MapDenial> {
    let partitions = changed_identities
        .iter()
        .map(|identity| {
            let index = all_identities
                .as_slice()
                .binary_search(identity)
                .expect("checked identity");
            MapPartition {
                identity: *identity,
                value: BackInput {
                    interior: interiors[index].clone(),
                    interface_slice: slices[index].clone(),
                },
                read_keys: Vec::new(),
                write_keys: Vec::new(),
                kernel_scratch_bytes: 0,
                max_result_bytes,
            }
        })
        .collect();
    ExecutionMap::try_from_declared_partitions(changed_identities.to_vec(), partitions)
}
