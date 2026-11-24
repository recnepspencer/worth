//! Exact retained record images, including payloads at deleted slots.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::{DurableCheckpoint, PartitionCheckpointImage},
    identity::{EntityId, PartitionId, RelationId},
    runtime::RelationalRuntime,
    storage::RecordLifecycleState,
};

pub(super) struct RetainedRecords(DurableCheckpoint);

impl RetainedRecords {
    pub(super) fn capture(
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
    ) -> Self {
        Self(
            runtime
                .durability_authority()
                .checkpoint(execution)
                .unwrap(),
        )
    }

    fn partition(&self, id: PartitionId) -> &PartitionCheckpointImage {
        self.0
            .partition_images
            .iter()
            .find(|image| image.partition_id == id)
            .unwrap()
    }

    pub(super) fn entity(&self, id: EntityId) -> (bool, Vec<u8>) {
        let arena = &self.partition(id.partition_id).entity_arena;
        let index = slot(&arena.slots, id.local_slot_value());
        let generation = arena.generations[index];
        assert_eq!(generation, id.generation_value());
        let extra = &arena.extra[index];
        let payload = rmp_serde::to_vec(&(
            id,
            arena.kind_ids[index],
            generation,
            extra.lineage_id,
            &extra.authoritative_aspect_state,
        ))
        .unwrap();
        (
            arena.lifecycle[index] == RecordLifecycleState::Live,
            payload,
        )
    }

    pub(super) fn relation(&self, id: RelationId) -> (bool, Vec<u8>) {
        let arena = &self.partition(id.partition_id).relation_arena;
        let index = slot(&arena.slots, id.local_slot_value());
        let generation = arena.generations[index];
        assert_eq!(generation, id.generation_value());
        let extra = &arena.extra[index];
        let payload = rmp_serde::to_vec(&(
            id,
            arena.kind_ids[index],
            generation,
            &extra.endpoints,
            &extra.authoritative_aspect_state,
        ))
        .unwrap();
        (
            arena.lifecycle[index] == RecordLifecycleState::Live,
            payload,
        )
    }
}

fn slot(slots: &[u64], absolute: u64) -> usize {
    if slots.is_empty() {
        usize::try_from(absolute).unwrap()
    } else {
        slots.binary_search(&absolute).unwrap()
    }
}
