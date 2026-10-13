//! Allocation accounting for a map transported inside another pattern.
use super::*;
impl<T: ChargedBytes, K> ChargedBytes for ExecutionMap<T, K> {
    fn additional_charged_bytes(&self) -> u64 {
        let Self {
            batch,
            _read_sets: _,
            _write_sets: _,
        } = self;
        // The batch's access declaration is the owner's retained access-set charge.
        batch.owned_storage_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    #[test]
    fn transporting_a_map_counts_its_inputs_capacity() {
        let map = |value: Vec<u8>| {
            ExecutionMap::<_, u64>::from_keyless_partitions(BTreeMap::from([(
                worth_foundational::PartitionIdentity::new(1),
                KeylessPartition {
                    value,
                    kernel_scratch_bytes: 0,
                    max_result_bytes: 0,
                },
            )]))
            .unwrap()
        };
        let empty = map(Vec::new());
        let mut allocated = Vec::with_capacity(128);
        allocated.push(1);
        let capacity = allocated.capacity() as u64;
        let allocated = map(allocated);
        assert_eq!(
            allocated.additional_charged_bytes() - empty.additional_charged_bytes(),
            capacity
        );
    }
}
