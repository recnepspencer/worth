//! One replay allowance per actual adjacency, with every original reader.
use super::*;
use crate::domain_computation::primary_graph::{
    aggregate_observation::AggregateObservation, structural_walk::memory,
};
use std::cmp::Ordering;
use worth_execution::{ChargedBytes, ExecutionRequest, MapKernelContext};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

mod admission;

type Key = WorthQueryApplicationFactKey;

fn allowance(key: &Key) -> Option<usize> {
    match key {
        Key::Adjacency {
            maximum_work_units, ..
        } => Some(*maximum_work_units),
        _ => None,
    }
}
fn set_allowance(key: &mut Key, value: usize) {
    if let Key::Adjacency {
        maximum_work_units, ..
    } = key
    {
        *maximum_work_units = value;
    }
}
#[expect(
    clippy::disallowed_methods,
    reason = "Comparison uses declared semantic keys; type names label diagnostics only"
)]
fn locus_order(left: &Key, right: &Key) -> Ordering {
    match (left, right) {
        (
            Key::Adjacency {
                relation: a,
                anchor: b,
                direction: c,
                ..
            },
            Key::Adjacency {
                relation: x,
                anchor: y,
                direction: z,
                ..
            },
        ) => (a, b, c).cmp(&(x, y, z)),
        _ => left.cmp(right),
    }
}
fn checkpoint(context: &mut MapKernelContext<'_, '_>, units: u64) -> Result<(), Cause> {
    (if context.has_accounting() {
        context.checkpoint(units)
    } else {
        context.poll()
    })
    .map_err(|stop| Cause::from(worth_execution::WorkCeilingDenial::Stopped(stop)))
}
fn key_work(context: &mut MapKernelContext<'_, '_>, key: &Key, passes: u64) -> Result<(), Cause> {
    let bytes = match key {
        Key::Adjacency { relation, .. } => relation.len(),
        _ => 0,
    };
    let units = u64::try_from(bytes)
        .ok()
        .and_then(|n| n.checked_add(1))
        .and_then(|n| n.checked_mul(passes))
        .ok_or(Cause::WorkCounterOverflow)?;
    checkpoint(context, units)
}
fn levels(count: usize) -> u64 {
    u64::from(usize::BITS - count.leading_zeros()) + 1
}

fn fresh<'a>(
    handler: &'a BTreeSet<Key>,
    readers: &'a BTreeMap<Key, ComputationFactReads>,
) -> impl Iterator<Item = &'a Key> {
    let mut left = handler
        .iter()
        .filter(|key| allowance(key).is_some())
        .peekable();
    let mut right = readers
        .keys()
        .filter(|key| allowance(key).is_some())
        .peekable();
    std::iter::from_fn(move || match (left.peek(), right.peek()) {
        (Some(a), Some(b)) if a <= b => left.next(),
        (Some(_), Some(_)) | (None, Some(_)) => right.next(),
        (Some(_), None) => left.next(),
        (None, None) => None,
    })
}
impl ComputationFactAttribution {
    pub(in crate::domain_computation::primary_graph) fn normalize_adjacencies(
        &mut self,
        handler: &mut BTreeSet<Key>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Cause> {
        let result = execution.run_native_with_checkpoint(|context| {
            // Filtering visits only fresh maps. Four semantic passes cover the
            // two-way merge and its adjacent-locus comparisons in both scans.
            checkpoint(
                context,
                u64::try_from(
                    handler
                        .len()
                        .checked_add(self.readers.len())
                        .ok_or(Cause::WorkCounterOverflow)?,
                )
                .map_err(|_| Cause::WorkCounterOverflow)?,
            )?;
            for key in handler.iter().chain(self.readers.keys()) {
                if allowance(key).is_some() {
                    key_work(context, key, 4)?;
                }
            }
            let (mut conflict, mut carried_conflict) = (false, false);
            let mut previous: Option<&Key> = None;
            for key in fresh(handler, &self.readers) {
                if let Some(prior) = previous {
                    if locus_order(prior, key).is_eq() {
                        conflict |= allowance(prior) != allowance(key);
                    } else {
                        carried_conflict |= self.carried_adjacency_conflicts(prior, context)?;
                    }
                }
                previous = Some(key);
            }
            if let Some(last) = previous {
                carried_conflict |= self.carried_adjacency_conflicts(last, context)?;
            }
            conflict |= carried_conflict;
            if !conflict {
                return Ok(());
            }
            let bytes = self.normalization_bytes(handler, carried_conflict, context)?;
            self.normalization_memory =
                memory::reserve(execution, context.has_accounting(), bytes)?;
            if carried_conflict {
                // Only a real conflict leaves the fixed-key path. Existing
                // custody expands actual carried calls without relabeling fresh.
                self.materialize_carried();
            }
            let count = handler
                .len()
                .checked_add(self.readers.len())
                .ok_or(Cause::WorkCounterOverflow)?;
            let passes = levels(count)
                .checked_mul(11)
                .ok_or(Cause::WorkCounterOverflow)?;
            let mut maxima = BTreeMap::<Key, usize>::new();
            for key in handler.iter().chain(self.readers.keys()) {
                if let Some(value) = allowance(key) {
                    key_work(context, key, passes)?;
                    let mut locus = key.clone();
                    set_allowance(&mut locus, 0);
                    let maximum = maxima.entry(locus).or_insert(value);
                    *maximum = (*maximum).max(value);
                }
            }
            // Remove/reinsert only adjacency entries. Other key forms retain
            // their original nodes, meanings and fixed-key native backing.
            let handler_keys: Vec<_> = handler
                .iter()
                .filter(|key| allowance(key).is_some())
                .cloned()
                .collect();
            for key in handler_keys {
                key_work(context, &key, passes)?;
                let mut key = handler.take(&key).expect("original or canonical adjacency");
                canonicalize(&mut key, &maxima, context)?;
                key_work(context, &key, passes)?;
                handler.insert(key);
            }
            let reader_keys: Vec<_> = self
                .readers
                .keys()
                .filter(|key| allowance(key).is_some())
                .cloned()
                .collect();
            for key in reader_keys {
                key_work(context, &key, passes)?;
                let (mut key, mut reads) = self
                    .readers
                    .remove_entry(&key)
                    .expect("original or canonical adjacency");
                canonicalize(&mut key, &maxima, context)?;
                key_work(context, &key, passes)?;
                let target = self.readers.entry(key).or_default();
                let count = reads
                    .item_keys
                    .len()
                    .checked_add(reads.partitions.len())
                    .and_then(|n| n.checked_add(target.item_keys.len()))
                    .and_then(|n| n.checked_add(target.partitions.len()))
                    .ok_or(Cause::WorkCounterOverflow)?;
                checkpoint(
                    context,
                    u64::try_from(count)
                        .ok()
                        .and_then(|n| n.checked_add(1))
                        .and_then(|n| n.checked_mul(passes))
                        .ok_or(Cause::WorkCounterOverflow)?,
                )?;
                target.membership |= reads.membership;
                target.item_keys.append(&mut reads.item_keys);
                target.partitions.append(&mut reads.partitions);
            }
            AggregateObservation::checkpoint(context)
        });
        match result {
            Ok((value, _)) => value,
            Err(stop) => Err(stop.into()),
        }
    }

    fn carried_adjacency_conflicts(
        &self,
        key: &Key,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<bool, Cause> {
        let Some(native) = &self.native else {
            return Ok(false);
        };
        let metadata = &native.facts().metadata;
        let (mut low, mut high) = (0, metadata.len());
        // entry_at navigates an AVL path. Its actual ordinal navigation and
        // borrowed name comparison are admitted before every lookup.
        let navigation = levels(metadata.len())
            .checked_mul(2)
            .ok_or(Cause::WorkCounterOverflow)?;
        while low < high {
            let middle = low + (high - low) / 2;
            checkpoint(context, navigation)?;
            let (candidate, _) = metadata.entry_at(middle).expect("bounded ordinal");
            key_work(context, key, 1)?;
            key_work(context, candidate, 1)?;
            if locus_order(candidate, key).is_lt() {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        while low < metadata.len() {
            checkpoint(context, navigation)?;
            let (candidate, fact) = metadata.entry_at(low).expect("bounded ordinal");
            key_work(context, key, 1)?;
            key_work(context, candidate, 1)?;
            if !locus_order(candidate, key).is_eq() {
                break;
            }
            if allowance(candidate) != allowance(key) {
                for read in fact.readers.reads() {
                    AggregateObservation::checkpoint(context)?;
                    if self.is_carried(read) {
                        return Ok(true);
                    }
                }
            }
            low += 1;
        }
        Ok(false)
    }
}
fn canonicalize(
    key: &mut Key,
    maxima: &BTreeMap<Key, usize>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Cause> {
    if allowance(key).is_some() {
        key_work(
            context,
            key,
            levels(maxima.len())
                .checked_mul(11)
                .ok_or(Cause::WorkCounterOverflow)?,
        )?;
        set_allowance(key, 0);
        let maximum = *maxima.get(key).expect("actual adjacency locus");
        set_allowance(key, maximum);
    }
    Ok(())
}
