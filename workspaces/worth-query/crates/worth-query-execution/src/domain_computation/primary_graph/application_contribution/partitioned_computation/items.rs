//! The items a run partitions and their routes, met the same way by a full
//! run and an incremental one: named once in item identity order, known to a
//! producer's run by their digests, and each routed by its key's digest.

use std::collections::BTreeMap;
use std::sync::Arc;

use worth_execution::PartitionItemId;
use worth_foundational::facade::PartitionIdentity;
use worth_query_declaration::facade::application_operation::{
    application_computation_item_digest, application_computation_partition_identity,
    ApplicationComputationPartitionIdentityDenial,
};
use worth_query_declaration::facade::application_program::ApplicationComputationPartition;

use super::super::request_execution::QueryMemoryReservation;
use super::super::WorthQueryManagedComputationResourceDenial;
use super::installed::encoding_resource_denial;
use super::remaining_work::{EncodingMeter, RemainingWork};
use super::routing::ComputationPartitionRouting;
use super::WorthQueryPartitionedComputationDenial;

/// The items of one run by identity.
pub(super) type Items<Item> = Arc<BTreeMap<PartitionItemId, Item>>;

/// Each item's digest by identity.
pub(super) type ItemDigests = Arc<BTreeMap<PartitionItemId, [u8; 32]>>;

/// The plan's items in ascending identity order. An identity the plan names
/// twice is denied before any item is digested or keyed; when several are,
/// the least is named.
pub(super) fn planned_items<Item, Stopped>(
    mut entries: Vec<(PartitionItemId, Item)>,
) -> Result<Items<Item>, WorthQueryPartitionedComputationDenial<Stopped>> {
    entries.sort_by_key(|(item, _)| *item);
    if let Some(pair) = entries.windows(2).find(|pair| pair[0].0 == pair[1].0) {
        return Err(WorthQueryPartitionedComputationDenial::DuplicateItem { item: pair[0].0 });
    }
    Ok(Arc::new(entries.into_iter().collect()))
}

/// Every item's digest, in ascending item identity order, spending each
/// encoding's work from the declared work. Only a producer's run digests its
/// items: a full run and an incremental one each digest every item once per
/// run of the membership, and a run that carries the membership spends what
/// its digests spent.
pub(super) fn item_digests<Item, Stopped>(
    items: &BTreeMap<PartitionItemId, Item>,
    remaining_work: &mut RemainingWork,
    declared_bytes: u64,
) -> Result<ItemDigests, WorthQueryPartitionedComputationDenial<Stopped>>
where
    Item: ApplicationComputationPartition,
{
    items
        .iter()
        .map(|(item, value)| {
            let mut meter = EncodingMeter::new(remaining_work, declared_bytes);
            application_computation_item_digest(value, &mut |charge| meter.admit(charge))
                .map(|digest| (*item, digest))
                .map_err(|denial| match denial {
                    ApplicationComputationPartitionIdentityDenial::Key(denial) => {
                        WorthQueryPartitionedComputationDenial::ItemNotEncodable {
                            item: *item,
                            denial,
                        }
                    }
                    denial => encoding_resource_denial(denial),
                })
        })
        .collect::<Result<BTreeMap<_, _>, _>>()
        .map(Arc::new)
}

/// Derives `item`'s partition from its `key` and routes it, spending the
/// key's encoding and the route from the declared work. `memory` holds what
/// the routing retains. Returns the partition and the key's canonical
/// encoding length.
pub(super) fn route_item<Key, Stopped>(
    item: PartitionItemId,
    key: &Key,
    routing: &mut ComputationPartitionRouting,
    memory: &mut QueryMemoryReservation,
    remaining_work: &mut RemainingWork,
    declared_bytes: u64,
) -> Result<(PartitionIdentity, u64), WorthQueryPartitionedComputationDenial<Stopped>>
where
    Key: ApplicationComputationPartition,
{
    let mut meter = EncodingMeter::new(remaining_work, declared_bytes);
    let derived =
        application_computation_partition_identity(key, &mut |charge| meter.admit(charge))
            .map_err(|denial| match denial {
                ApplicationComputationPartitionIdentityDenial::Key(denial) => {
                    WorthQueryPartitionedComputationDenial::KeyNotEncodable { item, denial }
                }
                denial => encoding_resource_denial(denial),
            })?;
    let (partition, routed) =
        routing.route(item, *derived.digest(), |bound| memory.resize(bound))?;
    remaining_work
        .spend(routed.units())
        .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
    let key_bytes = u64::try_from(derived.work().encoded_bytes()).map_err(|_| {
        WorthQueryPartitionedComputationDenial::Resource(
            WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted,
        )
    })?;
    Ok((partition, key_bytes))
}
