//! `index_currency`'s vocabulary, moved whole from `primary_graph::index_currency`: its root file's items here, one child per other source file.

#[derive(Debug)]
pub enum WorthQueryPrimaryIndexCurrencyDenial {
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    Basis(crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial),
    IndexUnavailable(&'static str),
}
