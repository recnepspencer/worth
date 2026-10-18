//! The member's actual row and custody share one refusal ending.
use super::super::frontier::RequiredMember;
use super::member_stop::StopRecording;
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry;

pub(super) enum RefusalRow<'a> {
    Member,
    HeldHead(&'a WorthQueryOutputDemandKey),
}
pub(super) enum RefusalEnd {
    Held,
    QueueBudget,
}
/// A caller keeps its continuation for recovery; a queue refusal ends it.
pub(super) enum RefusalCustody<'a, Schema: ApplicationSchema> {
    Caller,
    Queue(&'a mut RequiredContinuations<Schema>),
}

pub(super) struct RefusingMember<'a, Schema: ApplicationSchema> {
    pub registry: &'a WorthQueryOutputDemandRegistry,
    pub member: &'a mut RequiredMember,
    pub row: RefusalRow<'a>,
    pub custody: RefusalCustody<'a, Schema>,
}
impl<'a, Schema: ApplicationSchema + 'static> RefusingMember<'a, Schema> {
    pub(super) fn from_driver(
        registry: &'a WorthQueryOutputDemandRegistry,
        member: &'a mut RequiredMember,
        row: RefusalRow<'a>,
        queue_custody: &'a mut RequiredContinuations<Schema>,
    ) -> Self {
        let custody = if member.caller {
            RefusalCustody::Caller
        } else {
            RefusalCustody::Queue(queue_custody)
        };
        Self {
            registry,
            member,
            row,
            custody,
        }
    }

    pub(super) fn end(
        self,
        stop: WorthQueryOutputDemandDenial,
        recording: StopRecording,
        queue: &mut QueuedRequiredRoots,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<RefusalEnd, WorthQueryOutputDemandDenial> {
        let mut stop = member_stop(stop)?;
        let row = match self.row {
            RefusalRow::Member => self.member.key(),
            RefusalRow::HeldHead(head) => head,
        };
        recording.record_if_needed(self.registry, row, &stop);
        // A queued member's native terminal belongs to the advancing caller's
        // cleanup handoff, never to a suppressed registry diagnostic.
        if let Some(cleanup) = stop.take_managed_cleanup() {
            return Err(stop.with_managed_cleanup(cleanup));
        }
        match self.custody {
            RefusalCustody::Caller => return Err(stop),
            RefusalCustody::Queue(custody) => custody.end_refused(&stop),
        }
        self.member.state = MemberState::Pending;
        if matches!(
            stop.kind(),
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
                | WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
        ) {
            return Ok(RefusalEnd::QueueBudget);
        }
        if let Some(queued) = self.member.queued.take() {
            queue.hold_root(queued, Some(stop), admission)?;
        }
        Ok(RefusalEnd::Held)
    }
}

#[cfg(test)]
mod tests;

/// Request failures end either custody before a row-owned stop is recorded.
pub(super) fn member_stop(
    stop: WorthQueryOutputDemandDenial,
) -> Result<WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenial> {
    if matches!(
        stop.kind(),
        WorthQueryOutputDemandDenialKind::ExecutionRequest(_)
    ) {
        Err(stop)
    } else {
        Ok(stop)
    }
}
impl RefusalEnd {
    /// The caller's refusal returns before this door. A held queue row leaves
    /// its cursor open; a queue-budget stop invokes the owner's canonical end.
    pub(super) fn finish<T>(self, finish_cursor: impl FnOnce() -> T) -> Option<T> {
        match self {
            Self::Held => None,
            Self::QueueBudget => Some(finish_cursor()),
        }
    }
}
