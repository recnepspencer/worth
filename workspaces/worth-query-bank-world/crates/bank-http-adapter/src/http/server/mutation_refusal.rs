//! Native refusals projected without consuming returned custody or inventing posture.
use super::super::protocol::{
    BankHttpDenial, BankHttpDenialKind as Kind, BankHttpNextAction as Next,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRecoveryRequestDenial as Recovery, WorthQueryOperationCloseDenial,
    WorthQuerySourceOnlyDenial,
};
pub(super) const fn denial(kind: Kind) -> BankHttpDenial {
    BankHttpDenial::new(kind, Next::None)
}
pub(super) fn source_denial(cause: WorthQuerySourceOnlyDenial) -> BankHttpDenial {
    match cause {
        WorthQuerySourceOnlyDenial::Mutation(cause) => {
            super::mutation_application::request_mutation_denial(cause.kind())
        }
        WorthQuerySourceOnlyDenial::Recovery(Recovery::Request(cause)) => {
            super::mutation_application::request_mutation_denial(cause.kind())
        }
        WorthQuerySourceOnlyDenial::Cleanup(cause) => close_denial(cause),
        WorthQuerySourceOnlyDenial::Execution(cause) => {
            super::advancement_denial::advancement(cause)
        }
        WorthQuerySourceOnlyDenial::History(_) => denial(Kind::Unavailable),
        WorthQuerySourceOnlyDenial::Recovery(_) => denial(Kind::Unavailable),
    }
}
pub(super) fn close_denial(cause: WorthQueryOperationCloseDenial) -> BankHttpDenial {
    match cause {
        WorthQueryOperationCloseDenial::Execution(cause) => {
            super::advancement_denial::advancement(cause)
        }
        WorthQueryOperationCloseDenial::FreshRequestMismatch => denial(Kind::PermissionDenied),
        _ => denial(Kind::Unavailable),
    }
}
