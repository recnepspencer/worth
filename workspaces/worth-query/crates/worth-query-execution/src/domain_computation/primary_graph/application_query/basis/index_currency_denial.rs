use super::admission_denial::admission_denial;
use crate::domain_computation::primary_graph::application_query::{
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
};
use crate::domain_computation::primary_graph::index_currency::WorthQueryPrimaryIndexCurrencyDenial;
use crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial;

pub(in crate::domain_computation::primary_graph::application_query) fn map_index_currency_denial(
    denial: WorthQueryPrimaryIndexCurrencyDenial,
) -> WorthQueryApplicationQueryAdmissionDenial {
    match denial {
        WorthQueryPrimaryIndexCurrencyDenial::Execution(cause) => {
            use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
            let kind = match crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_kind(cause, None) {
                Ok(kind) => WorthQueryApplicationQueryAdmissionDenialKind::ExecutionRequest(kind),
                Err(Control::Cancelled) => WorthQueryApplicationQueryAdmissionDenialKind::Cancelled,
                Err(Control::TimedOut) => WorthQueryApplicationQueryAdmissionDenialKind::DeadlineExceeded,
            };
            admission_denial(kind, "primary graph index currency")
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
