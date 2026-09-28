//! Workflow condition expressions over installed query results: the operand
//! sources an application supplies, their evaluation under the shared
//! language, and the identity that fences a transition to those sources.

mod currentness;
mod evaluation;
mod inputs;

pub(in crate::domain_computation::primary_graph) use currentness::supporting_identity;
pub(in crate::domain_computation::primary_graph) use evaluation::evaluate_condition;
pub(in crate::domain_computation::primary_graph) use inputs::supplies;
pub use inputs::WorthQueryWorkflowConditionSources;
