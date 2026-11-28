mod conditions;

use crate::data::aspect::AspectMask;
use crate::data::comparator::VersionComparatorPolicy;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::{
    ArtifactPolicyClass, AuthorityPolicy, ContextRequirement, EquivalenceContract, MaintenanceMode,
    NodeContract, NodeEvaluationConfig, NodeProjectionContract, PathClass,
};
use crate::data::output::PartitionSubscription;
use crate::data::output_equivalence::OutputEquivalencePolicy;
use crate::data::reuse::{ArtifactEquivalenceContract, NodeReuseContract};

use crate::schema::data::{SignalSchemaId, SignalSchemaName};

/// Fluent node builder for accessible public graph configuration.
///
/// This is the intended front door for most node setup. Prefer these helpers
/// over constructing `NodeEvaluationConfig` manually unless you are exposing
/// a higher-level host abstraction.
pub struct NodeBuilder<'a> {
    graph: &'a mut SignalGraph,
    config: NodeEvaluationConfig,
}

impl<'a> NodeBuilder<'a> {
    pub(crate) fn new(graph: &'a mut SignalGraph) -> Self {
        Self {
            graph,
            config: NodeEvaluationConfig::default(),
        }
    }

    /// Replace the full node contract.
    pub fn with_contract(mut self, contract: NodeContract) -> Self {
        self.config.contract = contract;
        self
    }

    /// Bind this node to a schema descriptor by semantic name and inherit its default contract.
    pub fn schema_name(
        mut self,
        schema_name: impl Into<String>,
    ) -> Result<Self, crate::data::error::SignalError> {
        let schema_name = SignalSchemaName::new(schema_name);
        let descriptor = self
            .graph
            .schema_registry()
            .resolve_by_name(&schema_name)
            .ok_or_else(|| {
                crate::data::error::SignalError::invalid_input(format!(
                    "unknown signal schema `{}`",
                    schema_name.as_str()
                ))
            })?;
        self.config.schema_binding = Some(descriptor.binding());
        self.config.contract = descriptor.default_contract().clone();
        Ok(self)
    }

    /// Bind this node to a schema descriptor by registry id and inherit its default contract.
    pub fn schema_id(
        mut self,
        schema_id: SignalSchemaId,
    ) -> Result<Self, crate::data::error::SignalError> {
        let descriptor = self
            .graph
            .schema_registry()
            .resolve_by_id(schema_id)
            .ok_or_else(|| {
                crate::data::error::SignalError::invalid_input(format!(
                    "unknown signal schema id `{}`",
                    schema_id.0
                ))
            })?;
        self.config.schema_binding = Some(descriptor.binding());
        self.config.contract = descriptor.default_contract().clone();
        Ok(self)
    }

    /// Declare which upstream aspects this node reads.
    pub fn reads_aspects(mut self, aspects: impl Into<AspectMask>) -> Self {
        self.config.contract = self.config.contract.with_reads(aspects);
        self
    }

    /// Declare which aspects this node produces.
    pub fn produces_aspects(mut self, aspects: impl Into<AspectMask>) -> Self {
        self.config.contract = self.config.contract.with_produces(aspects);
        self
    }

    /// Declare a partition scope contract for this node.
    pub fn with_partition_scope(
        mut self,
        partition_scope: impl Into<PartitionSubscription>,
    ) -> Self {
        self.config.contract = self.config.contract.with_partition_scope(partition_scope);
        self
    }

    /// Declare additional context required to evaluate this node.
    pub fn requires_context(mut self, required_context: ContextRequirement) -> Self {
        self.config.contract = self.config.contract.with_required_context(required_context);
        self
    }

    /// Replace the full equivalence contract for this node.
    pub fn equivalence(mut self, equivalence: EquivalenceContract) -> Self {
        self.config.contract = self.config.contract.with_equivalence(equivalence);
        self
    }

    /// Replace the full projection contract for this node.
    pub fn projection_contract(mut self, projection_contract: NodeProjectionContract) -> Self {
        self.config.contract = self
            .config
            .contract
            .with_projection_contract(projection_contract);
        self
    }

    /// Declare whether this node belongs to an operational or rich path.
    pub fn path_class(mut self, path_class: PathClass) -> Self {
        self.config.contract = self.config.contract.with_path_class(path_class);
        self
    }

    /// Declare whether this node is incremental-only, rebuild-capable, or adaptive.
    pub fn maintenance_mode(mut self, maintenance_mode: MaintenanceMode) -> Self {
        self.config.contract = self.config.contract.with_maintenance_mode(maintenance_mode);
        self
    }

    /// Declare the artifact policy class expected by this node.
    pub fn artifact_policy(mut self, artifact_policy: ArtifactPolicyClass) -> Self {
        self.config.contract = self.config.contract.with_artifact_policy(artifact_policy);
        self
    }

    /// Declare whether this node must wait for authority or may reconcile later.
    pub fn authority_policy(mut self, authority_policy: AuthorityPolicy) -> Self {
        self.config.contract = self.config.contract.with_authority_policy(authority_policy);
        self
    }

    /// Replace the full reuse contract for this node.
    pub fn reuse_contract(mut self, reuse_contract: NodeReuseContract) -> Self {
        self.config.contract = self.config.contract.with_reuse_contract(reuse_contract);
        self
    }

    /// Declare the semantic equivalence boundaries required for artifact reuse.
    pub fn artifact_equivalence_contract(
        mut self,
        equivalence_contract: ArtifactEquivalenceContract,
    ) -> Self {
        self.config.contract = self
            .config
            .contract
            .with_artifact_equivalence_contract(equivalence_contract);
        self
    }

    /// Explicitly admit cross-identity persistent matching for this node.
    pub fn cross_identity_persistent_matching(mut self) -> Self {
        self.config.contract = self
            .config
            .contract
            .with_cross_identity_persistent_matching();
        self
    }

    /// Explicitly admit partial artifact splicing for this node.
    pub fn partial_artifact_splicing(mut self) -> Self {
        self.config.contract = self.config.contract.with_partial_artifact_splicing();
        self
    }

    /// Control whether full reuse certification must be retained for this node.
    pub fn retain_reuse_certification(mut self, retain_certification: bool) -> Self {
        self.config.contract = self
            .config
            .contract
            .with_reuse_certification_retention(retain_certification);
        self
    }

    /// Override consumer-side dependency comparison for this node.
    pub fn dependency_comparator(mut self, comparator: VersionComparatorPolicy) -> Self {
        self.config.comparator = Some(comparator);
        self
    }

    #[deprecated(note = "use dependency_comparator")]
    pub fn comparator(self, comparator: VersionComparatorPolicy) -> Self {
        self.dependency_comparator(comparator)
    }

    /// Configure producer-side semantic output equivalence for this node.
    pub fn output_equivalence(mut self, policy: OutputEquivalencePolicy) -> Self {
        self.config.contract = self.config.contract.with_output_equivalence(&policy);
        self.config.output_equivalence = policy;
        self
    }

    /// Convenience override for tolerance-based comparison.
    ///
    /// Use this when small upstream version drift should not count as a
    /// meaningful change for this node. This is different from
    /// `delta_threshold(...)`, which is an evaluation condition.
    pub fn tolerance(self, epsilon: u64) -> Self {
        self.dependency_comparator(VersionComparatorPolicy::Tolerance { epsilon })
    }

    /// Use output-identity-aware downstream suppression for this node.
    ///
    /// This is useful when the node can detect that the logical output did not
    /// change even though evaluation happened.
    pub fn output_identity(self) -> Self {
        self.output_equivalence(OutputEquivalencePolicy::OutputIdentity)
    }

    /// Declare that this node reports partition-aware output changes.
    pub fn partitioned_output(mut self) -> Self {
        self.config.partitioned_output = true;
        self
    }

    /// Build the node into the graph.
    pub fn build(self) -> NodeId {
        self.graph.create_node_with_config(self.config)
    }
}
