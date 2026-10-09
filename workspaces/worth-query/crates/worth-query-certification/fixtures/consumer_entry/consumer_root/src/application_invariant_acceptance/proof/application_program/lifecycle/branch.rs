//! Product branch creation for the real multi-root lifecycle courts.
use crate::ConsumerSchema;
pub(super) fn fork(
    application: &worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
        ConsumerSchema,
    >,
    source: worth_query_host::facade::product::WorthQueryProductBranch,
) -> worth_query_host::facade::product::WorthQueryProductBranch {
    application
        .branches()
        .fork(source)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .unwrap()
}
