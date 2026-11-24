//! Exact inputs for cloning the authoritative working state.
use contracts::config::data::policies::AdjacencyPolicy;
use worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port as mutation;

pub struct WorkingStatePreparation<'a> {
    pub adjacency_policy: &'a AdjacencyPolicy,
    pub instrumentation: &'a RuntimeInstrumentation,
}

/// The selected schema and the staging footprint of the actual transaction.
pub struct SelectionTransaction<'a> {
    pub transaction_id: contracts::transactions::data::primitives::TransactionId,
    pub schema:
        std::sync::Arc<worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority>,
    pub footprint:
        &'a mut mutation::mvcc::transaction::footprint::RelationalTransactionFootprint,
}

use contracts::config::data::policies::CrossContextPolicy;
use contracts::schema::data::RelationalSchemaRegistry;
use contracts::transactions::data::primitives::TransactionId;
use mutation::mvcc::transaction::footprint::RelationalTransactionFootprint;
use mutation::mvcc::transaction::normalized_plan::OwnedNormalizedPlan;
use mutation::mvcc::transaction::staged_plan_normalization::StagedPlanNormalization;
use std::sync::{Arc, Mutex};
use worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority;

/// The draft borrows only the installed inputs used in plan validation and cloning.
pub struct WorkingStateDraftRuntime<'a> {
    pub working_state: WorkingStatePreparation<'a>,
    pub normalization: StagedPlanNormalization<'a>,
    pub cross_context: CrossContextPolicy,
    pub commit_work: Option<&'a contracts::execution::request_work::RequestWorkBudget>,
}

pub struct WorkingStateDraft<'a> {
    pub transaction: TransactionId,
    pub overlay: &'a mut mutation::mvcc::transaction::overlay::DetachedRelationalTransactionOverlay,
    pub footprint: &'a mut RelationalTransactionFootprint,
    pub basis_schema: &'a RelationalSchemaRegistry,
    pub mutation_schema: &'a Arc<RelationalBranchRootSchemaAuthority>,
    pub merge_parent_count: usize,
    pub coupled_commands: Option<&'a Mutex<Option<Arc<OwnedNormalizedPlan>>>>,
}

pub struct WorkingStateScope<'a> {
    pub footprint: &'a RelationalTransactionFootprint,
    pub schema: &'a Arc<RelationalBranchRootSchemaAuthority>,
    pub merge_parent_count: usize,
}
