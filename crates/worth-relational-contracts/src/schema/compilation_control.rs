//! Consultation policy for one shared metadata compilation, never retained.
use crate::execution::denial_cause::RelationalExecutionDenialCause as Denial;
use worth_execution::ExecutionRequest;

#[derive(Clone, Copy)]
pub enum SchemaCompilationControl<'request, 'authority> {
    Configuration,
    Request(ExecutionRequest<'request, 'authority>),
}
impl SchemaCompilationControl<'_, '_> {
    pub fn checkpoint(self) -> Result<(), Denial> {
        match self {
            Self::Configuration => Ok(()),
            Self::Request(request) => crate::execution::request_custody::check_request(&request),
        }
    }
    pub fn checked_bytes(self, bytes: Option<usize>) -> Result<usize, Denial> {
        self.checkpoint()?;
        bytes.ok_or(Denial::MemoryOverflow)
    }
    pub fn reserve(
        self,
        bytes: usize,
    ) -> Result<Option<worth_execution::ExecutionMemoryReservation>, Denial> {
        self.checkpoint()?;
        match self {
            Self::Configuration => Ok(None),
            Self::Request(request) => request
                .reserve_retained_memory(u64::try_from(bytes).map_err(|_| Denial::MemoryOverflow)?)
                .map_err(Denial::from),
        }
    }
}
