use super::admission_denial::admission_denial;
use crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial;
use crate::domain_computation::primary_graph::vocabulary::index_currency::WorthQueryPrimaryIndexCurrencyDenial;
use {
    crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenial,
    crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenialKind,
};

pub(in crate::domain_computation::primary_graph::application_query) fn map_index_currency_denial(
    denial: WorthQueryPrimaryIndexCurrencyDenial,
) -> WorthQueryApplicationQueryAdmissionDenial {
    match denial {
        WorthQueryPrimaryIndexCurrencyDenial::Execution(stop) => {
            let stop = crate::domain_computation::primary_graph::vocabulary::provider::relational_execution_denial::relational_execution_kind(stop, None);
            let kind = match stop {
                Ok(kind) => WorthQueryApplicationQueryAdmissionDenialKind::ExecutionRequest(kind),
                Err(
                    crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::Cancelled,
                ) => WorthQueryApplicationQueryAdmissionDenialKind::Cancelled,
                Err(
                    crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::TimedOut,
                ) => WorthQueryApplicationQueryAdmissionDenialKind::DeadlineExceeded,
            };
            admission_denial(kind, "index currency execution stopped")
        }

        WorthQueryPrimaryIndexCurrencyDenial::Basis(
            WorthQueryExactBasisSnapshotDenial::RetentionCapacityExhausted,
        ) => admission_denial(
            WorthQueryApplicationQueryAdmissionDenialKind::RetentionCapacityExhausted,
            "primary graph index currency exhausted relational retention capacity",
        ),
        WorthQueryPrimaryIndexCurrencyDenial::Basis(
            WorthQueryExactBasisSnapshotDenial::RetentionIdentityExhausted,
        ) => admission_denial(
            WorthQueryApplicationQueryAdmissionDenialKind::RetentionIdentityExhausted,
            "primary graph index currency exhausted retention identity space",
        ),
        WorthQueryPrimaryIndexCurrencyDenial::Basis(
            WorthQueryExactBasisSnapshotDenial::SnapshotIdentityExhausted,
        ) => admission_denial(
            WorthQueryApplicationQueryAdmissionDenialKind::SnapshotIdentityExhausted,
            "primary graph index currency exhausted snapshot identity space",
        ),
        WorthQueryPrimaryIndexCurrencyDenial::Basis(
            WorthQueryExactBasisSnapshotDenial::BranchMaterializationSuspended,
        ) => admission_denial(
            WorthQueryApplicationQueryAdmissionDenialKind::BranchMaterializationSuspended,
            "the selected branch materialization is suspended",
        ),
        WorthQueryPrimaryIndexCurrencyDenial::IndexUnavailable(detail) => admission_denial(
            WorthQueryApplicationQueryAdmissionDenialKind::RuntimeSupportUnavailable,
            detail,
        ),
        WorthQueryPrimaryIndexCurrencyDenial::Basis(_) => admission_denial(
            WorthQueryApplicationQueryAdmissionDenialKind::RuntimeSupportUnavailable,
            "primary graph branch basis is unavailable for index currency",
        ),
    }
}
