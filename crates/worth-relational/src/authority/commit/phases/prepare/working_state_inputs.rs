//! Exact inputs for cloning the authoritative working state.
use crate::config::data::AdjacencyPolicy;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;

pub(crate) struct WorkingStatePreparation<'a> {
    pub(crate) adjacency_policy: &'a AdjacencyPolicy,
    pub(crate) instrumentation: &'a RuntimeInstrumentation,
}

/// The selected schema and the staging footprint of the actual transaction.
pub(crate) struct SelectionTransaction<'a> {
    pub(crate) transaction_id: crate::transactions::data::primitives::TransactionId,
    pub(crate) schema:
        std::sync::Arc<crate::branch::root::schema::RelationalBranchRootSchemaAuthority>,
    pub(crate) footprint:
        &'a mut crate::mvcc::transaction::footprint::RelationalTransactionFootprint,
}
