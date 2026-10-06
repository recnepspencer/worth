use std::{cell::RefCell, collections::BinaryHeap, mem::size_of};

use worth_execution::{
    ChargedBytes, ExecutionResourceLease, ExecutionScan, MapKernelContext, MapKernelFailure,
    ScanDenial, ScanOutcome,
};
use worth_foundational::PartitionIdentity;

use super::{MergeHead, OrderedReductionStream, Reverse};
use crate::execution::{PacketBudgetDenial, PacketExecutionStop, RequestWorkBudget};

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
pub(crate) fn canonical_merge_streams_checked<K, T>(
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

pub(crate) fn canonical_merge_values_checked<K, T>(
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
    let owned_bytes = (streams.capacity() as u64)
        .saturating_mul(size_of::<OrderedReductionStream<K, T>>() as u64)
        .saturating_add(streams.iter().fold(0_u64, |bytes, stream| {
            bytes.saturating_add(stream.owned_allocation_capacity_bytes(&item_bytes))
        }));
    let identity = PartitionIdentity::new(1);
    let scan = ExecutionScan::try_from_ordered(
        vec![identity],
        vec![(
            identity,
            MergeInput {
                streams: RefCell::new(Some(streams)),
                owned_bytes,
            },
        )],
    )
    .map_err(|denial| {
        PacketExecutionStop::Admission(match denial {
            ScanDenial::IdentitiesNotCanonical => {
                worth_execution::MapDenial::ExpectedIdentitiesNotCanonical
            }
            ScanDenial::CoverageMismatch => worth_execution::MapDenial::CoverageMismatch,
            ScanDenial::MemoryOverflow => worth_execution::MapDenial::MemoryOverflow,
        })
    })?;
    let ceiling = lease.policy().budget().charged_memory_bytes() / 4;
    let outcome = crate::execution::run_with_remaining_request_work(
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
                    let streams = input.streams.borrow_mut().take().expect("one merge step");
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
            Ok(prefixes.pop().expect("one merge output").items)
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
            .expect("merge head references a current item");
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

#[cfg(test)]
mod tests {
    use std::{
        num::NonZeroUsize,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use worth_execution::{CancellationSource, CancellationToken, LeaseRequest};
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };

    use super::*;

    fn lease(work: u64, cancellation: CancellationToken) -> ExecutionResourceLease<'static> {
        crate::tests::support::test_execution_authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    ExecutionPosture::Automatic,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), 4 * 1024 * 1024, work),
                ),
                deadline: None,
                cancellation,
            })
            .unwrap()
    }

    fn streams() -> Vec<OrderedReductionStream<u64, u64>> {
        vec![OrderedReductionStream::new(
            (0..1_000).map(|index| (index, index)).collect(),
        )]
    }

    #[test]
    fn checked_merge_preserves_order_and_stops_inside_one_large_stream() {
        let expected = super::super::canonical_merge_streams(streams());
        let actual = canonical_merge_streams_checked(
            streams(),
            &lease(10_000, CancellationToken::new()),
            None,
            |_, _| 0,
            |_| 0,
        )
        .unwrap();
        assert_eq!(actual, expected);
        let values = canonical_merge_values_checked(
            streams(),
            &lease(10_000, CancellationToken::new()),
            None,
            |_, _| 0,
            |_| 0,
        )
        .unwrap();
        assert_eq!(
            values,
            expected
                .into_iter()
                .map(|(_, value)| value)
                .collect::<Vec<_>>()
        );

        let visited = AtomicUsize::new(0);
        let stop = canonical_merge_streams_checked(
            streams(),
            &lease(1_030, CancellationToken::new()),
            None,
            |_, _| 0,
            |_| {
                visited.fetch_add(1, Ordering::SeqCst);
                0
            },
        )
        .unwrap_err();
        assert!(visited.load(Ordering::SeqCst) > 1);
        assert!(visited.load(Ordering::SeqCst) < 1_000);
        assert!(matches!(
            stop,
            PacketExecutionStop::Execution {
                reason: worth_execution::MapStop::WorkExhausted { .. },
                ..
            }
        ));
    }

    #[test]
    fn checked_merge_observes_cancellation_between_heap_pops() {
        let cancellation = CancellationSource::new();
        let visits = AtomicUsize::new(0);
        let stop = canonical_merge_streams_checked(
            streams(),
            &lease(10_000, cancellation.token()),
            None,
            |_, _| 0,
            |_| {
                if visits.fetch_add(1, Ordering::SeqCst) == 40 {
                    cancellation.cancel();
                }
                0
            },
        )
        .unwrap_err();
        assert!(visits.load(Ordering::SeqCst) < 1_000);
        assert!(matches!(
            stop,
            PacketExecutionStop::Execution {
                reason: worth_execution::MapStop::Failure {
                    cause: worth_execution::MapKernelFailure::Stop(
                        worth_execution::MapKernelStop::Cancelled
                    ),
                    ..
                },
                ..
            }
        ));
    }
}
