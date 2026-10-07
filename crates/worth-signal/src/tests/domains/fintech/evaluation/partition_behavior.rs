use super::FintechEvaluationView;
use crate::data::error::SignalError;
use crate::data::output::PartitionSubscription;
use crate::facade::{AspectVersion, NodeEvaluationResult};
use crate::logic::evaluation::EvaluationOutput;

use super::super::aspects::PRICE;

impl super::FintechEvaluationShape {
    pub(super) fn evaluate_partition_node<V: FintechEvaluationView>(
        &self,
        view: &mut V,
    ) -> Result<Option<EvaluationOutput>, SignalError> {
        let node = view.node();
        if node == self.partition.rates_partition {
            let bucket_zero = view
                .read_partitioned_aspect_version(
                    self.partition.market_regions,
                    PRICE,
                    PartitionSubscription::partition_and_detail("rates", "bucket-0"),
                )?
                .get(PRICE);
            let bucket_one = view
                .read_partitioned_aspect_version(
                    self.partition.market_regions,
                    PRICE,
                    PartitionSubscription::partition_and_detail("rates", "bucket-1"),
                )?
                .get(PRICE);
            let price = bucket_zero.saturating_add(bucket_one);
            return Ok(Some(
                view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                        PRICE, price,
                    )]))
                    .with_output_identity(format!("rates-partition-{price}"))
                    .with_continuity_token("rates-partition"),
                ),
            ));
        }

        if node == self.partition.credit_partition {
            let price = view
                .read_partitioned_aspect_version(
                    self.partition.market_regions,
                    PRICE,
                    PartitionSubscription::whole_partition("credit"),
                )?
                .get(PRICE);
            return Ok(Some(
                view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                        PRICE, price,
                    )]))
                    .with_output_identity(format!("credit-partition-{price}"))
                    .with_continuity_token("credit-partition"),
                ),
            ));
        }

        if node == self.partition.rates_bucket_zero {
            let price = view
                .read_partitioned_aspect_version(
                    self.partition.market_regions,
                    PRICE,
                    PartitionSubscription::partition_and_detail("rates", "bucket-0"),
                )?
                .get(PRICE);
            return Ok(Some(
                view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                        PRICE, price,
                    )]))
                    .with_output_identity(format!("rates-bucket-zero-{price}"))
                    .with_continuity_token("rates-bucket-zero"),
                ),
            ));
        }

        if node == self.partition.coarse_book {
            let rates = view
                .read_aspect_version(self.partition.rates_partition, PRICE)?
                .get(PRICE);
            let credit = view
                .read_aspect_version(self.partition.credit_partition, PRICE)?
                .get(PRICE);
            let total = rates + credit;
            return Ok(Some(
                view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                        PRICE, total,
                    )]))
                    .with_output_identity(format!("coarse-book-{total}"))
                    .with_continuity_token("coarse-book"),
                ),
            ));
        }
        Ok(None)
    }
}
