use super::super::installation::{
    WorthQueryConditionalRuntimeInstallationDenial as Denial,
    WorthQueryConditionalRuntimeInstallationDenialKind as Kind,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_runtime_bridge::facade::{BridgeInstalledConditionalLowering, BridgeSemanticLocality};

use crate::domain_computation::primary_graph::conditional_operation::temporal_reconstruction::WorthQueryReconstructedTemporalIntent;

/// Exact source entities whose commits can affect one retained product
/// evaluation. The set survives intent completion for as long as the product
/// binding itself remains retained. Whole-graph dependencies watch every
/// commit; the owner's retained touches decide relevance against this set.
#[derive(Clone, Default)]
pub(in crate::domain_computation::primary_graph::conditional_operation) struct WorthQueryConditionalCommitWatchSet
{
    entities: Option<Arc<WatchEntries>>,
    whole_graph: bool,
}

struct WatchEntries {
    entities: BTreeSet<EntityId>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl WorthQueryConditionalCommitWatchSet {
    pub(super) fn successor<Clock, Input>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        predecessor: Option<&Self>,
        intents: &BTreeMap<String, WorthQueryReconstructedTemporalIntent<Clock, Input>>,
        lowering: &BridgeInstalledConditionalLowering,
        maximum_records: Option<usize>,
    ) -> Result<Self, Denial> {
        let entities = Self::retain_entities(
            execution,
            predecessor,
            intents
                .values()
                .map(|intent| source_entity(intent.source_record())),
            intents.len(),
            maximum_records,
        )?;
        let mut watch = Self {
            entities: Some(entities),
            whole_graph: predecessor.is_some_and(|prior| prior.whole_graph),
        };
        watch.whole_graph |= (0..lowering.contract().dependency_count()).any(|ordinal| {
            matches!(
                lowering.dependency_locality(ordinal),
                Some(BridgeSemanticLocality::WholeLogicalGraph)
            )
        });
        Ok(watch)
    }

    fn retain_entities(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        predecessor: Option<&Self>,
        incoming: impl Iterator<Item = EntityId>,
        incoming_count: usize,
        maximum_records: Option<usize>,
    ) -> Result<Arc<WatchEntries>, Denial> {
        check_request(execution)?;
        let prior = predecessor.map(Self::entities);
        let count = prior
            .map_or(0, BTreeSet::len)
            .checked_add(incoming_count)
            .ok_or_else(capacity)?;
        // A B-tree insertion can allocate at most one full node per key.
        // Reserve the entire new root before cloning old or adding new keys;
        // the predecessor keeps its own shared grant until its last holder.
        let node = 11usize
            .checked_mul(std::mem::size_of::<EntityId>())
            .and_then(|bytes| bytes.checked_add(12 * std::mem::size_of::<usize>() + 64))
            .ok_or_else(capacity)?;
        let bytes = count
            .checked_mul(node)
            .and_then(|bytes| {
                bytes.checked_add(
                    std::mem::size_of::<WatchEntries>() + 4 * std::mem::size_of::<usize>(),
                )
            })
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(capacity)?;
        let memory = execution.allocation_policy().reserve_memory(bytes);
        check_request(execution)?;
        let memory = memory.map_err(|cause| {
            Denial::new(
                Kind::RetentionAllocation(cause),
                "conditional commit watch allocation",
            )
        })?;
        let mut entities = prior.cloned().unwrap_or_default();
        entities.extend(incoming);
        if maximum_records.is_some_and(|maximum| entities.len() > maximum) {
            return Err(capacity());
        }
        Ok(Arc::new(WatchEntries {
            entities,
            _memory: memory,
        }))
    }

    pub(super) fn entities(&self) -> &BTreeSet<EntityId> {
        static EMPTY: BTreeSet<EntityId> = BTreeSet::new();
        self.entities
            .as_ref()
            .map_or(&EMPTY, |entries| &entries.entities)
    }

    pub(super) const fn includes_whole_graph(&self) -> bool {
        self.whole_graph
    }
}

/// The Relational entity behind one Bridge record identity.
pub(in crate::domain_computation::primary_graph::conditional_operation) fn source_entity(
    record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
) -> EntityId {
    EntityId::new(
        PartitionId(record.partition_id()),
        record.local_slot(),
        record.generation(),
    )
}

fn capacity() -> Denial {
    Denial::new(
        Kind::RetentionCapacityExhausted,
        "conditional commit watch capacity is exhausted",
    )
}
#[cfg(test)]
mod tests;

fn check_request(execution: &worth_execution::ExecutionRequest<'_, '_>) -> Result<(), Denial> {
    use crate::domain_computation::primary_graph::{
        WorthQueryManagedComputationCheckpointDenial as Stop,
        WorthQueryManagedComputationInterruption as Interrupted,
    };
    use crate::domain_computation::{
        WorthQueryProviderSessionControlStopKind as Control,
        WorthQueryProviderSessionDenialKind as Refusal,
    };
    execution.check_live().map_err(|stop| {
        let cause = match crate::domain_computation::primary_graph::application_contribution::execution_denial::checkpoint_denial(stop) {
            Stop::Interrupted(Interrupted::Cancelled) => Err(Control::Cancelled),
            Stop::Interrupted(Interrupted::DeadlineExceeded) => Err(Control::TimedOut),
            Stop::Resource(denial) => Ok(Refusal::ExecutionResource { denial, partition_identity: None, policy_ancestor: None }),
            Stop::NestedPatternStopped => Ok(Refusal::ExecutionNestedPatternStopped { partition_identity: None }),
        };
        Denial::new(Kind::ReadExecution(cause), "conditional commit watch request stopped")
    })
}
