use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalBridgeSinkErrorTag {}
pub type SignalBridgeSinkError = BridgeMessageError<SignalBridgeSinkErrorTag>;

pub trait InvalidationSink: Send + Sync + 'static {
    fn deliver_invalidation(
        &self,
        delivery: BridgeSignalInvalidationDelivery,
        _lease: Option<&crate::facade::ExecutionResourceLease<'_>>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError>;
}

pub trait SignalBridgeSink: InvalidationSink {}

impl<T> SignalBridgeSink for T where T: InvalidationSink {}
