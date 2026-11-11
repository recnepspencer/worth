//! Model ownership is admitted from actual current facts, never a scope label.
use super::*;
use worth_query_host::facade::primary_graph::{
    RelationalModelOwnershipDenial, StructuralWalkDenialKind,
};
pub(super) fn denial(cause: RelationalModelOwnershipDenial) -> BankHttpDenial {
    use RelationalModelOwnershipDenial as D;
    match cause {
        D::Request(cause) => advancement_denial::request(cause),
        D::Native(cause) => relational_execution_denial::denial(cause),
        D::Custody(cause) => publication_read::denial(cause),
        D::Traversal(cause) => managed_index::native_source::walk(cause),
        D::Read(cause) => managed_index::native_source::walk(StructuralWalkDenialKind::Read(cause)),
        D::TargetNotCurrent | D::SchemaChanged | D::EntityUnavailable(_) => stale(),
        D::ForeignBasis | D::ModelRootKind { .. } | D::Conflict(_) | D::Adjacency(_) => operator(),
    }
}
