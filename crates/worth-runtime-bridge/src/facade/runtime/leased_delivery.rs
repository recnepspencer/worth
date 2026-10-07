use super::*;

impl RuntimeBridge {
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
        crate::delivery::deliver_prepared_route_with_lease(self, prepared, Some(lease))
    }
}
