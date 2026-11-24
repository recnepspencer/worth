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

    use super::super::checked_engine::{
        canonical_merge_streams_checked, canonical_merge_values_checked, extent,
    };
    use super::super::OrderedReductionStream;
    use crate::execution::read_only_packets::PacketExecutionStop;
    use worth_execution::ExecutionResourceLease;

    fn lease(work: u64, cancellation: CancellationToken) -> ExecutionResourceLease<'static> {
        crate::tests::support::test_execution_authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    ExecutionPosture::Automatic,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), None, Some(work)),
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

    /// Admission prices every retained item's heap key before the kernel runs,
    /// so kernel visits are the key callbacks after this declaration pass.
    fn declaration_key_visits() -> usize {
        let visits = AtomicUsize::new(0);
        let item_bytes = |_: &u64, _: &u64| 0;
        let key_clone_bytes = |_: &u64| {
            visits.fetch_add(1, Ordering::SeqCst);
            0
        };
        extent::declaration::<u64, u64, (u64, u64)>(&streams(), &item_bytes, &key_clone_bytes)
            .unwrap();
        visits.into_inner()
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
        let kernel_visits = visited.load(Ordering::SeqCst) - declaration_key_visits();
        assert!(kernel_visits > 1);
        assert!(kernel_visits < 1_000);
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
        let declaration = declaration_key_visits();
        let visits = AtomicUsize::new(0);
        let stop = canonical_merge_streams_checked(
            streams(),
            &lease(10_000, cancellation.token()),
            None,
            |_, _| 0,
            |_| {
                if visits.fetch_add(1, Ordering::SeqCst) == declaration + 40 {
                    cancellation.cancel();
                }
                0
            },
        )
        .unwrap_err();
        assert!(visits.load(Ordering::SeqCst) - declaration < 1_000);
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
