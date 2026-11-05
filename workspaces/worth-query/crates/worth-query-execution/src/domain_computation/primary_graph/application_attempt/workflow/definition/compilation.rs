//! Discardable execution meaning reconstructed from performed definition facts.

pub(crate) mod plan;
pub(crate) mod publication_binding;
pub(crate) mod reconstruction;
mod reuse;

pub(in crate::domain_computation::primary_graph) use plan::{
    CompiledWorkflowAssessmentApplicability, CompiledWorkflowConnection,
    CompiledWorkflowConnectionKind, CompiledWorkflowDefinition, CompiledWorkflowNode,
    CompiledWorkflowNodeKind,
};
pub(in crate::domain_computation::primary_graph) use reconstruction::WorkflowDefinitionCompilationPosture;
pub(in crate::domain_computation::primary_graph) use reconstruction::{
    compile_node, observe_connection, reconstruct_compiled_definition,
};
pub(in crate::domain_computation::primary_graph) use reuse::WorkflowDefinitionCompilationReuse;
