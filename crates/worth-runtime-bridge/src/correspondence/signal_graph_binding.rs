use worth_signal::facade::SignalGraph;

use crate::adapter::RelationalCommittedPatchRequest;
use crate::facade::RuntimeBridge;

use super::{
    BridgeCorrespondenceRebindRequired, BridgeInstalledSemanticCorrespondence,
    BridgeSemanticDependencyCandidate, CorrespondenceAdmissionOutcome,
    CorrespondenceDeliveryOutcome,
};

/// The sole ordinary seam between one Bridge runtime and its executable
/// Signal graph. Binding once prevents callers from assembling a loose
/// runtime/graph pair independently at every correspondence transition.
pub struct BridgeSignalGraphBinding<'runtime, 'graph> {
    runtime: &'runtime RuntimeBridge,
    graph: &'graph mut SignalGraph,
}

impl<'runtime, 'graph> BridgeSignalGraphBinding<'runtime, 'graph> {
    pub(crate) fn admit(
        runtime: &'runtime RuntimeBridge,
        graph: &'graph mut SignalGraph,
    ) -> Result<Self, BridgeCorrespondenceRebindRequired> {
        let graph_instance_id = graph.installed_graph_capability().graph_instance_id();
        if let Some(registered_graph) = runtime
            .semantic_dependency_registry
            .signal_graph_instance_id()
        {
            if registered_graph != graph_instance_id {
                return Err(BridgeCorrespondenceRebindRequired::SignalGraphGeneration);
            }
            graph
                .claim_aspect_lowering_owner(&runtime.signal_aspect_lowering_owner)
                .map_err(|_| BridgeCorrespondenceRebindRequired::SignalGraphLoweringOwner)?;
        }
        Ok(Self { runtime, graph })
    }

    pub fn install_semantic_correspondence(
        &mut self,
        dependency: BridgeSemanticDependencyCandidate,
    ) -> CorrespondenceAdmissionOutcome {
        self.runtime
            .install_semantic_correspondence(dependency, self.graph)
    }

    pub fn deliver_installed_correspondence(
        &mut self,
        correspondence: &BridgeInstalledSemanticCorrespondence,
        request: RelationalCommittedPatchRequest,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> CorrespondenceDeliveryOutcome {
        self.runtime.deliver_installed_correspondence(
            correspondence,
            self.graph,
            request,
            execution,
        )
    }
}
