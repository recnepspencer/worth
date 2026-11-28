use crate::diagnostics::BridgeFailureSource;
use crate::error::{BridgeDeliveryError, BridgeErrorContext};
use crate::facade::RuntimeBridge;

pub(super) fn delivery_context(
    route_identity: crate::routing::BridgeRouteIdentity,
    snapshot_identity: crate::snapshot::TruthSnapshotIdentity,
) -> BridgeErrorContext {
    BridgeErrorContext::delivery(route_identity, snapshot_identity)
}

pub(super) fn reject_delivery(
    runtime: &RuntimeBridge,
    source: BridgeFailureSource,
    error: BridgeDeliveryError,
) -> BridgeDeliveryError {
    runtime.diagnostics.record_delivery_failure(source, &error);
    error
}
