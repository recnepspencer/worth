use worth_runtime_bridge::facade::{
    BridgeOwnedAsyncRequestAdmission, BridgeOwnedAsyncRetryAdmission,
    BridgeOwnedAsyncRetrySchedule, BridgeOwnedAsyncRevalidationAdmission,
    BridgeOwnedAsyncTimeoutAdmission,
};

use super::{WorthQueryOwnedAsyncRuntimeDenial, WorthQueryRuntime};

impl WorthQueryRuntime {
    pub fn advance_owned_bridge_async_request_to_timeout(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
        coordinate: u64,
    ) -> Result<BridgeOwnedAsyncTimeoutAdmission, WorthQueryOwnedAsyncRuntimeDenial> {
        self.installed_product
            .as_ref()
            .map(|product| &product.conditional)
            .ok_or(WorthQueryOwnedAsyncRuntimeDenial::ConditionalRuntimeUnavailable)?
            .advance_owned_async_request_to_timeout(request, coordinate)
            .map_err(WorthQueryOwnedAsyncRuntimeDenial::Completion)
    }

    pub fn schedule_owned_bridge_async_retry(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
        timeout: &BridgeOwnedAsyncTimeoutAdmission,
    ) -> Result<BridgeOwnedAsyncRetrySchedule, WorthQueryOwnedAsyncRuntimeDenial> {
        self.installed_product
            .as_ref()
            .map(|product| &product.conditional)
            .ok_or(WorthQueryOwnedAsyncRuntimeDenial::ConditionalRuntimeUnavailable)?
            .schedule_owned_async_retry(request, timeout)
            .map_err(WorthQueryOwnedAsyncRuntimeDenial::Completion)
    }

    pub fn advance_owned_bridge_async_retry(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
        schedule: &BridgeOwnedAsyncRetrySchedule,
        coordinate: u64,
    ) -> Result<BridgeOwnedAsyncRetryAdmission, WorthQueryOwnedAsyncRuntimeDenial> {
        self.installed_product
            .as_ref()
            .map(|product| &product.conditional)
            .ok_or(WorthQueryOwnedAsyncRuntimeDenial::ConditionalRuntimeUnavailable)?
            .advance_owned_async_retry(request, schedule, coordinate)
            .map_err(WorthQueryOwnedAsyncRuntimeDenial::Completion)
    }

    pub fn revalidate_owned_bridge_async_request(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
        selected: &worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease,
    ) -> Result<BridgeOwnedAsyncRevalidationAdmission, WorthQueryOwnedAsyncRuntimeDenial> {
        let product = self
            .installed_product
            .as_ref()
            .ok_or(WorthQueryOwnedAsyncRuntimeDenial::ConditionalRuntimeUnavailable)?;
        product
            .validate_selected_source(selected, None)
            .map_err(|_| WorthQueryOwnedAsyncRuntimeDenial::ProductBasisRequired)?;
        product
            .world
            .revalidate_owned_async_request(&product.conditional, request, selected)
            .map_err(|denial| match denial {
                worth_query_execution::facade::integration::RuntimeWorldOwnedAsyncRevalidationDenial::ForeignOwner
                | worth_query_execution::facade::integration::RuntimeWorldOwnedAsyncRevalidationDenial::RelationalSourceMismatch => {
                    WorthQueryOwnedAsyncRuntimeDenial::ProductBasisRequired
                }
                worth_query_execution::facade::integration::RuntimeWorldOwnedAsyncRevalidationDenial::Bridge(
                    denial,
                ) => WorthQueryOwnedAsyncRuntimeDenial::Completion(denial),
            })
    }

    pub fn owned_bridge_async_active_request_count(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
    ) -> Result<usize, WorthQueryOwnedAsyncRuntimeDenial> {
        self.installed_product
            .as_ref()
            .map(|product| &product.conditional)
            .ok_or(WorthQueryOwnedAsyncRuntimeDenial::ConditionalRuntimeUnavailable)?
            .owned_async_active_request_count(request)
            .map_err(WorthQueryOwnedAsyncRuntimeDenial::Completion)
    }
}
