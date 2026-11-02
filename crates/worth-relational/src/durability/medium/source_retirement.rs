//! A medium retires only its own physical capabilities, retaining its payload.
use super::RelationalDurableColdSource;
use worth_execution::ExecutionRequest;

#[derive(Debug)]
pub enum RelationalMediumRetirementDenial {
    Busy,
    Unsupported,
    ForeignSource,
    MissingBinding,
    SharedBinding,
    MediumMoved,
    UnresolvedPublication,
    TransitionReaderAttached,
    Execution(worth_execution::WorkCeilingDenial),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RelationalMediumRetirementProgress {
    Advanced,
    Complete,
}

pub(super) fn unsupported(
    request: &ExecutionRequest<'_, '_>,
    _source: &dyn RelationalDurableColdSource,
) -> Result<(), RelationalMediumRetirementDenial> {
    request
        .consult()
        .map_err(RelationalMediumRetirementDenial::Execution)?;
    Err(RelationalMediumRetirementDenial::Unsupported)
}
