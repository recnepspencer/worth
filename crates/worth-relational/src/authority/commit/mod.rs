pub(crate) mod phases;
pub(crate) mod pipeline;
pub(crate) mod preparation;
mod publication;
pub(crate) mod semantic_plan;
pub(crate) mod structural_summary;
mod touched_scope;

pub(crate) use pipeline::CommitResultSeal;
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
