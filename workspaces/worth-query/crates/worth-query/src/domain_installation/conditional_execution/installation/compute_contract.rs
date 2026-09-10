#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorthQueryConditionalComputeContext {
    location: worth_query_installation::facade::WorthQueryConditionalNodeLocation,
    operation_identity: String,
    binding_identity: String,
    basis_identity: String,
    workflow_run_identity: Option<String>,
    snapshot_identity: String,
    attempt: u64,
    execution_resources: crate::domain_installation::WorthQueryExecutionResourceAttemptEvidence,
    resource_envelope:
        std::sync::Arc<worth_query_installation::facade::WorthQueryExecutionResourceEnvelope>,
}

pub(crate) struct WorthQueryConditionalComputeContextParts {
    pub(crate) location: worth_query_installation::facade::WorthQueryConditionalNodeLocation,
    pub(crate) operation_identity: String,
    pub(crate) binding_identity: String,
    pub(crate) basis_identity: String,
    pub(crate) workflow_run_identity: Option<String>,
    pub(crate) snapshot_identity: String,
    pub(crate) attempt: u64,
    pub(crate) execution_resources:
        crate::domain_installation::WorthQueryExecutionResourceAttemptEvidence,
    pub(crate) resource_envelope:
        std::sync::Arc<worth_query_installation::facade::WorthQueryExecutionResourceEnvelope>,
}

impl WorthQueryConditionalComputeContext {
    pub fn location(&self) -> &worth_query_installation::facade::WorthQueryConditionalNodeLocation {
        &self.location
    }

    pub fn operation_identity(&self) -> &str {
        &self.operation_identity
    }

    pub fn binding_identity(&self) -> &str {
        &self.binding_identity
    }

    pub fn basis_identity(&self) -> &str {
        &self.basis_identity
    }

    pub fn workflow_run_identity(&self) -> Option<&str> {
        self.workflow_run_identity.as_deref()
    }

    pub fn snapshot_identity(&self) -> &str {
        &self.snapshot_identity
    }

    pub const fn attempt(&self) -> u64 {
        self.attempt
    }

    pub fn execution_resources(
        &self,
    ) -> &crate::domain_installation::WorthQueryExecutionResourceAttemptEvidence {
        &self.execution_resources
    }

    pub fn resource_envelope(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryExecutionResourceEnvelope {
        &self.resource_envelope
    }

    pub(crate) fn new(parts: WorthQueryConditionalComputeContextParts) -> Self {
        Self {
            location: parts.location,
            operation_identity: parts.operation_identity,
            binding_identity: parts.binding_identity,
            basis_identity: parts.basis_identity,
            workflow_run_identity: parts.workflow_run_identity,
            snapshot_identity: parts.snapshot_identity,
            attempt: parts.attempt,
            execution_resources: parts.execution_resources,
            resource_envelope: parts.resource_envelope,
        }
    }
}

pub trait WorthQueryConditionalNodeComputeProvider<D, O, F>: Send + Sync + 'static {
    /// Complete owner-native compute meaning used when comparing a reinstalled
    /// provider. Runtime counters and observation state do not belong here.
    type SemanticContract: Eq + Send + Sync + 'static;

    fn semantic_contract(&self) -> Self::SemanticContract;

    fn retained_heap_bytes(
        &self,
        semantic_contract: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    >;

    fn execution_resource_support(
        &self,
    ) -> crate::domain_installation::WorthQueryExecutionResourceSupport;

    fn compute(
        &self,
        context: &WorthQueryConditionalComputeContext,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String>;
}
