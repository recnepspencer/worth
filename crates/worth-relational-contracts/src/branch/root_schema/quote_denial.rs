//! Closed failures of schema allocation geometry and its invoking request.
use crate::execution::denial_cause::RelationalExecutionDenialCause;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaCompilationQuoteDenial {
    AllocationLayoutOverflow,
    Request(RelationalExecutionDenialCause),
}
impl From<SchemaCompilationQuoteDenial> for RelationalExecutionDenialCause {
    fn from(cause: SchemaCompilationQuoteDenial) -> Self {
        match cause {
            SchemaCompilationQuoteDenial::AllocationLayoutOverflow => Self::MemoryOverflow,
            SchemaCompilationQuoteDenial::Request(cause) => cause,
        }
    }
}
