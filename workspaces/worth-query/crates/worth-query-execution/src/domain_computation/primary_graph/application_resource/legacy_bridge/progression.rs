use super::{
    WorthQueryOwnedAsyncRequestCustody as Custody, WorthQueryOwnedAsyncRuntimeCause as Cause,
    WorthQueryOwnedAsyncRuntimeDenial as Denial, WorthQueryOwnedAsyncRuntimeServices,
};
use worth_runtime_bridge::facade::{
    BridgeAsyncRetryLineage, BridgeOwnedAsyncManagedRevalidation, BridgeOwnedAsyncRetrySchedule,
    BridgeOwnedAsyncTimeoutAdmission,
};
use worth_runtime_world::facade::RuntimeWorldOwnedAsyncManagedRevalidationDenial as WorldDenial;

impl WorthQueryOwnedAsyncRuntimeServices<'_> {
    pub fn timeout(
        &self,
        owner: &mut Custody,
        coordinate: u64,
    ) -> Result<BridgeOwnedAsyncTimeoutAdmission, Cause> {
        self.require_custody(owner)?;
        self.bridge
            .advance_managed_owned_async_request_to_timeout(&mut owner.native, coordinate)
            .map_err(Cause::Custody)
    }

    pub fn schedule_retry(
        &self,
        owner: &mut Custody,
        timeout: &BridgeOwnedAsyncTimeoutAdmission,
    ) -> Result<BridgeOwnedAsyncRetrySchedule, Cause> {
        self.require_custody(owner)?;
        self.bridge
            .schedule_managed_owned_async_retry(&mut owner.native, timeout)
            .map_err(Cause::Custody)
    }

    pub fn advance_retry(
        &self,
        owner: &mut Custody,
        schedule: &BridgeOwnedAsyncRetrySchedule,
        coordinate: u64,
    ) -> Result<BridgeAsyncRetryLineage, Cause> {
        self.require_custody(owner)?;
        self.bridge
            .advance_managed_owned_async_retry(&mut owner.native, schedule, coordinate)
            .map_err(Cause::Custody)
    }

    pub fn revalidate(
        &self,
        owner: &mut Custody,
        selected: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<BridgeOwnedAsyncManagedRevalidation, Cause> {
        self.require_custody(owner)?;
        self.require_selected(selected)?;
        self.world
            .revalidate_custodied_owned_async_request(self.bridge, &mut owner.native, selected)
            .map_err(|cause| match cause {
                WorldDenial::ForeignOwner | WorldDenial::RelationalSourceMismatch => {
                    Denial::ProductSelectionMismatch.into()
                }
                WorldDenial::NoCommittedSource => Denial::NoCommittedSource.into(),
                WorldDenial::Bridge(cause) => Cause::Custody(cause),
            })
    }
}
