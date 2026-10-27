mod admission;
mod completion;
mod completion_commit;
mod declaration;
mod installation;
mod managed_queue;
mod managed_request;
mod retirement;
mod retry_revalidation;
mod supersession;

pub use completion_commit::BridgeOwnedAsyncCommittedCompletion;
pub use declaration::BridgeOwnedAsyncRequestResponseDeclaration;
pub(super) use installation::BridgeInstalledOwnedAsyncDeclaration;
pub use managed_request::{BridgeOwnedAsyncManagedDenial, BridgeOwnedAsyncManagedRequestAdmission};

fn foreign_continuation_denial() -> crate::facade::BridgeAsyncCompletionRejection {
    crate::facade::BridgeAsyncCompletionRejection::new(
        crate::facade::BridgeAsyncCompletionRejectionKind::ForeignOwnerObservationAuthority,
        "owned async continuation belongs to another Bridge runtime",
    )
}
