//! Shared original custody for the installed and retained Source catalogs.
use worth_execution::ExecutionMemoryReservation;

pub struct CustomInvariantCompilationFunding {
    pub _memory: Option<ExecutionMemoryReservation>,
}
impl std::fmt::Debug for CustomInvariantCompilationFunding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CustomInvariantCompilationFunding")
    }
}
