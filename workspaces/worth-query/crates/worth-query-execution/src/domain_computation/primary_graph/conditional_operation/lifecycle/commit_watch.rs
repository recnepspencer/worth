use std::collections::{BTreeMap, BTreeSet};

use worth_relational::facade::{
    identity::{EntityId, PartitionId},
    transactions::RecordRef,
};
use worth_runtime_bridge::facade::{BridgeInstalledConditionalLowering, BridgeSemanticLocality};

use crate::domain_computation::primary_graph::conditional_operation::temporal_reconstruction::WorthQueryReconstructedTemporalIntent;

/// Exact source records whose commits can affect one retained product
/// evaluation. The set survives intent completion for as long as the product
/// binding itself remains retained.
#[derive(Clone, Default)]
pub(in crate::domain_computation::primary_graph::conditional_operation) struct WorthQueryConditionalCommitWatchSet
{
    records: BTreeSet<RecordRef>,
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
        watch.records.extend(intents.values().map(|intent| {
            let record = intent.source_record();
            RecordRef::Entity(EntityId::new(
                PartitionId(record.partition_id()),
                record.local_slot(),
                record.generation(),
            ))
        }));
        if watch.records.len() > maximum_records {
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

    pub(super) fn records(&self) -> impl Iterator<Item = &RecordRef> {
        self.records.iter()
    }

    pub(super) const fn includes_whole_graph(&self) -> bool {
        self.whole_graph
    }
}
