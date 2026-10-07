use crate::adapter::{InvalidationSink, SignalBridgeSinkError};
use crate::delivery::BridgeDeliveryReceipt;
use crate::routing::BridgeSignalInvalidationDelivery;

pub(in crate::facade::tests) struct StaticSink;

impl InvalidationSink for StaticSink {
    fn deliver_invalidation(
        &self,
        delivery: BridgeSignalInvalidationDelivery,
        _lease: Option<&crate::facade::ExecutionResourceLease<'_>>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}
