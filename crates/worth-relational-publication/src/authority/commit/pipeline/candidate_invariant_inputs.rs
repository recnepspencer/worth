//! Exact request-stage validation reads borrowed from the prepared candidate owner.
use worth_relational_mutation::internal_port as mutation;
use worth_relational_branch::internal_port::branch::root::schema::compiled_retention::RetainedRelationalSchemaRegistry;
use worth_relational_branch::internal_port::branch::root::RelationalBranchRootState;
use worth_relational_contracts::internal_port::config::data::runtime_config::RelationalRuntimeConfig;
use worth_relational_contracts::internal_port::execution::request_work::RequestWorkBudget;
use crate::runtime::state::subsystems::history::history_preparation_binding::RelationalPreparationHistory;
use mutation::runtime::state::subsystems::schema_contract_runtime::SchemaContractRuntimeSubsystem;
use worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use worth_relational_branch::internal_port::runtime::state::subsystems::visibility::snapshot_handles::published_capacity::PublishedSnapshotCapacityOwner;
use mutation::validation::engine::runtime_view::InvariantRuntimeInputs;
use mutation::validation::engine::runtime_view::InvariantRuntimeView;
use std::sync::Arc;

pub struct CandidateInvariantInputs<'a> {
    pub config: &'a Arc<RelationalRuntimeConfig>,
    pub schema_contract_runtime: &'a Arc<SchemaContractRuntimeSubsystem>,
    pub execution_registry: Option<&'a RetainedRelationalSchemaRegistry>,
    pub home_target: bool,
    pub commit_work_budget: Option<&'a RequestWorkBudget>,
    pub instrumentation: &'a RuntimeInstrumentation,
    pub history: &'a RelationalPreparationHistory,
    pub snapshot_capacity: &'a PublishedSnapshotCapacityOwner,
}

impl<'a> CandidateInvariantInputs<'a> {
    pub fn select(&self, state: &RelationalBranchRootState) -> InvariantRuntimeView<'a> {
        InvariantRuntimeView::from_inputs(InvariantRuntimeInputs {
            selected_registry: worth_relational_branch::internal_port::schema::registry_custody::SchemaRegistryCustody::Selected(
                self.execution_registry
                    .cloned()
                    .unwrap_or_else(|| state.root().schema_authority().registry_arc()),
            ),
            prior_registry: self.home_target.then(|| {
                worth_relational_branch::internal_port::schema::registry_custody::SchemaRegistryCustody::Selected(
                    state.root().schema_authority().registry_arc(),
                )
            }),
            commit_work_budget: self.commit_work_budget.cloned(),
            config: Arc::clone(self.config),
            schema_contract_runtime: Arc::clone(self.schema_contract_runtime),
            instrumentation: self.instrumentation,

            entity_count: state.entity_slot_count(),
            relation_count: state.relation_slot_count(),
            version_depth: self.history.current_version_id().0 as usize,
            snapshot_pressure: self.snapshot_capacity.occupied_handles() > 10,
        })
    }
}
