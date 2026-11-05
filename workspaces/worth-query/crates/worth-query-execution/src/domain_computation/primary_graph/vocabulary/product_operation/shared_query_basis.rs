//! Moved whole from `domain_computation::primary_graph::product_operation::shared_query_basis`: the items below and every impl they own.
use worth_relational::facade::mvcc::CompanionPreflightStop;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectedQueryBasisRetentionStop {
    Basis,
    Admission(CompanionPreflightStop),
}
