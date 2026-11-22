//! A member compute cannot open a second installed host advancement.
// Execution carries this same refusal context onto every descendant worker.
pub struct MemberComputeCustody {
    _context: worth_execution::ExecutionOwnerContext,
}
impl MemberComputeCustody {
    pub fn enter() -> Self {
        Self {
            _context: worth_execution::ExecutionOwnerContext::enter(),
        }
    }
}
pub fn active() -> bool {
    worth_execution::ExecutionOwnerContext::is_active()
}
