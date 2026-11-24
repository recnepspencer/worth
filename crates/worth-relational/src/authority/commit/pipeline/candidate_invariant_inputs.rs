//! Exact request-stage validation reads borrowed from the prepared candidate owner.
use crate::branch::root::schema::RetainedRelationalSchemaRegistry;
use crate::branch::root::RelationalBranchRootState;
use crate::config::data::runtime_config::RelationalRuntimeConfig;
use crate::execution::request_work::RequestWorkBudget;
use crate::runtime::state::subsystems::history::history_preparation_binding::RelationalPreparationHistory;
use crate::runtime::state::subsystems::schema_contract_runtime::SchemaContractRuntimeSubsystem;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::runtime::state::subsystems::visibility::PublishedSnapshotCapacityOwner;
use crate::validation::engine::{InvariantRuntimeInputs, InvariantRuntimeView};
use std::sync::Arc;

pub(crate) struct CandidateInvariantInputs<'a> {
    pub(crate) config: &'a Arc<RelationalRuntimeConfig>,
    pub(crate) schema_contract_runtime: &'a Arc<SchemaContractRuntimeSubsystem>,
    pub(crate) execution_registry: Option<&'a RetainedRelationalSchemaRegistry>,
    pub(crate) home_target: bool,
    pub(crate) commit_work_budget: Option<&'a RequestWorkBudget>,
    pub(crate) instrumentation: &'a RuntimeInstrumentation,
    pub(crate) history: &'a RelationalPreparationHistory,
    pub(crate) snapshot_capacity: &'a PublishedSnapshotCapacityOwner,
}

impl<'a> CandidateInvariantInputs<'a> {
    pub(crate) fn select(&self, state: &RelationalBranchRootState) -> InvariantRuntimeView<'a> {
        InvariantRuntimeView::from_inputs(InvariantRuntimeInputs {
            selected_registry: crate::schema::SchemaRegistryCustody::Selected(
                self.execution_registry
                    .cloned()
                    .unwrap_or_else(|| state.root().schema_authority().registry_arc()),
            ),
            prior_registry: self.home_target.then(|| {
                crate::schema::SchemaRegistryCustody::Selected(
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
