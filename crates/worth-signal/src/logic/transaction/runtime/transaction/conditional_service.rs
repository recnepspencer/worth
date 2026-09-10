use crate::branch::owner_services::conditional_execution::{
    SignalConditionalDefinitionPublicationScope, SignalConditionalExecutionPort,
    SignalConditionalOperationScopeBinding,
};
use crate::data::conditional_execution::{
    InstalledSignalConditionalContract, SignalConditionalContractDefinition,
};
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;

use super::SignalTransaction;

impl<D, I, E, Ctx, T> SignalTransaction<'_, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn admits_conditional_operation_scope(
        &self,
        port: &SignalConditionalExecutionPort<D, I, T>,
    ) -> bool {
        self.conditional_operation_scope.as_ref().is_some_and(
            |scope: &SignalConditionalOperationScopeBinding| scope.admits_port(port, self.graph),
        )
    }

    pub(crate) fn admits_conditional_definition_publication(
        &self,
        scope: &SignalConditionalDefinitionPublicationScope,
    ) -> bool {
        self.conditional_operation_scope
            .as_ref()
            .is_some_and(|installed| installed.admits_definition_publication(scope))
    }

    pub(crate) fn conditional_execution_graph(&self) -> &SignalGraph {
        self.graph
    }

    pub(crate) fn conditional_execution_graph_mut(&mut self) -> &mut SignalGraph {
        self.graph
    }

    pub(crate) fn install_conditional_contract_within_owner(
        &mut self,
        claimant: &crate::data::aspect::SignalAspectLoweringOwner,
        node: NodeId,
        definition: SignalConditionalContractDefinition,
    ) -> Result<InstalledSignalConditionalContract, crate::data::error::SignalError> {
        self.ensure_rollback_packets();
        self.scratch
            .graph_patches
            .stage_original(self.graph, node)?;
        let worth_proof::TransitionOutcome::Success(capability) =
            self.graph.admit_installed_node(node)
        else {
            return Err(crate::data::error::SignalError::internal(
                "conditional installation target is stale",
            ));
        };
        self.graph
            .install_conditional_contract(claimant, capability, definition)
            .map_err(|denial| {
                crate::data::error::SignalError::internal(format!(
                    "conditional installation extension was denied: {denial:?}"
                ))
            })
    }

    pub(crate) fn apply_prepared_conditional_contract_within_owner(
        &mut self,
        prepared: crate::data::conditional_execution::PreparedSignalConditionalContract,
    ) -> Result<InstalledSignalConditionalContract, crate::data::error::SignalError> {
        self.ensure_rollback_packets();
        let node = prepared.contract().node();
        self.scratch
            .graph_patches
            .stage_original(self.graph, node)?;
        self.graph
            .apply_prepared_conditional_contract(prepared)
            .map_err(|denial| {
                crate::data::error::SignalError::internal(format!(
                    "prepared conditional installation was denied: {denial:?}"
                ))
            })
    }

    pub(crate) fn create_conditional_installation_node(&mut self) -> NodeId {
        self.ensure_rollback_packets();
        let node = self.graph.node().build();
        self.scratch.created_nodes.push(node);
        node
    }
}
