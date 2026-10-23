//! The items a run partitions and their routes, met the same way by a full
//! run and an incremental one: named once in item identity order, known to a
//! producer's run by their digests, and each routed by its key's digest.

use std::collections::BTreeMap;
use std::sync::Arc;

use worth_execution::PartitionItemId;
use worth_foundational::facade::PartitionIdentity;
use worth_query_declaration::facade::application_operation::{
    application_computation_input_digest, application_computation_input_digest_polled,
    application_computation_item_digest, application_computation_item_digest_polled,
    application_computation_partition_digest_polled, application_computation_partition_identity,
};
use worth_query_declaration::facade::application_program::{
    ApplicationComputationInput, ApplicationComputationPartition,
};

use super::super::request_execution::QueryRequestExecution;
use super::preparation_work::{charge_preparation, encode_identity, PreparationWork};
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

/// Every item's digest, in ascending item identity order, admitting each
/// encoding through the original request. Only a producer's run digests its
/// items: a full run and an incremental one each digest every item once per
/// run of the membership, and a run that carries the membership spends what
/// its digests spent.
pub(super) fn item_digests<Item, Stopped>(
    items: &BTreeMap<PartitionItemId, Item>,
    execution: &QueryRequestExecution<'_>,
    work: &mut PreparationWork,
) -> Result<ItemDigests, WorthQueryPartitionedComputationDenial<Stopped>>
where
    Item: ApplicationComputationPartition,
{
    items
        .iter()
        .map(|(item, value)| {
            encode_identity(
                execution,
                work,
                |meter| {
                    application_computation_item_digest(value, &mut |charge| meter.admit(charge))
                },
                |poll| application_computation_item_digest_polled(value, &mut || poll()),
                |denial| WorthQueryPartitionedComputationDenial::ItemNotEncodable {
                    item: *item,
                    denial,
                },
            )
            .map(|digest| (*item, digest))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()
        .map(Arc::new)
}

/// Derives `item`'s partition from its `key` and routes it, spending the
/// key's encoding through the original request. New routing cells
/// hold their native grants. Returns the native canonical partition identity.
pub(super) fn route_item<Key, Stopped>(
    item: PartitionItemId,
    key: &Key,
    routing: &mut ComputationPartitionRouting,
    execution: &QueryRequestExecution<'_>,
    work: &mut PreparationWork,
) -> Result<PartitionIdentity, WorthQueryPartitionedComputationDenial<Stopped>>
where
    Key: ApplicationComputationPartition,
{
    let derived = encode_identity(
        execution,
        work,
        |meter| {
            application_computation_partition_identity(key, &mut |charge| meter.admit(charge))
                .map(|identity| *identity.digest())
        },
        |poll| application_computation_partition_digest_polled(key, &mut || poll()),
        |denial| WorthQueryPartitionedComputationDenial::KeyNotEncodable { item, denial },
    )?;
    let (partition, routed) = routing.route_in(
        item,
        derived,
        execution
            .allocation_policy()
            .map_err(WorthQueryPartitionedComputationDenial::Resource)?,
    )?;
    let units = if execution.has_accounting() {
        routed.units()
    } else {
        None
    };
    charge_preparation(execution, work, units)?;
    Ok(partition)
}

/// The producer input's digest has the same original-request encoding boundary.
pub(super) fn input_digest<Input, Stopped>(
    input: &Input::Value,
    execution: &QueryRequestExecution<'_>,
    work: &mut PreparationWork,
) -> Result<[u8; 32], WorthQueryPartitionedComputationDenial<Stopped>>
where
    Input: ApplicationComputationInput,
{
    encode_identity(
        execution,
        work,
        |meter| {
            application_computation_input_digest::<Input, _, _>(input, &mut |charge| {
                meter.admit(charge)
            })
        },
        |poll| application_computation_input_digest_polled::<Input, _, _>(input, &mut || poll()),
        WorthQueryPartitionedComputationDenial::InputNotEncodable,
    )
}
