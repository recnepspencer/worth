//! Discardable execution meaning reconstructed from performed definition facts.

mod plan;
mod publication_binding;
mod reconstruction;
mod reuse;

pub(in crate::domain_computation::primary_graph) use plan::{
    CompiledWorkflowDefinition, CompiledWorkflowNode, CompiledWorkflowNodeKind,
};
pub(in crate::domain_computation::primary_graph) use reconstruction::reconstruct_compiled_definition;
pub(in crate::domain_computation::primary_graph) use reconstruction::WorkflowDefinitionCompilationPosture;
pub(in crate::domain_computation::primary_graph) use reuse::WorkflowDefinitionCompilationReuse;
pub use reuse::WorthQueryWorkflowCompilationReuseCounters;
