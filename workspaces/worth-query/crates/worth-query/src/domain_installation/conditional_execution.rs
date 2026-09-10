mod bridge_lowering;
mod compute_bridge;
mod evaluation;
mod installation;
mod reentry;
mod registry;

pub(crate) use bridge_lowering::query_location_from_bridge_candidate;
pub(crate) use compute_bridge::QueryComputeProvider;
pub(crate) use evaluation::{
    evaluate_bound_conditionals, evaluate_settled_projection_conditionals,
    WorthQueryConditionalEvaluationPass, WorthQueryConditionalEvaluationScope,
    WorthQueryConditionalEvaluationStop,
};
pub(crate) use installation::{
    PendingConditionalInstallation, PendingConditionalNode, PendingOwnedConditionalNode,
    WorthQueryConditionalComputeContextParts,
};
pub use installation::{
    WorthQueryConditionalComputeContext, WorthQueryConditionalDependencyInstallation,
    WorthQueryConditionalNodeComputeProvider, WorthQueryConditionalNodeInstallationDenial,
    WorthQueryOwnedConditionalDependencyInstallation,
};
pub(crate) use reentry::classify_signal_decision;
pub use reentry::{
    WorthQueryConditionalAdmissionDenial, WorthQueryConditionalOutcomeClass,
    WorthQueryConditionalProvenance, WorthQueryConditionalSemanticObservation,
    WorthQueryDeferredDomainOperation, WorthQueryDeferredWorkflowStage,
    WorthQueryDeferredWorkflowStart,
};
pub use registry::WorthQueryConditionalExecutionIndexRebuildReport;
pub(crate) use registry::{
    WorthQueryConditionalExecutionRegistry, WorthQueryInstalledConditionalNode,
};
