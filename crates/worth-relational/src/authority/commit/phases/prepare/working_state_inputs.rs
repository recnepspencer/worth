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

use crate::branch::root::schema::RelationalBranchRootSchemaAuthority;
use crate::config::data::CrossContextPolicy;
use crate::mvcc::transaction::footprint::RelationalTransactionFootprint;
use crate::mvcc::transaction::normalized_plan::OwnedNormalizedPlan;
use crate::mvcc::transaction::staged_plan_normalization::StagedPlanNormalization;
use crate::schema::data::RelationalSchemaRegistry;
use crate::transactions::data::primitives::TransactionId;
use std::sync::{Arc, Mutex};

/// The draft borrows only the installed inputs used in plan validation and cloning.
pub(crate) struct WorkingStateDraftRuntime<'a> {
    pub(crate) working_state: WorkingStatePreparation<'a>,
    pub(crate) normalization: StagedPlanNormalization<'a>,
    pub(crate) cross_context: CrossContextPolicy,
    pub(crate) commit_work: Option<&'a crate::execution::RequestWorkBudget>,
}

pub(crate) struct WorkingStateDraft<'a> {
    pub(crate) transaction: TransactionId,
    pub(crate) overlay:
        &'a mut crate::mvcc::transaction::overlay::DetachedRelationalTransactionOverlay,
    pub(crate) footprint: &'a mut RelationalTransactionFootprint,
    pub(crate) basis_schema: &'a RelationalSchemaRegistry,
    pub(crate) mutation_schema: &'a Arc<RelationalBranchRootSchemaAuthority>,
    pub(crate) merge_parent_count: usize,
    pub(crate) coupled_commands: Option<&'a Mutex<Option<Arc<OwnedNormalizedPlan>>>>,
}

pub(crate) struct WorkingStateScope<'a> {
    pub(crate) footprint: &'a RelationalTransactionFootprint,
    pub(crate) schema: &'a Arc<RelationalBranchRootSchemaAuthority>,
    pub(crate) merge_parent_count: usize,
}
