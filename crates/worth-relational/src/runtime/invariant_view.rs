//! Select the runtime's immutable validation inputs from one owner epoch.
use crate::runtime::{RelationalPreparationRuntime, RelationalRuntime};
use crate::validation::engine::{InvariantRuntimeInputs, InvariantRuntimeView};
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;

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
        self.candidate_validation_inputs().select(state)
    }
}

impl<'a> From<&'a crate::mvcc::RelationalTransactionValidationInput>
    for crate::validation::engine::request::InvariantPreparationSource<'a>
{
    fn from(input: &'a crate::mvcc::RelationalTransactionValidationInput) -> Self {
        Self {
            operation: input.control(),
            retention: &(*input.basis().owner_inner()).retention_binding,
        }
    }
}

impl From<&crate::mvcc::RelationalTransactionValidationInput>
    for crate::validation::candidate_source::ManagedSourceIssueInputs
{
    fn from(input: &crate::mvcc::RelationalTransactionValidationInput) -> Self {
        let catalog_receipt = input
            .home_target()
            .map(|target| target.meaning.catalog_receipt())
            .or_else(|| {
                crate::runtime::home_program_transition::retain_root_execution_contracts(
                    &(*input.basis().owner_inner()).root,
                )
                .map(|meaning| meaning.catalog_receipt())
            });
        Self {
            physical_basis: input.basis().clone(),
            posture: input.intent().materialization_mode(),
            materialization: input.intent().materialization_source().cloned(),
            restoration: input.intent().restoration_origin().cloned(),
            home_target_step: input.home_target().map(|target| target.step),
            home_coverage: input.home_target_coverage().cloned(),
            catalog_receipt,
        }
    }
}
