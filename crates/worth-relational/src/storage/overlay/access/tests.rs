use super::*;

struct Ids(Vec<PartitionId>);

impl PartitionAccess for Ids {
    fn get_partition(&self, _: PartitionId) -> Option<&PartitionState> {
        None
    }
    fn partition_ids(&self) -> Vec<PartitionId> {
        self.0.clone()
    }
    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, PartitionIdsDenial> {
        collect_partition_ids(self.0.len(), self.0.iter().copied(), memory)
    }
    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        Box::new(self.0.iter().copied())
    }
}

struct Memory {
    held: u64,
    peak: u64,
    ceiling: u64,
}

impl PartitionIdMemory for Memory {
    fn reserve(&mut self, bytes: u64) -> Result<(), worth_execution::MemoryLimitDenial> {
        let total = self.held.checked_add(bytes).expect("small fixture");
        if total > self.ceiling {
            return Err(worth_execution::MemoryLimitDenial::Capacity {
                requested: bytes,
                admitted: self.ceiling - self.held,
                level: worth_execution::MemoryLimitLevel::Policy { ancestor: 2 },
            });
        }
        self.held = total;
        self.peak = self.peak.max(total);
        Ok(())
    }
    fn release(&mut self, bytes: u64) {
        self.held -= bytes;
    }
}

#[test]
fn partition_vector_refusal_precedes_source_iteration() {
    let visited = std::cell::Cell::new(0);
    let ids = [PartitionId(1)]
        .into_iter()
        .inspect(|_| visited.set(visited.get() + 1));
    let mut memory = Memory {
        held: 0,
        peak: 0,
        ceiling: 0,
    };
    let result = collect_partition_ids(1, ids, &mut memory);
    assert_eq!(
        result,
        Err(PartitionIdsDenial::Memory(
            worth_execution::MemoryLimitDenial::Capacity {
                requested: std::mem::size_of::<PartitionId>() as u64,
                admitted: 0,
                level: worth_execution::MemoryLimitLevel::Policy { ancestor: 2 },
            }
        ))
    );
    assert_eq!(visited.get(), 0);
    assert_eq!(memory.held, 0);
}

#[test]
fn partition_merge_funds_coexistence_and_releases_inputs_on_refusal() {
    let base = Ids(vec![PartitionId(1), PartitionId(3)]);
    let staged = Ids(vec![PartitionId(2), PartitionId(3)]);
    let overlay = OverlayStateView::new(&base, &staged);
    let item = std::mem::size_of::<PartitionId>() as u64;
    let mut memory = Memory {
        held: 0,
        peak: 0,
        ceiling: 8 * item,
    };
    let output = overlay.partition_ids_with_memory(&mut memory).unwrap();
    assert_eq!(output, [PartitionId(1), PartitionId(2), PartitionId(3)]);
    assert_eq!(memory.held, partition_id_bytes(output.capacity()).unwrap());
    assert_eq!(memory.peak, 8 * item);
    memory::release_ids(output, &mut memory);
    assert_eq!(memory.held, 0);
    memory.ceiling = 8 * item - 1;
    assert!(matches!(
        overlay.partition_ids_with_memory(&mut memory),
        Err(PartitionIdsDenial::Memory(
            worth_execution::MemoryLimitDenial::Capacity { .. }
        ))
    ));
    assert_eq!(memory.held, 0);
}

#[test]
fn overlay_lazy_partition_merge_is_canonical_and_deduplicated() {
    let base = Ids(vec![PartitionId(1), PartitionId(3), PartitionId(7)]);
    let staged = Ids(vec![PartitionId(2), PartitionId(3), PartitionId(8)]);
    let overlay = OverlayStateView::new(&base, &staged);
    let expected = vec![
        PartitionId(1),
        PartitionId(2),
        PartitionId(3),
        PartitionId(7),
        PartitionId(8),
    ];
    assert_eq!(overlay.partition_ids_iter().collect::<Vec<_>>(), expected);
    assert_eq!(overlay.partition_ids(), expected);
}
