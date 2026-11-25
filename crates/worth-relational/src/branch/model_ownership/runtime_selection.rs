//! Bind model-ownership reads to this runtime's selected basis and history.
use super::*;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::runtime::RelationalRuntime;
use worth_execution::ExecutionRequest;
use worth_relational_mutation::internal_port::branch::model_ownership::graph_admission::RelationalModelOwnershipOwnerPort as _;
use RelationalOwnedModelSelectionDenial as Denial;

impl RelationalRuntime {
    pub fn observe_model_owner(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        ownership: &RelationalModelOwnership,
        subject: EntityId,
    ) -> Result<Option<EntityId>, RelationalModelOwnershipDenial> {
        let (owner, _memory) =
            basis::observe(self.runtime_instance_id(), execution, basis, |read| {
                ownership.as_ref().owner(read, subject)
            })?;
        Ok(owner)
    }
}

impl RelationalRuntime {
    pub fn prepare_owned_model_entity_boundary(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        current: &AdmittedRelationalBranchBasis,
        selected: &AdmittedRelationalBranchBasis,
        ownership: &RelationalModelOwnership,
        model: EntityId,
    ) -> Result<RelationalOwnedModelEntityBoundary, RelationalModelOwnershipDenial> {
        // Both closure traversals spend one request meter, including on the
        // serial backend. Their retained results coexist under the same budget.
        let mut result = None;
        execution
            .run_native_with_checkpoint(|context| {
                result = Some(selection::prepare_entity_boundary(
                    self.runtime_instance_id(),
                    execution,
                    context,
                    current,
                    selected,
                    ownership,
                    model,
                ));
            })
            .map_err(RelationalModelOwnershipDenial::Request)?;
        result.expect("completed entity boundary preparation")
    }
}

impl RelationalRuntime {
    pub fn prepare_owned_model_graph_selection(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        current: &AdmittedRelationalBranchBasis,
        selected: &AdmittedRelationalBranchBasis,
        ownership: &RelationalModelOwnership,
        installed_ownership: &RelationalModelOwnership,
        model: EntityId,
    ) -> Result<RelationalOwnedModelGraphSelection, Denial> {
        self.prepare_owned_model_graph_selection_with_companions(
            execution,
            current,
            selected,
            ownership,
            installed_ownership,
            model,
            RelationalOwnedGraphCompanions::default(),
        )
    }

    /// As `prepare_owned_model_graph_selection`, with companion records
    /// outside the owned closure that follow the selection (tree-based undo's
    /// reversal facts, public #56 ruling (u)).
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_owned_model_graph_selection_with_companions(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        current: &AdmittedRelationalBranchBasis,
        selected: &AdmittedRelationalBranchBasis,
        ownership: &RelationalModelOwnership,
        installed_ownership: &RelationalModelOwnership,
        model: EntityId,
        companions: RelationalOwnedGraphCompanions,
    ) -> Result<RelationalOwnedModelGraphSelection, Denial> {
        let mut result = None;
        execution
            .run_native_with_checkpoint(|context| {
                result = Some((|| {
                    let entities = selection::prepare_entity_boundary(
                        self.runtime_instance_id(),
                        execution,
                        context,
                        current,
                        selected,
                        ownership,
                        model,
                    )?;
                    let bytes = ownership
                        .retention_bytes(context)?
                        .saturating_add(installed_ownership.retention_bytes(context)?);
                    crate::execution::request_stage::checkpoint(context, bytes).map_err(
                        |stop| {
                            Denial::Ownership(super::RelationalModelOwnershipDenial::Request(
                                worth_execution::WorkCeilingDenial::Stopped(stop),
                            ))
                        },
                    )?;
                    let meaning_memory =
                        execution.reserve_retained_memory(bytes).map_err(|stop| {
                            Denial::Ownership(super::RelationalModelOwnershipDenial::Request(stop))
                        })?;
                    graph_selection::prepare(
                        self.history.canonical_read(),
                        execution,
                        context,
                        entities,
                        ownership,
                        installed_ownership,
                        meaning_memory,
                        companions,
                    )
                })());
            })
            .map_err(|cause| {
                Denial::Ownership(super::RelationalModelOwnershipDenial::Request(cause))
            })?;
        result.expect("completed graph selection preparation")
    }
}
