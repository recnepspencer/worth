use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalBridgeSinkError {
    Message(Arc<str>),
    Execution(crate::error::BridgeExecutionDenial),
}

impl SignalBridgeSinkError {
    pub fn new(message: impl Into<Arc<str>>) -> Self {
        Self::Message(message.into())
    }

    pub(crate) fn delivery_kind(&self) -> BridgeDeliveryErrorKind {
        match self {
            Self::Message(_) => BridgeDeliveryErrorKind::SignalSinkRejection,
            Self::Execution(denial) => BridgeDeliveryErrorKind::ExecutionDenied(*denial),
        }
    }
}

impl From<worth_execution::LeaseDenial> for SignalBridgeSinkError {
    fn from(denial: worth_execution::LeaseDenial) -> Self {
        Self::Execution(denial.into())
    }
}

impl std::fmt::Display for SignalBridgeSinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Message(message) => f.write_str(message),
            Self::Execution(denial) => write!(f, "Bridge execution refused: {denial:?}"),
        }
    }
}
impl std::error::Error for SignalBridgeSinkError {}

pub trait InvalidationSink: Send + Sync + 'static {
    fn deliver_invalidation(
        &self,
        delivery: BridgeSignalInvalidationDelivery,
        _lease: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError>;
}

pub trait SignalBridgeSink: InvalidationSink {}

impl<T> SignalBridgeSink for T where T: InvalidationSink {}
