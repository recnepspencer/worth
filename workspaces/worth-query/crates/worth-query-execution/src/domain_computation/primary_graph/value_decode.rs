//! One native projection's codec admission, borrowing the original request.
mod session;
mod state;

pub(in crate::domain_computation::primary_graph) use session::ValueDecodeSession;
pub(in crate::domain_computation::primary_graph) use state::{with_value_decode, ValueDecodeState};

/// The first refusal of a native admitted field decoder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryValueDecodeDenial {
    Checkpoint(worth_execution::MapKernelStop),
    Memory(worth_execution::MemoryLimitDenial),
    Scope(worth_execution::WorkCeilingDenial),
    Reservation(worth_execution::LeaseDenial),
    AccountingOverflow,
    BindingContractMismatch,
    CodecRejected,
}

impl WorthQueryValueDecodeDenial {
    pub(in crate::domain_computation::primary_graph) fn request_cause(
        self,
    ) -> Option<worth_execution::WorkCeilingDenial> {
        use worth_execution::{LeaseDenial, WorkCeilingDenial as Request};
        match self {
            Self::Checkpoint(stop) => Some(Request::Stopped(stop)),
            Self::Scope(cause) => Some(cause),
            Self::Memory(memory) => Some(Request::Admission(LeaseDenial::MemoryExhausted(memory))),
            Self::Reservation(lease) => Some(Request::Admission(lease)),
            Self::AccountingOverflow | Self::BindingContractMismatch | Self::CodecRejected => None,
        }
    }
}
