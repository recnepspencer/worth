//! The publication a delivered root's original producer receipt belongs to.
use worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt;

pub(super) fn commit(
    receipt: Option<&WorthQueryApplicationCommitReceipt>,
) -> impl PartialEq + std::fmt::Debug + use<> {
    receipt
        .expect("a delivered committed root keeps its original producer receipt")
        .committed_product_publication()
        .composite_commit()
        .clone()
}
