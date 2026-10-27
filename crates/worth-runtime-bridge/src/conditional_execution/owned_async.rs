mod admission;
mod completion;
mod completion_commit;
mod declaration;
mod installation;
mod managed_progress;
mod managed_queue;
mod managed_request;
mod managed_retry;
mod managed_revalidation;
mod retirement;
mod retry_revalidation;
mod supersession;

pub use completion_commit::BridgeOwnedAsyncCommittedCompletion;
pub use declaration::BridgeOwnedAsyncRequestResponseDeclaration;
pub(super) use installation::BridgeInstalledOwnedAsyncDeclaration;
pub use managed_request::{BridgeOwnedAsyncManagedDenial, BridgeOwnedAsyncManagedRequestAdmission};
pub use managed_revalidation::BridgeOwnedAsyncManagedRevalidation;

fn foreign_continuation_denial() -> crate::facade::BridgeAsyncCompletionRejection {
    crate::facade::BridgeAsyncCompletionRejection::new(
        crate::facade::BridgeAsyncCompletionRejectionKind::ForeignOwnerObservationAuthority,
        "owned async continuation belongs to another Bridge runtime",
    )
}
