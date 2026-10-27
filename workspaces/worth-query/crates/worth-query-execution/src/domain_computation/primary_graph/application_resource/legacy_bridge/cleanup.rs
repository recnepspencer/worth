use super::{
    WorthQueryOwnedAsyncRequestCustody, WorthQueryOwnedAsyncRuntimeCause as Cause,
    WorthQueryOwnedAsyncRuntimeServices,
};

impl WorthQueryOwnedAsyncRuntimeServices<'_> {
    /// Failed-entry cleanup never implicitly cancels a coalesced winner.
    pub fn cleanup(&self, owner: &mut WorthQueryOwnedAsyncRequestCustody) -> Result<(), Cause> {
        self.require_custody(owner)?;
        self.bridge
            .finish_unqueued_owned_async_entry_cleanup(&mut owner.native)
            .map_err(Cause::Custody)
    }

    /// Explicit caller cancellation names the exact occurrence, including shared winners.
    pub fn cancel(&self, owner: &WorthQueryOwnedAsyncRequestCustody) -> Result<(), Cause> {
        self.require_custody(owner)?;
        self.bridge
            .cancel_managed_owned_async_request(&owner.native)
            .map(|_| ())
            .map_err(Cause::Custody)
    }

    pub fn finish_wake_cleanup(
        &self,
        owner: &mut WorthQueryOwnedAsyncRequestCustody,
    ) -> Result<(), Cause> {
        self.require_custody(owner)?;
        self.bridge
            .finish_managed_owned_async_wake_cleanup(&mut owner.native)
            .map_err(Cause::Custody)
    }
}
