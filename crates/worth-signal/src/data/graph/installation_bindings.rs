//! Fresh program bindings reused while replaying several authored branch images.
use super::{SignalAuthoredGraphReadmission, SignalGraph};
use crate::data::aspect::SignalAspectLoweringOwner;
use crate::data::persistent_ord_set::PersistentOrdSet;
use crate::durability::{SignalAuthoredImage, SignalAuthoredImageDenial};
use crate::runtime_policy::InstalledSignalRuntimePolicy;
use crate::schema::data::SignalSchemaRegistry;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

/// Contains no arena, topology, values or live branch. Its strong lifecycle
/// share belongs to the original consuming owner and is dropped at owner close.
pub(crate) struct SignalGraphInstallationBindings {
    instance_id: u64,
    lifecycle_token: Arc<()>,
    schema_registry: Arc<SignalSchemaRegistry>,
    lowering_owner: Option<SignalAspectLoweringOwner>,
    authorization_policies: PersistentOrdSet<[u8; 32]>,
    runtime_policy: InstalledSignalRuntimePolicy,
}
impl SignalGraphInstallationBindings {
    pub(crate) fn consume(graph: Box<SignalGraph>) -> Self {
        Self::capture(&graph)
    }
    pub(crate) fn capture(graph: &SignalGraph) -> Self {
        Self {
            instance_id: graph.instance_id,
            lifecycle_token: Arc::clone(&graph.lifecycle_token),
            schema_registry: Arc::clone(&graph.schema_registry),
            lowering_owner: graph.aspect_lowering_owner.clone(),
            authorization_policies: graph.authorization_policy_identities.clone(),
            runtime_policy: graph.observation.installed_policy(),
        }
    }
    pub(crate) fn readmit(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        image: &SignalAuthoredImage,
    ) -> Result<SignalAuthoredGraphReadmission, SignalAuthoredImageDenial> {
        let mut graph = Box::new(SignalGraph::new());
        graph.instance_id = self.instance_id;
        graph.lifecycle_token = Arc::clone(&self.lifecycle_token);
        graph.schema_registry = Arc::clone(&self.schema_registry);
        graph.aspect_lowering_owner = self.lowering_owner.clone();
        graph.authorization_policy_identities = self.authorization_policies.clone();
        graph.observation.install_policy(self.runtime_policy);
        graph.rebind_observation_capture_state();
        graph.initial_empty_origin = false;
        SignalGraph::readmit_authored_rows(graph, execution, image)
    }
}
