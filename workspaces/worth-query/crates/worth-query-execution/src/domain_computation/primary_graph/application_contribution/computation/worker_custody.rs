//! A member compute cannot open a second installed host advancement.
// Execution carries this same refusal context onto every descendant worker.
pub(in crate::domain_computation::primary_graph) struct MemberComputeCustody {
    _context: worth_execution::ExecutionOwnerContext,
}
impl MemberComputeCustody {
    pub(super) fn enter() -> Self {
        Self {
            _context: worth_execution::ExecutionOwnerContext::enter(),
        }
    }
}
pub(in crate::domain_computation::primary_graph) fn active() -> bool {
    worth_execution::ExecutionOwnerContext::is_active()
}

#[cfg(test)]
mod tests;
