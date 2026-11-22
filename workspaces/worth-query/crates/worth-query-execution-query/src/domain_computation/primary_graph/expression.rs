//! Workflow condition expressions over installed query results: the operand
//! sources an application supplies, their evaluation under the shared
//! language, and the identity that fences a transition to those sources.

mod currentness;
mod evaluation;
mod inputs;
#[cfg(test)]
mod tests;

pub use currentness::supporting_identity;
pub use evaluation::evaluate_condition;
pub use inputs::supplies;
pub use inputs::WorthQueryWorkflowConditionSources;
