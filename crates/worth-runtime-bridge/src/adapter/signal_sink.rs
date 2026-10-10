use super::*;

/// The exact sink refusal; messages never stand in for a cause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalBridgeSinkErrorKind {
    /// An external sink reported an opaque failure.
    ExternalSinkFailure,
    /// The request refused this sink contact.
    Execution(crate::error::BridgeExecutionDenial),
    /// Signal refused evaluation or mutation.
    Signal(worth_signal::facade::SignalError),
    /// Installed scoped-change admission refused the target set.
    ScopedChangeAdmission(worth_signal::facade::SignalInstalledScopedChangeDenial),
    /// The committed-patch owner service refused delivery.
    CommittedPatch(worth_signal::facade::branch::SignalCommittedPatchDeliveryDenial),
    /// Query mutation must use its World publication service.
    WorldPublicationRequired,
}

/// A typed sink cause accompanied by diagnostic text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalBridgeSinkError {
    kind: SignalBridgeSinkErrorKind,
    message: Arc<str>,
}

impl SignalBridgeSinkError {
    /// Attach diagnostic text to an explicitly named cause.
    pub fn new(kind: SignalBridgeSinkErrorKind, message: impl Into<Arc<str>>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// The sink cause before any delivery projection.
    pub fn kind(&self) -> &SignalBridgeSinkErrorKind {
        &self.kind
    }

    pub(crate) fn delivery_kind(&self) -> BridgeDeliveryErrorKind {
        match &self.kind {
            SignalBridgeSinkErrorKind::Execution(denial) => {
                BridgeDeliveryErrorKind::ExecutionDenied(*denial)
            }
            SignalBridgeSinkErrorKind::ExternalSinkFailure
            | SignalBridgeSinkErrorKind::Signal(_)
            | SignalBridgeSinkErrorKind::ScopedChangeAdmission(_)
            | SignalBridgeSinkErrorKind::CommittedPatch(_)
            | SignalBridgeSinkErrorKind::WorldPublicationRequired => {
                BridgeDeliveryErrorKind::SignalSinkRejection(self.kind.clone())
            }
        }
    }
}

impl From<worth_execution::LeaseDenial> for SignalBridgeSinkError {
    fn from(denial: worth_execution::LeaseDenial) -> Self {
        let cause: crate::error::BridgeExecutionDenial = denial.into();
        Self::new(SignalBridgeSinkErrorKind::Execution(cause), cause.label())
    }
}

impl std::fmt::Display for SignalBridgeSinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for SignalBridgeSinkError {}

pub trait InvalidationSink: Send + Sync + 'static {
    fn deliver_invalidation(
        &self,
        delivery: BridgeSignalInvalidationDelivery,
        request: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError>;
}

pub trait SignalBridgeSink: InvalidationSink {}
impl<T> SignalBridgeSink for T where T: InvalidationSink {}
