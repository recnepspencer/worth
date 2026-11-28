//! World retains the actual Native retirement phase and its inert completion.
use super::RuntimeWorldCloseDenial as Denial;
use worth_execution::{CooperativeRetirementState, CooperativeStep, ExecutionRequest};
use worth_relational::facade::{
    change_source::RelationalRetainedObservationRetirementProgress as Progress,
    durability::{
        RelationalRecoveredPublicationCustody, RelationalRecoveredPublicationRetirement,
        RetiredRelationalRecoveredPublication,
    },
};

#[derive(Debug)]
pub(super) enum RecoveredNativeRetirement {
    Active(
        CooperativeRetirementState<
            RelationalRecoveredPublicationRetirement,
            RetiredRelationalRecoveredPublication,
        >,
    ),
    Publication(crate::history::ReadmittedPublicationRetirement),
}

impl RecoveredNativeRetirement {
    pub(super) fn new(original: RelationalRecoveredPublicationCustody) -> Self {
        Self::Active(CooperativeRetirementState::Retiring(Some(
            original.into_retirement(),
        )))
    }

    pub(super) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        owner: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<bool, Denial> {
        let task = match self {
            Self::Publication(task) => return task.advance(request, owner),
            Self::Active(task) => task,
        };
        task.advance_with(
            request,
            |task, request| match task.advance(request) {
                Progress::Advanced => Ok(CooperativeStep::Continue),
                Progress::Busy => Ok(CooperativeStep::Busy),
                Progress::Stopped(cause) => Ok(CooperativeStep::Stopped(cause)),
                Progress::Refused(cause) => Err(Denial::RecoveredNativeRetirement(cause)),
                Progress::Complete => Ok(CooperativeStep::Complete(())),
            },
            |task| {
                task.into_retired()
                    .map_err(|task| (task, Denial::CloseProgressInvariant))
            },
        )?
        .into_progress(Denial::Busy, Denial::Execution)
        .map(|complete| complete.is_some())
    }
}
