//! One native projection's codec admission, borrowing the original request.
mod field;
pub mod session;
pub use field::{FieldDecoder, FieldEncoder, ScalarFieldDecoder, ScalarFieldEncoder};
pub mod state;

pub use session::ValueDecodeSession;
pub use state::ValueDecodeState;

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
    Content(worth_relational::facade::content::RelationalContentDenial),
    RequiredContentFieldAbsent,
    ContentBasisUnavailable,
    ContentProjectionWorkExceeded,
    ContentProjectionWorkOverflow,
}

pub fn decode_request_cause(
    denial: WorthQueryValueDecodeDenial,
) -> Option<worth_execution::WorkCeilingDenial> {
    use worth_execution::{LeaseDenial, WorkCeilingDenial as Request};
    match denial {
        WorthQueryValueDecodeDenial::Checkpoint(stop) => Some(Request::Stopped(stop)),
        WorthQueryValueDecodeDenial::Scope(cause) => Some(cause),
        WorthQueryValueDecodeDenial::Memory(memory) => {
            Some(Request::Admission(LeaseDenial::MemoryExhausted(memory)))
        }
        WorthQueryValueDecodeDenial::Reservation(lease) => Some(Request::Admission(lease)),
        WorthQueryValueDecodeDenial::Content(
            worth_relational::facade::content::RelationalContentDenial::Execution(cause),
        ) => cause.as_kernel_stop().map(Request::Stopped),
        WorthQueryValueDecodeDenial::Content(_)
        | WorthQueryValueDecodeDenial::RequiredContentFieldAbsent
        | WorthQueryValueDecodeDenial::ContentBasisUnavailable
        | WorthQueryValueDecodeDenial::ContentProjectionWorkExceeded
        | WorthQueryValueDecodeDenial::ContentProjectionWorkOverflow
        | WorthQueryValueDecodeDenial::AccountingOverflow
        | WorthQueryValueDecodeDenial::BindingContractMismatch
        | WorthQueryValueDecodeDenial::CodecRejected => None,
    }
}
