//! Retained routing, charge and fact fields compared across edit histories.
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::reroute::*;
/// Every field of a retained run a later run reads.
#[derive(Debug, PartialEq)]
pub struct Retained {
    pub digests: Vec<(PartitionItemId, [u8; 32])>,
    pub membership_units: Option<u64>,
    /// Each item key call's declared work.
    pub item_keys: Vec<(PartitionItemId, Option<u64>)>,
    pub routing: Vec<(PartitionIdentity, Vec<PartitionItemId>)>,
    /// Each partition's key and native kernel work.
    pub partitions: Vec<(PartitionIdentity, u64, u64)>,
    pub leaves: Vec<(PartitionIdentity, Option<u64>)>,
    pub result: u64,
    pub facts: Vec<(
        WorthQueryApplicationFactKey,
        WorthQueryApplicationObservedFact,
        ComputationFactReaders,
    )>,
}

pub fn retained(sealed: &SealedComputationRun) -> Retained {
    let typed = (*(*sealed.state_for_test()).typed_for_test())
        .downcast_ref::<RetainedPartitions<Parity, Number, u64>>()
        .expect("the run retains its partitions");
    Retained {
        digests: (*typed.digests_for_test())
            .iter()
            .map(|(item, digest)| (*item, *digest))
            .collect(),
        membership_units: (*(*typed.membership_for_test()).declared_units_for_test()),
        item_keys: (*typed.item_keys_for_test())
            .iter()
            .map(|(item, call)| (*item, (*call.declared_units_for_test())))
            .collect(),
        routing: (*typed.routing_for_test())
            .partitions_for_test()
            .map(|partition| {
                (
                    partition,
                    (*typed.routing_for_test())
                        .members_for_test(partition)
                        .collect(),
                )
            })
            .collect(),
        partitions: (*typed.partitions_for_test())
            .iter_for_test()
            .map(|(identity, partition)| {
                let key = (*partition.key_for_test()).0;
                (
                    *identity,
                    key,
                    (*typed.native_run_for_test())
                        .result(*identity)
                        .unwrap()
                        .charged_work(),
                )
            })
            .collect(),
        leaves: (*typed.partitions_for_test())
            .keys_for_test()
            .map(|identity| (*identity, (*typed.tree_for_test()).leaf(*identity).copied()))
            .collect(),
        result: *(*typed.tree_for_test()).result(),
        facts: (*(*sealed.state_for_test()).facts_for_test())
            .facts_for_test()
            .map(|(key, fact, readers)| (key.clone(), fact.clone(), readers.clone()))
            .collect(),
    }
}
