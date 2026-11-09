//! Original source observation failures retain their existing query projections.
use super::*;
use crate::http::server::relational_execution_denial;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestQueryDenial,
    primary_graph::{
        SelectedQueryBasisRetentionStop, SourceQueryObservationStop, WorthQueryAdmittedOneShotStop,
    },
};

pub(super) fn denial(cause: SourceQueryObservationStop) -> BankHttpDenial {
    use SourceQueryObservationStop as D;
    use WorthQueryApplicationRequestQueryDenial as Query;
    match cause {
        D::CurrentControl(cause) => query(Query::OutputSettlement(cause)),
        D::ManagedIndex(cause) => managed_index::managed_index(&cause),
        D::Installation(cause) => query(Query::BindingInstallation(cause)),
        D::Principal(cause) => query(Query::PrincipalResolution(cause)),
        D::Scope(cause) => query(Query::ScopeResolution(cause)),
        D::Admission(cause) | D::Preparation { cause, .. } => query(Query::Admission(cause)),
        D::Read(cause) => match cause {
            WorthQueryAdmittedOneShotStop::Admission(cause) => {
                relational_execution_denial::companion(cause)
            }
            WorthQueryAdmittedOneShotStop::Execution(cause) => query(Query::Execution(cause)),
            WorthQueryAdmittedOneShotStop::WorkCounterOverflow
            | WorthQueryAdmittedOneShotStop::WorkAccountingMismatch => operator(),
        },
        D::Basis(cause) => match cause {
            SelectedQueryBasisRetentionStop::Basis => unavailable(),
            SelectedQueryBasisRetentionStop::Admission(cause) => {
                relational_execution_denial::companion(cause)
            }
        },
        D::Missing | D::Ambiguous => unavailable(),
        D::ForeignClosure => operator(),
        D::SourceFacts(cause) => managed_index::source(&cause),
    }
}

fn query(cause: WorthQueryApplicationRequestQueryDenial) -> BankHttpDenial {
    query_denial(BankApplicationQueryDenial::from_request_query(cause))
}
fn operator() -> BankHttpDenial {
    BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator)
}
