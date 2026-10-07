use std::collections::{BTreeMap, BTreeSet};

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
    entities: BTreeSet<EntityId>,
    whole_graph: bool,
}

impl WorthQueryConditionalCommitWatchSet {
    pub(super) fn successor<Clock, Input>(
        predecessor: Option<&Self>,
        intents: &BTreeMap<String, WorthQueryReconstructedTemporalIntent<Clock, Input>>,
        lowering: &BridgeInstalledConditionalLowering,
        maximum_records: usize,
    ) -> Result<Self, &'static str> {
        let mut watch = predecessor.cloned().unwrap_or_default();
        watch.entities.extend(
            intents
                .values()
                .map(|intent| source_entity(intent.source_record())),
        );
        if watch.entities.len() > maximum_records {
            return Err("conditional commit watch capacity is exhausted");
        }
        watch.whole_graph |= (0..lowering.contract().dependency_count()).any(|ordinal| {
            matches!(
                lowering.dependency_locality(ordinal),
                Some(BridgeSemanticLocality::WholeLogicalGraph)
            )
        });
        Ok(watch)
    }

    pub(super) const fn entities(&self) -> &BTreeSet<EntityId> {
        &self.entities
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
