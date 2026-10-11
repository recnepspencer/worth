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
impl<Schema: ApplicationSchema + 'static> RefusingMember<'_, Schema> {
    pub(super) fn end(
        self,
        stop: WorthQueryOutputDemandDenial,
        recording: StopRecording,
        queue: &mut QueuedRequiredRoots,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<RefusalEnd, WorthQueryOutputDemandDenial> {
        let stop = member_stop(stop)?;
        let row = match self.row {
            RefusalRow::Member => self.member.key(),
            RefusalRow::HeldHead(head) => head,
        };
        recording.record_if_needed(self.registry, row, &stop);
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
    /// The same budget ending releases and holds the canonical cursor for both routes.
    pub(super) fn finish<Schema, Family>(
        self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        wave: &RequiredWaveSelection<'_, Schema>,
        frontier: &mut RequiredFrontier,
        queue: &mut QueuedRequiredRoots,
        admission: &mut InvalidationEditAdmission,
    ) -> Option<Result<Option<WorthQueryOutputDemandAdvance>, WorthQueryOutputDemandDenial>>
    where
        Schema: ApplicationSchema + 'static,
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        match self {
            Self::Held => None,
            Self::QueueBudget => Some(canonical_apply::finish(
                runtime, demand, wave, frontier, queue, admission,
            )),
        }
    }
}
