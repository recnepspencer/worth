//! Closed failures of schema allocation geometry and its invoking request.
use crate::execution::RelationalExecutionDenialCause;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SchemaCompilationQuoteDenial {
    AllocationLayoutOverflow,
    Request(RelationalExecutionDenialCause),
}
impl From<SchemaCompilationQuoteDenial> for crate::branch::RelationalBranchRootCaptureDenial {
    fn from(cause: SchemaCompilationQuoteDenial) -> Self {
        match cause {
            SchemaCompilationQuoteDenial::AllocationLayoutOverflow => {
                Self::AllocationLayoutOverflow
            }
            SchemaCompilationQuoteDenial::Request(cause) => Self::Request(cause),
        }
    }
}
impl From<SchemaCompilationQuoteDenial> for RelationalExecutionDenialCause {
    fn from(cause: SchemaCompilationQuoteDenial) -> Self {
        match cause {
            SchemaCompilationQuoteDenial::AllocationLayoutOverflow => Self::MemoryOverflow,
            SchemaCompilationQuoteDenial::Request(cause) => cause,
        }
    }
}
