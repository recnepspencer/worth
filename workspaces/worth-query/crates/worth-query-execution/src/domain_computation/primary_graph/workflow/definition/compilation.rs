//! Discardable execution meaning reconstructed from performed definition facts.

mod plan;
mod reconstruction;

pub(in crate::domain_computation::primary_graph) use plan::{
    CompiledWorkflowDefinition, CompiledWorkflowNode, CompiledWorkflowNodeKind,
};
pub(in crate::domain_computation::primary_graph) use reconstruction::reconstruct_compiled_definition;
pub(in crate::domain_computation::primary_graph) use reconstruction::WorkflowDefinitionCompilationPosture;
