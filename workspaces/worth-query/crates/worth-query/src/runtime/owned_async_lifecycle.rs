use super::{
    WorthQueryOwnedAsyncRequestCustody, WorthQueryOwnedAsyncRuntimeCause,
    WorthQueryOwnedAsyncRuntimeDenial, WorthQueryRuntime,
};
use worth_runtime_bridge::facade::{
    BridgeAsyncRetryLineage, BridgeOwnedAsyncManagedRevalidation, BridgeOwnedAsyncRequestAdmission,
    BridgeOwnedAsyncRetrySchedule, BridgeOwnedAsyncTimeoutAdmission,
};

impl WorthQueryRuntime {
    pub fn advance_owned_bridge_async_request_to_timeout(
        &self,
        owner: &mut WorthQueryOwnedAsyncRequestCustody,
        coordinate: u64,
    ) -> Result<BridgeOwnedAsyncTimeoutAdmission, WorthQueryOwnedAsyncRuntimeCause> {
        self.owned_async_services().timeout(owner, coordinate)
    }
    pub fn schedule_owned_bridge_async_retry(
        &self,
        owner: &mut WorthQueryOwnedAsyncRequestCustody,
        timeout: &BridgeOwnedAsyncTimeoutAdmission,
    ) -> Result<BridgeOwnedAsyncRetrySchedule, WorthQueryOwnedAsyncRuntimeCause> {
        self.owned_async_services().schedule_retry(owner, timeout)
    }
    pub fn advance_owned_bridge_async_retry(
        &self,
        owner: &mut WorthQueryOwnedAsyncRequestCustody,
        schedule: &BridgeOwnedAsyncRetrySchedule,
        coordinate: u64,
    ) -> Result<BridgeAsyncRetryLineage, WorthQueryOwnedAsyncRuntimeCause> {
        self.owned_async_services()
            .advance_retry(owner, schedule, coordinate)
    }
    pub fn revalidate_owned_bridge_async_request(
        &self,
        owner: &mut WorthQueryOwnedAsyncRequestCustody,
        selected: &worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease,
    ) -> Result<BridgeOwnedAsyncManagedRevalidation, WorthQueryOwnedAsyncRuntimeCause> {
        self.owned_async_services().revalidate(owner, selected)
    }
    pub fn finish_owned_bridge_async_wake_cleanup(
        &self,
        owner: &mut WorthQueryOwnedAsyncRequestCustody,
    ) -> Result<(), WorthQueryOwnedAsyncRuntimeCause> {
        self.owned_async_services().finish_wake_cleanup(owner)
    }
    pub fn owned_bridge_async_active_request_count(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
    ) -> Result<usize, WorthQueryOwnedAsyncRuntimeDenial> {
        self.owned_async_services().active_request_count(request)
    }
}
