//! Retire the original admitted basis/lease pairs without destroying payloads.
use super::{ReadmittedRelationalAssociatedEffect, RelationalRecoveredPublicationCustody};
use crate::change_source::{
    RelationalRetainedObservationRetirement as Task,
    RelationalRetainedObservationRetirementProgress as Progress,
    RetiredRelationalRetainedObservation,
};
use worth_execution::ExecutionRequest;

#[derive(Debug)]
#[must_use]
pub struct RelationalRecoveredPublicationRetirement {
    _effect: ReadmittedRelationalAssociatedEffect,
    predecessor: Option<LeaseRetirement>,
    successor: Option<LeaseRetirement>,
    complete: bool,
}
#[derive(Debug)]
enum LeaseRetirement {
    Active(Task),
    Retired(RetiredRelationalRetainedObservation),
}

/// Both original pairs are disarmed; no observation or admission can be issued.
#[derive(Debug)]
#[must_use]
pub struct RetiredRelationalRecoveredPublication {
    _original: RelationalRecoveredPublicationRetirement,
}

impl RelationalRecoveredPublicationCustody {
    pub fn into_retirement(self) -> RelationalRecoveredPublicationRetirement {
        let (effect, before, before_lease, after, after_lease) = self.into_retained_parts();
        RelationalRecoveredPublicationRetirement {
            _effect: effect,
            predecessor: Some(LeaseRetirement::Active(Task::from_basis_and_lease(
                before,
                before_lease,
            ))),
            successor: Some(LeaseRetirement::Active(Task::from_basis_and_lease(
                after,
                after_lease,
            ))),
            complete: false,
        }
    }
}
impl RelationalRecoveredPublicationRetirement {
    /// One nonwaiting original phase. Complete means BOTH issued pairs retired.
    pub fn advance(&mut self, request: &ExecutionRequest<'_, '_>) -> Progress {
        if self.complete {
            return Progress::Complete;
        }
        if let Err(cause) = request.consult() {
            return Progress::Stopped(cause);
        }
        match advance_lease(&mut self.predecessor, request) {
            Ok(false) => return Progress::Advanced,
            Ok(true) => {}
            Err(progress) => return progress,
        }
        match advance_lease(&mut self.successor, request) {
            Ok(false) => Progress::Advanced,
            Ok(true) => {
                self.complete = true;
                Progress::Complete
            }
            Err(progress) => progress,
        }
    }
    pub fn into_retired(self) -> Result<RetiredRelationalRecoveredPublication, Self> {
        if self.complete {
            Ok(RetiredRelationalRecoveredPublication { _original: self })
        } else {
            Err(self)
        }
    }
}
impl RetiredRelationalRecoveredPublication {
    /// Explicitly unbudgeted destruction, after operational owner retirement.
    pub fn reclaim(self) {
        drop(self);
    }
}
fn advance_lease(
    slot: &mut Option<LeaseRetirement>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<bool, Progress> {
    let original = slot.take().expect("original recovered lease phase");
    let (next, result) = match original {
        LeaseRetirement::Retired(retired) => (LeaseRetirement::Retired(retired), Ok(true)),
        LeaseRetirement::Active(mut task) => match task.advance(request) {
            Progress::Advanced => (LeaseRetirement::Active(task), Ok(false)),
            Progress::Busy => (LeaseRetirement::Active(task), Err(Progress::Busy)),
            Progress::Stopped(cause) => (LeaseRetirement::Active(task), Err(Progress::Stopped(cause))),
            Progress::Refused(cause) => (LeaseRetirement::Active(task), Err(Progress::Refused(cause))),
            Progress::Complete => match task.into_retired() {
                Ok(retired) => (LeaseRetirement::Retired(retired), Ok(true)),
                Err(task) => (LeaseRetirement::Active(task), Err(Progress::Refused(crate::change_source::RelationalRetainedObservationRetirementDenial::AccountingMismatch))),
            },
        },
    };
    *slot = Some(next);
    result
}
