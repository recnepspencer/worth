#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedSignalDelivery {
    pub delivery: crate::routing::BridgeSignalInvalidationDelivery,
    pub leased: bool,
}
#[derive(Debug, Clone, Default)]
pub struct RecordingSignalBridgeSink {
    deliveries: Arc<RwLock<Vec<RecordedSignalDelivery>>>,
}

impl RecordingSignalBridgeSink {
    pub fn deliveries(&self) -> Vec<RecordedSignalDelivery> {
        self.deliveries
            .read()
            .expect("bridge sink lock poisoned")
            .clone()
    }

    pub fn last_delivery(&self) -> Option<RecordedSignalDelivery> {
        self.deliveries().into_iter().last()
    }
}

impl InvalidationSink for RecordingSignalBridgeSink {
    fn deliver_invalidation(
        &self,
        delivery: crate::routing::BridgeSignalInvalidationDelivery,
        lease: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        self.deliveries
            .write()
            .expect("bridge sink lock poisoned")
            .push(RecordedSignalDelivery {
                delivery: delivery.clone(),
                leased: lease.is_leased(),
            });

        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}
use super::*;
