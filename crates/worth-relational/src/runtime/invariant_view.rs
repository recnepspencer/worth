//! Select the runtime's immutable validation inputs from one owner epoch.
use crate::runtime::{RelationalPreparationRuntime, RelationalRuntime};
use crate::validation::engine::{InvariantRuntimeInputs, InvariantRuntimeView};

impl RelationalRuntime {
    pub(crate) fn invariant_validation_view(&self) -> InvariantRuntimeView<'_> {
        let configuration = self.operation_configuration();
        let (entity_count, relation_count) = self
            .history
            .branch_cell(self.main_branch_identity().branch_id())
            .and_then(|cell| {
                cell.root()
                    .map(|root| (root.entity_slot_count(), root.relation_slot_count()))
            })
            .unwrap_or_default();
        InvariantRuntimeView::from_inputs(InvariantRuntimeInputs {
            selected_registry: crate::schema::SchemaRegistryCustody::Runtime(
                configuration.config.clone(),
            ),
            prior_registry: None,
            commit_work_budget: None,
            config: configuration.config,
            schema_contract_runtime: configuration.schema_contract_runtime,
            instrumentation: &self.services.instrumentation,
            #[cfg(test)]
            current_version_id: self.current_version_id(),
            entity_count,
            relation_count,
            version_depth: self.history().commit_count(),
            snapshot_pressure: self.visibility.active_snapshot_count() > 10,
        })
    }
}
impl RelationalPreparationRuntime {
    pub(crate) fn invariant_validation_view(
        &self,
        state: &crate::branch::root::RelationalBranchRootState,
    ) -> InvariantRuntimeView<'_> {
        InvariantRuntimeView::from_inputs(InvariantRuntimeInputs {
            selected_registry: crate::schema::SchemaRegistryCustody::Selected(
                self.execution_registry
                    .clone()
                    .unwrap_or_else(|| state.root().schema_authority().registry_arc()),
            ),
            prior_registry: self.home_target.as_ref().map(|_| {
                crate::schema::SchemaRegistryCustody::Selected(
                    state.root().schema_authority().registry_arc(),
                )
            }),
            commit_work_budget: self.commit_work_budget.clone(),
            config: std::sync::Arc::clone(&self.config),
            schema_contract_runtime: std::sync::Arc::clone(&self.schema_contract_runtime),
            instrumentation: &self.services.instrumentation,
            #[cfg(test)]
            current_version_id: self.current_version_id(),
            entity_count: state.entity_slot_count(),
            relation_count: state.relation_slot_count(),
            version_depth: self.current_version_id().0 as usize,
            snapshot_pressure: self.published_snapshot_count() > 10,
        })
    }
}
