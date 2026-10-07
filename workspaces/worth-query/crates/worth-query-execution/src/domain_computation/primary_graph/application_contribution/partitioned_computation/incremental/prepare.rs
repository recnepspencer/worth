//! An incremental run's prepare: the membership, every item's key and route
//! and every partition's gathering, each carried from the retained run or
//! made again, in a full run's order over this run's own items and
//! partitions.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_execution::{ChargedBytes, PartitionItemId};
use worth_query_declaration::facade::application_program::{
    ApplicationComputationPartition, ApplicationFeature, ApplicationManagedComputation,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::super::request_execution::{QueryMemoryReservation, QueryRequestExecution};
use super::super::compute::Denial;
use super::super::gather_memory::GatheredMemory;
use super::super::items::{item_digests, planned_items, route_item};
use super::super::plan::PlanShape;
use super::super::remaining_work::RemainingWork;
use super::super::{
    InputValue, WorthQueryComputationPartitionMembers, WorthQueryComputationReader,
    WorthQueryPartitionedComputationDenial, WorthQueryPartitionedComputationOwner,
};
use super::next_tree::{NextPartitioning, PreparedIncremental};
use super::retained::RetainedCall;
use super::{enter, IncrementalRun, Reader, COMPARATOR};
use crate::domain_computation::primary_graph::application_attempt::ComputationRead;

impl<Key, Item, Reduced> IncrementalRun<Key, Item, Reduced>
where
    Key: ApplicationComputationPartition,
    Item: ApplicationComputationPartition + ChargedBytes,
{
    /// Carries or makes again the membership, then every item's key and
    /// every partition's gathering in ascending identity order over this
    /// run's items and partitions.
    ///
    /// The membership is made again when a fact it read moved, and its items
    /// are digested again with it. An item is keyed and routed again when it
    /// is new, its digest changed or a fact its key read moved; every other
    /// item keeps its route. A partition is gathered again when an item left
    /// or joined it, one of its items' digests changed, or a fact its
    /// gathering read moved. An item or partition the run no longer has
    /// charges nothing.
    ///
    /// A carried call charges what it charged, where a full run makes it,
    /// and its facts and reached entities enter the attempt as its reads. A
    /// call the remaining work would not pass is made instead, and fails as
    /// a full run fails. `remaining_work` is what the input's digest left of
    /// the declared work.
    ///
    /// `None` where an item routed again meets another key digest in its
    /// partition identity. Where that collision falls among the items
    /// depends on which items kept their routes, so the reader is taken back
    /// to the run's start and the caller makes the run in full, which names
    /// the collision as a fresh build does.
    pub(in super::super) fn prepare<Schema, Feature, Computation, Owner>(
        self,
        owner: &Owner,
        reader: &mut Reader<'_, '_, Schema, Owner::Operation>,
        execution: &QueryRequestExecution<'_>,
        input: &InputValue<Schema, Feature, Computation>,
        mut remaining_work: RemainingWork,
        declared_bytes: u64,
    ) -> Result<
        Option<PreparedIncremental<Key, Item, Reduced, Owner::Gathered>>,
        Denial<Schema, Feature, Computation, Owner>,
    >
    where
        Schema: ApplicationSchema,
        Feature: ApplicationFeature<Schema>,
        Computation: ApplicationManagedComputation<Schema, Feature, Partition = Key>,
        Owner: WorthQueryPartitionedComputationOwner<
            Schema,
            Feature,
            Computation,
            Item = Item,
            PartitionResult = Reduced,
        >,
    {
        let resource = WorthQueryPartitionedComputationDenial::Resource;
        let start = reader.run_start(&COMPARATOR);
        let retained = Arc::clone(&self.prior.typed);
        let carried_membership =
            !self.membership_moved && reader.carry(&COMPARATOR, &retained.membership.charge);
        let (items, digests, membership) = if carried_membership {
            enter(reader, &self.membership, ComputationRead::Membership);
            remaining_work
                .spend(Some(retained.membership.declared_units))
                .map_err(resource)?;
            let membership = Some(retained.membership.clone());
            (
                Arc::clone(&retained.items),
                Arc::clone(&retained.digests),
                membership,
            )
        } else {
            let (plan, charge) = reader.measured(ComputationRead::Membership, |reader| {
                owner.partitions(&mut WorthQueryComputationReader::lend(reader), input)
            });
            let PlanShape::Keyed(entries) = plan?.shape;
            let items = planned_items(entries)?;
            let before = remaining_work;
            let digests = item_digests(&items, &mut remaining_work, declared_bytes)?;
            let membership =
                charge
                    .zip(remaining_work.spent_since(before))
                    .map(|(charge, declared_units)| RetainedCall {
                        charge,
                        declared_units,
                    });
            (items, digests, membership)
        };
        // An item is new, or its value changed, exactly when its digest is
        // not the one retained for it.
        let changed = |item: &PartitionItemId| retained.digests.get(item) != digests.get(item);
        let rekeyed = |item: &PartitionItemId| changed(item) || self.moved_items.contains(item);
        let mut copy = None;
        let mut routing_memory = execution.reserve(0).map_err(resource)?;
        // The routing is copied, without the items routed again, before the
        // first route changes it: a run that routes nothing again shares it.
        let copied = |routing_memory: &mut QueryMemoryReservation| {
            retained.routing.kept(
                |item| items.contains_key(&item) && !rekeyed(&item),
                |bound| routing_memory.resize(bound),
            )
        };
        let mut item_keys = BTreeMap::new();
        let mut carried_items = BTreeSet::new();
        let mut fresh_keys = BTreeMap::new();
        // The partitions an item left or joined.
        let mut moved = BTreeSet::new();
        let mut measured = membership.is_some();
        for (item, value) in items.iter() {
            let read = ComputationRead::ItemKey(*item);
            let carried = retained
                .item_keys
                .get(item)
                .filter(|_| !rekeyed(item))
                .filter(|call| reader.carry(&COMPARATOR, &call.charge));
            if let Some(call) = carried {
                enter(reader, self.item_keys.get(item).into_iter().flatten(), read);
                remaining_work
                    .spend(Some(call.declared_units))
                    .map_err(resource)?;
                carried_items.insert(*item);
                item_keys.insert(*item, call.clone());
                continue;
            }
            let (key, charge) = reader.measured(read, |reader| {
                owner.partition_key(&mut WorthQueryComputationReader::lend(reader), input, value)
            });
            let key = key?;
            let before = remaining_work;
            let routing = match &mut copy {
                Some(routing) => routing,
                None => copy.insert(copied(&mut routing_memory).map_err(resource)?),
            };
            let routed = route_item(
                *item,
                &key,
                routing,
                &mut routing_memory,
                &mut remaining_work,
                declared_bytes,
            );
            let (partition, key_bytes) = match routed {
                Err(WorthQueryPartitionedComputationDenial::PartitionIdentityCollision {
                    ..
                }) => {
                    reader.restart(&COMPARATOR, start);
                    return Ok(None);
                }
                routed => routed?,
            };
            match charge.zip(remaining_work.spent_since(before)) {
                Some((charge, declared_units)) => {
                    item_keys.insert(
                        *item,
                        RetainedCall {
                            charge,
                            declared_units,
                        },
                    );
                }
                None => measured = false,
            }
            let left = retained.routing.partition_of(*item);
            if left != Some(partition) || changed(item) {
                moved.extend(left);
                moved.insert(partition);
            }
            fresh_keys
                .entry(partition)
                .or_insert_with(|| (Arc::new(key), key_bytes));
        }
        let mut removed = false;
        for item in retained
            .items
            .keys()
            .filter(|item| !items.contains_key(item))
        {
            moved.extend(retained.routing.partition_of(*item));
            removed = true;
        }
        if removed && copy.is_none() {
            copy = Some(copied(&mut routing_memory).map_err(resource)?);
        }
        let (routing, routing_memory) = match copy {
            Some(copy) => (Arc::new(copy), Some(routing_memory)),
            None => (Arc::clone(&retained.routing), None),
        };
        let mut prepared = PreparedIncremental::new(
            remaining_work,
            declared_bytes,
            self.basis,
            self.prior,
            NextPartitioning {
                items: Arc::clone(&items),
                digests,
                membership: membership.filter(|_| measured),
                item_keys,
                routing: Arc::clone(&routing),
                routing_memory,
                carried_membership,
                carried_items,
            },
        );
        for identity in routing.partitions() {
            let read = ComputationRead::Partition(identity);
            // An emptied old partition frees its compact identity for a
            // different full digest. Neither its typed key nor its result
            // belongs to the replacement, even when only a key fact moved.
            let prior = retained
                .partitions
                .get(&identity)
                .filter(|_| retained.routing.digest(identity) == routing.digest(identity));
            let unmoved = !moved.contains(&identity) && !self.marked.contains(&identity);
            if let Some(prior) = prior.filter(|_| unmoved) {
                if reader.carry(&COMPARATOR, &prior.gather) {
                    enter(
                        reader,
                        self.partitions.get(&identity).into_iter().flatten(),
                        read,
                    );
                    prepared.carried(identity, prior);
                    continue;
                }
            }
            // A partition with the same full digest keeps its key. Any other
            // partition holds
            // only items routed again, and takes its least item's key.
            let (key, key_bytes) = match prior {
                Some(prior) => (Arc::clone(&prior.key), prior.key_bytes),
                None => fresh_keys
                    .remove(&identity)
                    .expect("a partition no retained run held has an item routed again"),
            };
            let members = routing.members(identity).collect::<Arc<[_]>>();
            let mut memory = GatheredMemory::new(execution, declared_bytes)?;
            memory.before_gather(execution)?;
            let (gathered, charge) = reader.measured(read, |reader| {
                owner.gather(
                    &mut WorthQueryComputationReader::lend(reader),
                    input,
                    WorthQueryComputationPartitionMembers::new(identity, &key, &members, &items),
                )
            });
            let gathered = gathered.map_err(|denial| {
                WorthQueryPartitionedComputationDenial::gathering(identity, denial)
            })?;
            prepared.gathered(identity, key, key_bytes, members, gathered, charge, memory)?;
        }
        Ok(Some(prepared))
    }
}
