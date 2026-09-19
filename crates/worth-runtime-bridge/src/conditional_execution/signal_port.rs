use std::ops::Deref;
use std::sync::{Arc, OnceLock};

type SignalPort = worth_signal::facade::branch::SignalConditionalExecutionPort<(), (), ()>;

/// Preallocated ownership of an exact Signal conditional service port.
///
/// Initial definitions share the port sealed with the graph. Definition
/// successors reserve their Arc storage before owner execution and initialize
/// only the fixed port value after Signal returns the exact successor basis.
#[derive(Clone)]
pub(super) enum BridgeConditionalSignalPort {
    Shared(Arc<SignalPort>),
    Reserved(Arc<OnceLock<SignalPort>>),
}

impl BridgeConditionalSignalPort {
    pub(super) fn shared(port: Arc<SignalPort>) -> Self {
        Self::Shared(port)
    }

    pub(super) fn reserve() -> Self {
        Self::Reserved(Arc::new(OnceLock::new()))
    }

    pub(super) fn initialize(&self, port: SignalPort) {
        if let Self::Reserved(slot) = self {
            slot.get_or_init(move || port);
        }
    }
}

impl Deref for BridgeConditionalSignalPort {
    type Target = SignalPort;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Shared(port) => port,
            Self::Reserved(slot) => slot
                .get()
                .expect("a published lowering retains its initialized Signal port"),
        }
    }
}
