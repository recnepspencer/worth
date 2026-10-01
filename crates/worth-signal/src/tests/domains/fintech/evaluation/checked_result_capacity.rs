use std::sync::OnceLock;

use crate::data::retained_storage::{RetainedStorageMeasurement, RetainedStoragePreparation};
use crate::facade::{AspectVersion, NodeEvaluationResult};

/// Covers every identity and continuity form returned by this fixture's
/// instrument, portfolio, and partition evaluators. Values and indices use
/// their platform type extremes; the actual retained String capacities are
/// measured by the same owner used to check a completed evaluation.
pub(in crate::tests::domains::fintech) fn maximum_checked_result_heap_bytes() -> u64 {
    static MAXIMUM: OnceLock<u64> = OnceLock::new();
    *MAXIMUM.get_or_init(|| {
        let mut maximum = 0;
        let mut work = RetainedStoragePreparation::new(usize::MAX);
        for number in [0_u64, u64::MAX] {
            for index in [0_usize, usize::MAX] {
                let forms = [
                    (format!("eur-jpy-{number}"), "fx-cross"),
                    (
                        format!("normalized-{number}-{number}-{number}-{number}"),
                        "normalized",
                    ),
                    (format!("price-{number}-{number}"), "price"),
                    (format!("risk-{number}-{number}"), "risk"),
                    (format!("alert-{number}"), "alert"),
                    (format!("threshold-{number}"), "threshold"),
                    (format!("bucket-{index}-{number}"), "bucket-risk"),
                    (format!("scenario-{index}-{number}"), "scenario-risk"),
                    (format!("book-{index}-{number}-{number}"), "book-aggregate"),
                    (format!("desk-{index}-{number}-{number}"), "desk-aggregate"),
                    (
                        format!("scenario-agg-{index}-{number}"),
                        "scenario-aggregate",
                    ),
                    (format!("bucket-agg-{index}-{number}"), "bucket-aggregate"),
                    (format!("rates-partition-{number}"), "rates-partition"),
                    (format!("credit-partition-{number}"), "credit-partition"),
                    (format!("rates-bucket-zero-{number}"), "rates-bucket-zero"),
                    (format!("coarse-book-{number}"), "coarse-book"),
                ];
                for (identity, continuity) in forms {
                    let result = NodeEvaluationResult::from_version(AspectVersion::zero())
                        .with_output_identity(identity)
                        .with_continuity_token(continuity);
                    let charge = result
                        .retained_heap_charge(&mut work)
                        .expect("finite fintech result shape is measurable");
                    maximum = maximum.max(charge.bytes());
                }
            }
        }
        maximum
    })
}
