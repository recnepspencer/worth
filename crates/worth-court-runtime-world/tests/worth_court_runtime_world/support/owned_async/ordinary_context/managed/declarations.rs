pub(crate) use std::num::NonZeroU64;
pub(crate) use {
    crate::owned_async::ordinary_context::fixture::admit_managed,
    crate::owned_async::ordinary_context::fixture::completion,
    crate::owned_async::ordinary_context::fixture::selected_fork,
    crate::owned_async::ordinary_context::fixture::source_commit,
    crate::owned_async::ordinary_context::fixture::AsyncSourceCourt,
};
pub(crate) use {
    worth_runtime_bridge::facade::BridgeOwnedAsyncManagedDenial,
    worth_runtime_bridge::facade::TruthCommitIdentity,
};
pub(crate) use {
    worth_signal::facade::ResourceManagedQueueDenialClass,
    worth_signal::facade::ResourceManagedQueueMutationReport,
};

pub(crate) fn assert_queue_denial(
    result: Result<ResourceManagedQueueMutationReport, BridgeOwnedAsyncManagedDenial>,
    expected: ResourceManagedQueueDenialClass,
) {
    let Err(worth_runtime_bridge::facade::BridgeOwnedAsyncManagedDenial::Signal(
        worth_signal::facade::branch::SignalOwnedAsyncQueueDenial::Queue(denied),
    )) = result
    else {
        panic!("actual native queue must supply the refusal");
    };
    assert_eq!(denied.class(), expected);
}
