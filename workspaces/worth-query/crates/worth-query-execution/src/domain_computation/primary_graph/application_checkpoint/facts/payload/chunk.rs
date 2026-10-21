//! Native leaf custody and the exhaustive cost of a NEW deep Fact clone.
use super::Fact;
use super::FactPayload;
use super::FactStorage;
use std::sync::OnceLock;
use worth_execution::ExecutionArray;
use worth_execution::ExecutionMemoryReservation;

#[derive(Debug)]
pub(super) struct NativeFactChunk {
    // Facts and their inline backing drop before their additional copy funding.
    pub(super) facts: ExecutionArray<Fact>,
    pub(super) clone_bytes: Option<u64>,
    resident_heaps: Option<u64>,
    pub(super) _copied_heaps: Option<ExecutionMemoryReservation>,
    // Bounded leaf-local grant slots; never a source index or parent chain.
    _replacement_heaps: Option<ExecutionArray<OnceLock<ExecutionMemoryReservation>>>,
}
impl FactPayload {
    pub(in crate::domain_computation::primary_graph) fn copied_native(
        facts: ExecutionArray<Fact>,
        copied_heaps: Option<ExecutionMemoryReservation>,
        replacement_heaps: Option<ExecutionArray<OnceLock<ExecutionMemoryReservation>>>,
    ) -> Self {
        let mut chunk = NativeFactChunk {
            facts,
            clone_bytes: Some(0),
            resident_heaps: Some(0),
            _copied_heaps: copied_heaps,
            _replacement_heaps: replacement_heaps,
        };
        let length = chunk.facts.len();
        use worth_execution::ChargedBytes;
        for fact in chunk.facts.iter() {
            crate::domain_computation::primary_graph::fact_verification_observation::native_payload_size();
            chunk.clone_bytes = chunk
                .clone_bytes
                .zip(deep_clone_bytes(fact))
                .and_then(|(sum, bytes)| sum.checked_add(bytes));
            chunk.resident_heaps = chunk
                .resident_heaps
                .and_then(|sum| sum.checked_add(fact.additional_charged_bytes()));
        }
        let storage_bytes = (|| {
            u64::try_from(
                std::mem::size_of::<NativeFactChunk>()
                    .checked_add(2 * std::mem::size_of::<usize>())?
                    .checked_add(std::mem::align_of::<NativeFactChunk>())?,
            )
            .ok()
            .and_then(|bytes| {
                bytes.checked_add(u64::try_from(length.checked_mul(size_of::<Fact>())?).ok()?)
            })
            .and_then(|bytes| {
                let slots = chunk
                    ._replacement_heaps
                    .as_ref()
                    .map_or(0, |slots| slots.len());
                bytes.checked_add(
                    u64::try_from(
                        slots.checked_mul(size_of::<OnceLock<ExecutionMemoryReservation>>())?,
                    )
                    .ok()?,
                )
            })
        })();
        let resident_bytes = storage_bytes
            .zip(chunk.resident_heaps)
            .and_then(|(storage, heaps)| storage.checked_add(heaps));
        Self {
            storage: FactStorage::Native(std::sync::Arc::new(chunk)),
            start: 0,
            length,
            storage_bytes,
            resident_bytes,
            height: 0,
        }
    }
}

fn bytes<T>(values: &Vec<T>) -> Option<u64> {
    u64::try_from(values.capacity().checked_mul(size_of::<T>())?).ok()
}

/// Existing shared native endpoints, index definitions and snapshot custody
/// clone their owner, without allocating NEW element bodies here.
pub(super) fn deep_clone_bytes(fact: &Fact) -> Option<u64> {
    let size = |bytes| u64::try_from(bytes).ok();
    match fact {
        Fact::RetiredOutputEntity { read_locator, .. } => size(read_locator.capacity()),
        Fact::SourceAspectRevision { aspect, .. } | Fact::AspectPresence { aspect, .. } => {
            size(aspect.owned_allocation_capacity_bytes())
        }
        Fact::SourceFieldRevision { locator, .. }
        | Fact::AbsentField { locator, .. }
        | Fact::OptionalAspectAbsent { locator, .. } => {
            size(locator.owned_allocation_capacity_bytes())
        }
        Fact::Field { locator, value, .. } => size(locator.owned_allocation_capacity_bytes())?
            .checked_add(size(value.owned_allocation_capacity_bytes())?),
        Fact::Relation {
            matching_relations, ..
        } => bytes(matching_relations),
        Fact::Adjacency { relations, .. } => bytes(relations),
        Fact::IndexedEntitySelection {
            locator,
            value,
            candidates,
            ..
        } => size(locator.owned_allocation_capacity_bytes())?
            .checked_add(size(value.owned_allocation_capacity_bytes())?)?
            .checked_add(bytes(candidates)?),
        Fact::WorkflowInstanceCapacity { instances, .. } => bytes(instances),
        Fact::SourcePublication { .. }
        | Fact::SourceEntity { .. }
        | Fact::Entity { .. }
        | Fact::SourceAdjacencyRevision { .. }
        | Fact::WorkflowDefinitionPredecessor { .. }
        | Fact::WorkflowDefinitionCurrent { .. }
        | Fact::WorkflowHistoryBasis { .. } => Some(0),
    }
}
