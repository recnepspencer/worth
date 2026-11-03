//! World retains the actual Native retirement phase and its inert completion.
use super::RuntimeWorldCloseDenial as Denial;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    change_source::RelationalRetainedObservationRetirementProgress as Progress,
    durability::{
        RelationalRecoveredPublicationCustody, RelationalRecoveredPublicationRetirement,
        RetiredRelationalRecoveredPublication,
    },
};
#[derive(Debug)]
pub(super) enum RecoveredNativeRetirement {
    Active(RelationalRecoveredPublicationRetirement),
    Retired(RetiredRelationalRecoveredPublication),
}
impl RecoveredNativeRetirement {
    pub(super) fn new(original: RelationalRecoveredPublicationCustody) -> Self {
        Self::Active(original.into_retirement())
    }
    pub(super) fn advance(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> (Self, Result<bool, Denial>) {
        match self {
            Self::Retired(retired) => (Self::Retired(retired), Ok(true)),
            Self::Active(mut task) => match task.advance(request) {
                Progress::Advanced => (Self::Active(task), Ok(false)),
                Progress::Busy => (Self::Active(task), Err(Denial::Busy)),
                Progress::Stopped(cause) => (Self::Active(task), Err(Denial::Execution(cause))),
                Progress::Refused(cause) => (
                    Self::Active(task),
                    Err(Denial::RecoveredNativeRetirement(cause)),
                ),
                Progress::Complete => match task.into_retired() {
                    Ok(retired) => (Self::Retired(retired), Ok(true)),
                    Err(task) => (Self::Active(task), Err(Denial::CloseProgressInvariant)),
                },
            },
        }
    }
}
