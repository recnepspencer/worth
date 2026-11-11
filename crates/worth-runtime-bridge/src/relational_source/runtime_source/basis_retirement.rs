//! Progress of consuming the source's original basis registrations.
use worth_execution::WorkCeilingDenial;
use worth_relational::facade::change_source::{
    RelationalRetainedObservationRetirementDenial, RelationalRetainedObservationRetirementProgress,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalBridgeBasisRetirementDenial {
    ForeignSource,
    RegistrationMismatch,
    CommitIndexMismatch,
    Native(RelationalRetainedObservationRetirementDenial),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalBridgeBasisRetirementProgress {
    Advanced,
    Busy,
    Stopped(WorkCeilingDenial),
    Refused(RelationalBridgeBasisRetirementDenial),
    Complete,
}

impl From<RelationalRetainedObservationRetirementProgress>
    for RelationalBridgeBasisRetirementProgress
{
    fn from(value: RelationalRetainedObservationRetirementProgress) -> Self {
        use RelationalRetainedObservationRetirementProgress as Native;
        match value {
            Native::Advanced => Self::Advanced,
            Native::Busy => Self::Busy,
            Native::Stopped(cause) => Self::Stopped(cause),
            Native::Refused(cause) => {
                Self::Refused(RelationalBridgeBasisRetirementDenial::Native(cause))
            }
            Native::Complete => Self::Complete,
        }
    }
}
