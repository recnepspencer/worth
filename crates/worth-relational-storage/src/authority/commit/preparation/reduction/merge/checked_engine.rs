use std::{cell::RefCell, collections::BinaryHeap, mem::size_of};

use worth_execution::{
    ChargedBytes, ExecutionResourceLease, MapKernelContext, MapKernelFailure, ScanOutcome,
};
use worth_foundational::PartitionIdentity;

use super::{MergeHead, OrderedReductionStream, Reverse};
use crate::execution::read_only_packets::PacketBudgetDenial;
use crate::execution::read_only_packets::PacketExecutionStop;
use crate::execution::request_work::RequestWorkBudget;

struct MergeInput<K, T> {
    streams: RefCell<Option<Vec<OrderedReductionStream<K, T>>>>,
    owned_bytes: u64,
}

impl<K, T> ChargedBytes for MergeInput<K, T> {
    fn additional_charged_bytes(&self) -> u64 {
        self.owned_bytes
    }
}

struct MergeOutput<O> {
    items: Vec<O>,
    owned_bytes: u64,
}

impl<O> ChargedBytes for MergeOutput<O> {
    fn additional_charged_bytes(&self) -> u64 {
        self.owned_bytes
    }
}

/// Merge canonical worker streams under the same request account as their map.
/// Nested item/key bytes are supplied by each domain's declared ownership model.
pub fn canonical_merge_streams_checked<K, T>(
    streams: Vec<OrderedReductionStream<K, T>>,
    lease: &ExecutionResourceLease<'_>,
    work_budget: Option<&RequestWorkBudget>,
    item_bytes: impl Fn(&K, &T) -> u64,
    key_clone_bytes: impl Fn(&K) -> u64,
) -> Result<Vec<(K, T)>, PacketExecutionStop>
where
    K: Ord + Clone,
{
    canonical_merge_projected(
        streams,
        lease,
        work_budget,
        item_bytes,
        key_clone_bytes,
        |key, item| (key, item),
    )
}

pub fn canonical_merge_values_checked<K, T>(
    streams: Vec<OrderedReductionStream<K, T>>,
    lease: &ExecutionResourceLease<'_>,
    work_budget: Option<&RequestWorkBudget>,
    item_bytes: impl Fn(&K, &T) -> u64,
    key_clone_bytes: impl Fn(&K) -> u64,
) -> Result<Vec<T>, PacketExecutionStop>
where
    K: Ord + Clone,
{
    canonical_merge_projected(
        streams,
        lease,
        work_budget,
        item_bytes,
        key_clone_bytes,
        |_, item| item,
    )
}

fn canonical_merge_projected<K, T, O>(
    streams: Vec<OrderedReductionStream<K, T>>,
    lease: &ExecutionResourceLease<'_>,
    work_budget: Option<&RequestWorkBudget>,
    item_bytes: impl Fn(&K, &T) -> u64,
    key_clone_bytes: impl Fn(&K) -> u64,
    project: impl Fn(K, T) -> O,
) -> Result<Vec<O>, PacketExecutionStop>
where
    K: Ord + Clone,
{
    let (owned_bytes, declared_bytes) =
        extent::declaration::<K, T, O>(&streams, &item_bytes, &key_clone_bytes)?;
    let identity = PartitionIdentity::new(1);
    let scan = crate::execution::ordered_scan::admit_ordered_scan(vec![(
        identity,
        MergeInput {
            streams: RefCell::new(Some(streams)),
            owned_bytes,
        },
    )])?;
    // The merge declares the extent it will hold; the lease's own memory
    // policy admits that declaration, not a fixed share of the policy.
    let ceiling = declared_bytes;
    let outcome = crate::execution::request_work::run_with_remaining_request_work(
        lease,
        work_budget,
        |child_lease| {
            scan.run(
                Some(child_lease),
                (),
                0,
                ceiling,
                0,
                ceiling,
                |_, input, context| {
                    context.checkpoint(0)?;
                    let streams = input.streams.borrow_mut().take().expect(/* Proof: This private adapter submits exactly one packet; successful completion invokes its FnOnce step once and returns exactly one output. Stops return before extraction. */ "one merge step");
                    let (merged, owned_bytes) = merge_checked(
                        streams,
                        context,
                        ceiling,
                        &item_bytes,
                        &key_clone_bytes,
                        &project,
                    )?;
                    if owned_bytes > ceiling {
                        return Err(MapKernelFailure::ResultCapacityExceeded);
                    }
                    Ok((
                        (),
                        MergeOutput {
                            items: merged,
                            owned_bytes,
                        },
                    ))
                },
            )
        },
        ScanOutcome::report,
    );
    match outcome {
        ScanOutcome::Complete { mut prefixes, .. } => {
            Ok(prefixes.pop().expect(/* Proof: This private adapter submits exactly one packet; successful completion invokes its FnOnce step once and returns exactly one output. Stops return before extraction. */ "one merge output").items)
        }
        ScanOutcome::Stopped {
            boundary, reason, ..
        } => Err(PacketExecutionStop::Execution { boundary, reason }),
    }
}

fn merge_checked<K, T, O>(
    streams: Vec<OrderedReductionStream<K, T>>,
    context: &mut MapKernelContext<'_, '_>,
    ceiling: u64,
    item_bytes: &impl Fn(&K, &T) -> u64,
    key_clone_bytes: &impl Fn(&K) -> u64,
    project: &impl Fn(K, T) -> O,
) -> Result<(Vec<O>, u64), MapKernelFailure<PacketBudgetDenial>>
where
    K: Ord + Clone,
{
    let stream_count = streams.len();
    let mut item_count = 0_usize;
    for stream in &streams {
        context.checkpoint(1).map_err(MapKernelFailure::Stop)?;
        item_count = item_count
            .checked_add(stream.items.len())
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
    }
    let result_elements_bytes = (item_count as u64).saturating_mul(size_of::<O>() as u64);
    let mut result_bytes = result_elements_bytes;
    for (key, item) in streams.iter().flat_map(|stream| &stream.items) {
        context.checkpoint(1).map_err(MapKernelFailure::Stop)?;
        result_bytes = result_bytes.saturating_add(item_bytes(key, item));
        if result_bytes > ceiling {
            return Err(MapKernelFailure::ResultCapacityExceeded);
        }
    }
    if result_bytes > ceiling {
        return Err(MapKernelFailure::ResultCapacityExceeded);
    }
    let fixed_scratch = (stream_count as u64).saturating_mul(
        (size_of::<std::vec::IntoIter<(K, T)>>()
            + size_of::<Option<(K, T)>>()
            + size_of::<MergeHead<K>>()) as u64,
    );
    if fixed_scratch > ceiling {
        return Err(MapKernelFailure::Domain(
            PacketBudgetDenial::ScratchCapacityExceeded,
        ));
    }
    let mut iterators = Vec::new();
    iterators
        .try_reserve_exact(stream_count)
        .map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
    iterators.extend(streams.into_iter().map(|stream| stream.items.into_iter()));
    let mut current = Vec::new();
    current
        .try_reserve_exact(stream_count)
        .map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
    let mut heap = BinaryHeap::new();
    heap.try_reserve(stream_count)
        .map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
    let mut cloned_key_bytes = 0_u64;
    for (stream_index, iterator) in iterators.iter_mut().enumerate() {
        context.checkpoint(1).map_err(MapKernelFailure::Stop)?;
        let next = iterator.next();
        if let Some((key, item)) = next {
            cloned_key_bytes = cloned_key_bytes.saturating_add(key_clone_bytes(&key));
            if fixed_scratch.saturating_add(cloned_key_bytes) > ceiling {
                return Err(MapKernelFailure::Domain(
                    PacketBudgetDenial::ScratchCapacityExceeded,
                ));
            }
            heap.push(MergeHead {
                key: Reverse(key.clone()),
                stream_index,
            });
            current.push(Some((key, item)));
        } else {
            current.push(None);
        }
    }
    let mut merged = Vec::new();
    merged
        .try_reserve_exact(item_count)
        .map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
    result_bytes = (merged.capacity() as u64)
        .saturating_mul(size_of::<O>() as u64)
        .saturating_add(result_bytes.saturating_sub(result_elements_bytes));
    if result_bytes > ceiling {
        return Err(MapKernelFailure::ResultCapacityExceeded);
    }
    while let Some(head) = heap.pop() {
        context.checkpoint(1).map_err(MapKernelFailure::Stop)?;
        cloned_key_bytes = cloned_key_bytes.saturating_sub(key_clone_bytes(&head.key.0));
        let stream_index = head.stream_index;
        let (key, item) = current[stream_index]
            .take()
            .expect(/* Proof: A heap entry is inserted only together with its current stream item; popping removes that entry before taking the item, then installs both successors together. */ "merge head references a current item");
        merged.push(project(key, item));
        if let Some((next_key, next_item)) = iterators[stream_index].next() {
            cloned_key_bytes = cloned_key_bytes.saturating_add(key_clone_bytes(&next_key));
            if fixed_scratch.saturating_add(cloned_key_bytes) > ceiling {
                return Err(MapKernelFailure::Domain(
                    PacketBudgetDenial::ScratchCapacityExceeded,
                ));
            }
            heap.push(MergeHead {
                key: Reverse(next_key.clone()),
                stream_index,
            });
            current[stream_index] = Some((next_key, next_item));
        }
    }
    Ok((merged, result_bytes))
}

#[path = "checked/extent.rs"]
pub mod extent;
