//! Retained routing, charge and fact fields compared across edit histories.
use super::*;
/// Every field of a retained run a later run reads.
#[derive(Debug, PartialEq)]
pub(super) struct Retained {
    pub(super) digests: Vec<(PartitionItemId, [u8; 32])>,
    pub(super) membership_units: u64,
    /// Each item key call's declared work.
    pub(super) item_keys: Vec<(PartitionItemId, u64)>,
    pub(super) routing: Vec<(PartitionIdentity, Vec<PartitionItemId>)>,
    /// Each partition's key and native kernel work.
    pub(super) partitions: Vec<(PartitionIdentity, u64, u64)>,
    pub(super) leaves: Vec<(PartitionIdentity, Option<u64>)>,
    pub(super) result: u64,
    pub(super) facts: Vec<(
        WorthQueryApplicationFactKey,
        WorthQueryApplicationObservedFact,
        ComputationFactReaders,
    )>,
}

pub(super) fn retained(sealed: &SealedComputationRun) -> Retained {
    let typed = sealed
        .state
        .typed
        .downcast_ref::<RetainedPartitions<Parity, Number, u64>>()
        .expect("the run retains its partitions");
    Retained {
        digests: typed
            .digests
            .iter()
            .map(|(item, digest)| (*item, *digest))
            .collect(),
        membership_units: typed.membership.declared_units,
        item_keys: typed
            .item_keys
            .iter()
            .map(|(item, call)| (*item, call.declared_units))
            .collect(),
        routing: typed
            .routing
            .partitions()
            .map(|partition| (partition, typed.routing.members(partition).collect()))
            .collect(),
        partitions: typed
            .partitions
            .iter()
            .map(|(identity, partition)| {
                let key = partition.key.0;
                (
                    *identity,
                    key,
                    typed.native_run.result(*identity).unwrap().charged_work(),
                )
            })
            .collect(),
        leaves: typed
            .partitions
            .keys()
            .map(|identity| (*identity, typed.tree.leaf(*identity).copied()))
            .collect(),
        result: *typed.tree.result(),
        facts: sealed
            .state
            .facts
            .facts()
            .map(|(key, fact, readers)| (key.clone(), fact.clone(), readers.clone()))
            .collect(),
    }
}
