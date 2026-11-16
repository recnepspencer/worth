mod admission;
mod clock;
mod completion;
mod completion_commit;
mod declaration;
pub(in crate::conditional_execution) mod definition_source;
mod initial_request;
mod installation;
mod installed_view;
mod managed_progress;
mod managed_queue;
mod managed_request;
mod managed_retry;
mod managed_revalidation;
mod process_local_release;
mod retirement;
mod supersession;
mod wake_observation_for_test;

pub use completion_commit::BridgeOwnedAsyncCommittedCompletion;
pub use declaration::{
    BridgeOwnedAsyncDeclarationDescription, BridgeOwnedAsyncRequestResponseDeclaration,
    BridgeOwnedAsyncRetention,
};
pub(super) use installation::BridgeInstalledOwnedAsyncDeclaration;
pub use managed_request::{BridgeOwnedAsyncManagedDenial, BridgeOwnedAsyncManagedRequestAdmission};
pub use managed_revalidation::BridgeOwnedAsyncManagedRevalidation;
pub use process_local_release::{
    BridgeOwnedAsyncReleasedResource, BridgeOwnedAsyncResourceReleaseRefusal,
};

fn foreign_continuation_denial() -> crate::facade::BridgeAsyncCompletionRejection {
    crate::facade::BridgeAsyncCompletionRejection::new(
        crate::facade::BridgeAsyncCompletionRejectionKind::ForeignOwnerObservationAuthority,
        "owned async continuation belongs to another Bridge runtime",
    )
}
#[cfg(test)]
#[path = "owned_async_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
