//! Native refusals retain their original cancellation, authority, and recovery meaning.
use super::NativeDenial;
use crate::http::server::{
    advancement_denial, authorization_denial, construction_denial, estate_denial,
    mutation_application, mutation_refusal, query_denial,
};
use crate::{BankHttpDenial, BankHttpDenialKind as Kind, BankHttpNextAction as Next};
use worth_query_host::facade::{application_entry::*, primary_graph::*};
mod recovery;
#[cfg(test)]
mod tests;

pub(super) fn command_denial(
    cause: WorthQueryCapabilityCommandDenial<NativeDenial>,
) -> BankHttpDenial {
    match cause {
        WorthQueryCapabilityCommandDenial::FreshRequestMismatch => {
            BankHttpDenial::new(Kind::PermissionDenied, Next::None)
        }
        WorthQueryCapabilityCommandDenial::Native(cause) => native_denial(cause),
        WorthQueryCapabilityCommandDenial::Execution(cause) => {
            advancement_denial::advancement(cause)
        }
        WorthQueryCapabilityCommandDenial::Cleanup(cause) => mutation_refusal::close_denial(cause),
    }
}

pub(in crate::http::server) fn native_denial(cause: NativeDenial) -> BankHttpDenial {
    use WorthQueryApplicationCapabilityRevocationDenial as D;
    match cause {
        D::ExecutionRequest(cause) => advancement_denial::advancement(cause),
        D::Program(cause) => mutation_application::commit_denial(cause.kind()),
        D::PrincipalMismatch => BankHttpDenial::new(Kind::PermissionDenied, Next::None),
        D::InputCapture(cause) => construction_denial::capture(&cause),
        D::Recovery(cause) => recovery::managed(cause),
        D::Settlement(cause) => recovery::settlement(cause),
        D::ProductSelection(cause) => query(
            WorthQueryApplicationRequestQueryDenial::ProductSelection(cause),
        ),
        D::PrincipalResolution(cause) => {
            query(WorthQueryApplicationRequestQueryDenial::PrincipalResolution(cause))
        }
        D::Authorization(cause) => authorization_denial::authorization_denial(cause.kind().into()),
        D::Idempotency(cause) => mutation_application::request_mutation_denial(
            WorthQueryApplicationRequestMutationDenial::Idempotency(cause).kind(),
        ),
        D::Preparation(cause) => estate_denial::estate_denial(cause),
        D::IdempotencyIntentDrift => BankHttpDenial::new(Kind::Stale, Next::CorrectRequest),
        D::IdentityEncoding(_)
        | D::ProgramMismatch
        | D::PrincipalBindingInstallation(_)
        | D::CapabilityInstallation(_)
        | D::OperationInstallation(_) => operator(),
    }
}

fn query(cause: WorthQueryApplicationRequestQueryDenial) -> BankHttpDenial {
    query_denial::query_denial(bank_server::BankApplicationQueryDenial::from_request_query(
        cause,
    ))
}
fn operator() -> BankHttpDenial {
    BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator)
}
fn unavailable() -> BankHttpDenial {
    BankHttpDenial::new(Kind::Unavailable, Next::Retry)
}
