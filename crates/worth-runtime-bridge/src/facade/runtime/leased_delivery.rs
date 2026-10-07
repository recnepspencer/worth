use super::*;

impl RuntimeBridge {
    pub fn deliver_invalidation_with_request(
        &self,
        route: BridgePlannedRoute,
        request: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeRouteResult, BridgeDeliveryError> {
        self.deliver_prepared_with_request(
            crate::delivery::prepare_planned_route_for_delivery(route),
            request,
        )
    }

    pub fn deliver_prepared_with_request(
        &self,
        prepared: BridgePreparedDeliveryRequest,
        request: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeRouteResult, BridgeDeliveryError> {
        crate::delivery::deliver_prepared_route_with_request(self, prepared, request)
    }

    /// Delivers a route while carrying the caller's execution lease into the
    /// admitted snapshot reader.
    pub fn deliver_invalidation_with_lease(
        &self,
        route: BridgePlannedRoute,
        lease: &worth_execution::ExecutionResourceLease<'_>,
    ) -> Result<BridgeRouteResult, BridgeDeliveryError> {
        crate::delivery::deliver_planned_route_with_lease(self, route, lease)
    }

    /// Delivers a prepared route under the same caller lease.
    pub fn deliver_prepared_with_lease(
        &self,
        prepared: BridgePreparedDeliveryRequest,
        lease: &worth_execution::ExecutionResourceLease<'_>,
    ) -> Result<BridgeRouteResult, BridgeDeliveryError> {
        crate::delivery::deliver_prepared_route_with_request(
            self,
            prepared,
            worth_execution::ExecutionRequest::leased(lease),
        )
    }
}
