//! The commit that ran a settled required root's producer.
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationProgramOutputSettlement,
    primary_graph::WorthQueryApplicationCommitReceipt,
};

/// A coupled root publishes inside its source edit, so that edit's receipt is
/// the producer's. The later demand reuses the output and carries no receipt
/// of its own; the settlement's root fulfillment names the source instead.
pub fn source_published_root<Query>(
    settled: &WorthQueryApplicationProgramOutputSettlement<Query>,
    source: &WorthQueryApplicationCommitReceipt,
) -> WorthQueryApplicationCommitReceipt {
    assert!(
        settled.root_receipt().is_none(),
        "reuse of a co-published root issues no separate producer receipt"
    );
    let fulfillment = settled
        .performed_source()
        .expect("a performed source retains its root fulfillment")
        .root_fulfillment();
    assert!(
        fulfillment.matches_source(source),
        "the root was fulfilled for this exact source edit"
    );
    assert_eq!(
        fulfillment.output_commit(),
        fulfillment.source_commit(),
        "the reused output is the one the source commit published"
    );
    source.clone()
}
