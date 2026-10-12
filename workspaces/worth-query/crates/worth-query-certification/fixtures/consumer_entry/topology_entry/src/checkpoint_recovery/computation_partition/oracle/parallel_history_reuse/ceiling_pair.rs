//! Identical sibling branches judge reuse without changing installation authority.
use super::*;
use worth_query_host::facade::product::WorthQueryProductBranch;
pub(super) fn branches(app: &execution::Application) -> [WorthQueryProductBranch; 2] {
    std::array::from_fn(|_| {
        app.branches()
            .fork(app.current_world())
            .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
            .create()
            .unwrap()
    })
}
pub(super) fn assert_stop(
    retained: &OracleRun,
    reference: &OracleRun,
    model: &ReuseFacts,
    reuse: bool,
) {
    use worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial;
    let first = model
        .members()
        .min_by_key(|item| super::super::tree_work::identity(item.key))
        .unwrap();
    assert_eq!(
        retained.outcome,
        Err(WorthQueryPartitionedComputationDenial::Partition {
            partition: super::super::tree_work::identity(first.key),
            cause: WorthQueryComputationPartitionStop::Resource(
                WorthQueryManagedComputationResourceDenial::WorkExhausted
            ),
        }),
        "a ceiling inside carried work reports its exact canonical partition and work stop"
    );
    assert_eq!(
        retained.outcome, reference.outcome,
        "carried checkpoint ceiling preserves the typed stop"
    );
    assert_eq!(
        retained.calls.kernels,
        if reuse { 1 } else { 4 },
        "the ceiling row really carries three kernels"
    );
    assert_eq!(
        reference.calls.kernels, 4,
        "the counterpart really computes without reuse"
    );
}
