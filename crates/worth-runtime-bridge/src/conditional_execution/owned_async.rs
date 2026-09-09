mod admission;
mod completion;
mod declaration;
mod installation;
mod retirement;
mod retry_revalidation;
mod supersession;

pub use declaration::BridgeOwnedAsyncRequestResponseDeclaration;

fn foreign_continuation_denial() -> crate::facade::BridgeAsyncCompletionRejection {
    crate::facade::BridgeAsyncCompletionRejection::new(
        crate::facade::BridgeAsyncCompletionRejectionKind::ForeignOwnerObservationAuthority,
        "owned async continuation belongs to another Bridge runtime",
    )
}
