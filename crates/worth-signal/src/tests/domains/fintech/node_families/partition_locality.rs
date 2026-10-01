use super::super::aspects::pricing_mask;
use super::{bounded, FintechRuntime, PartitionLocalityNodes};
use crate::facade::*;
use crate::tests::support::DependencyBatchBuilder;

pub(in crate::tests::domains::fintech) fn build_partition_locality_nodes(
    runtime: &mut FintechRuntime,
) -> PartitionLocalityNodes {
    let market_regions = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([]))
        .reads_aspects(pricing_mask())
        .produces_aspects(AspectMask::from([
            super::super::aspects::PRICE,
            super::super::aspects::RISK,
        ]))
        .partitioned_output()
        .build();
    let rates_partition = runtime
        .graph_mut()
        .node()
        .with_contract(
            NodeContract::wildcard()
                .with_bounded_inputs(BoundedSignalInputs::new([
                    DeclaredSignalInput::scoped(
                        market_regions,
                        super::super::aspects::PRICE,
                        PartitionSubscription::partition_and_detail("rates", "bucket-0"),
                    ),
                    DeclaredSignalInput::scoped(
                        market_regions,
                        super::super::aspects::PRICE,
                        PartitionSubscription::partition_and_detail("rates", "bucket-1"),
                    ),
                ]))
                .with_max_checked_result_heap_bytes(
                    super::super::evaluation::maximum_checked_result_heap_bytes(),
                ),
        )
        .reads_aspects(pricing_mask())
        .produces_aspects(super::super::aspects::PRICE)
        .tolerance(1)
        .build();
    let credit_partition = runtime
        .graph_mut()
        .node()
        .with_contract(
            NodeContract::wildcard()
                .with_bounded_inputs(BoundedSignalInputs::new([DeclaredSignalInput::scoped(
                    market_regions,
                    super::super::aspects::PRICE,
                    PartitionSubscription::whole_partition("credit"),
                )]))
                .with_max_checked_result_heap_bytes(
                    super::super::evaluation::maximum_checked_result_heap_bytes(),
                ),
        )
        .reads_aspects(pricing_mask())
        .produces_aspects(super::super::aspects::PRICE)
        .tolerance(1)
        .build();
    let rates_bucket_zero = runtime
        .graph_mut()
        .node()
        .with_contract(
            NodeContract::wildcard()
                .with_bounded_inputs(BoundedSignalInputs::new([DeclaredSignalInput::scoped(
                    market_regions,
                    super::super::aspects::PRICE,
                    PartitionSubscription::partition_and_detail("rates", "bucket-0"),
                )]))
                .with_max_checked_result_heap_bytes(
                    super::super::evaluation::maximum_checked_result_heap_bytes(),
                ),
        )
        .reads_aspects(pricing_mask())
        .produces_aspects(super::super::aspects::PRICE)
        .tolerance(1)
        .build();
    let coarse_book = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([
            (rates_partition, super::super::aspects::PRICE),
            (credit_partition, super::super::aspects::PRICE),
        ]))
        .reads_aspects(pricing_mask())
        .produces_aspects(super::super::aspects::PRICE)
        .tolerance(2)
        .build();

    let mut dependencies = DependencyBatchBuilder::new(runtime.graph_mut());
    dependencies
        .append_partition_detail_dependency(
            rates_partition,
            market_regions,
            super::super::aspects::PRICE,
            "rates",
            "bucket-0",
        )
        .unwrap()
        .append_partition_detail_dependency(
            rates_partition,
            market_regions,
            super::super::aspects::PRICE,
            "rates",
            "bucket-1",
        )
        .unwrap()
        .append_partition_dependency(
            credit_partition,
            market_regions,
            super::super::aspects::PRICE,
            "credit",
        )
        .unwrap()
        .append_partition_detail_dependency(
            rates_bucket_zero,
            market_regions,
            super::super::aspects::PRICE,
            "rates",
            "bucket-0",
        )
        .unwrap()
        .append_dependency(coarse_book, rates_partition, super::super::aspects::PRICE)
        .unwrap()
        .append_dependency(coarse_book, credit_partition, super::super::aspects::PRICE)
        .unwrap();
    dependencies.commit().unwrap();

    PartitionLocalityNodes {
        market_regions,
        rates_partition,
        credit_partition,
        rates_bucket_zero,
        coarse_book,
    }
}
