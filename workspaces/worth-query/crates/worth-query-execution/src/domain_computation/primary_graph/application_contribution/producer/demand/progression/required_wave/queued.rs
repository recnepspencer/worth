//! Dirty required records from the shared queue become frames of the wave
//! after the caller's own outcome, so one advance progresses the dirty set
//! and no queued record can starve the caller of its own request.

use worth_relational::facade::mvcc::CompanionPublicationCompletion;

use super::super::WorthQueryOutputDemandAdvance;
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::{
    SelectedRequiredWork, SelectedRequiredWorkKind,
};

/// Popped work for one advance. The frame item backs the queue chain being
/// certified. Held items requeue when this advance ends: only a cause
/// covered by a Current proof on this wave is acknowledged.
pub(super) struct RequiredQueueFrames {
    drained: bool,
    frame: Option<SelectedRequiredWork>,
    held: Vec<SelectedRequiredWork>,
    /// The caller's own outcome, decided before any queue frame runs.
    caller: Option<WorthQueryOutputDemandAdvance>,
    /// Whether the caller settled at the wave's current position. A queue
    /// frame's commit moves the wave past the caller's proof.
    caller_current: bool,
}

impl RequiredQueueFrames {
    pub(super) const fn new() -> Self {
        Self {
            drained: false,
            frame: None,
            held: Vec::new(),
            caller: None,
            caller_current: false,
        }
    }

    /// Whether the caller's outcome is decided and queue frames now run.
    pub(super) const fn draining(&self) -> bool {
        self.caller.is_some()
    }

    /// Whether the chain being certified started from a queue item.
    pub(super) const fn active(&self) -> bool {
        self.frame.is_some()
    }

    /// The caller's own chain ended; queue frames take the rest of the
    /// request.
    pub(super) fn finish_caller(&mut self, outcome: WorthQueryOutputDemandAdvance) {
        self.caller_current = matches!(&outcome, WorthQueryOutputDemandAdvance::Settled(_));
        self.caller = Some(outcome);
    }

    /// A committed successor moved the wave past the caller's proof.
    pub(super) fn wave_moved(&mut self) {
        self.caller_current = false;
    }

    /// The caller's outcome once queue work ends, in any way.
    pub(super) fn take_caller(&mut self) -> Option<WorthQueryOutputDemandAdvance> {
        self.caller.take()
    }

    /// Once the caller's outcome is decided, nothing a queue frame meets can
    /// replace it: held items requeue on drop and their errors surface on
    /// their own demands. Earlier errors are the caller's own.
    pub(super) fn conclude(
        &mut self,
        result: Result<Option<WorthQueryOutputDemandAdvance>, WorthQueryOutputDemandDenial>,
    ) -> Result<Option<WorthQueryOutputDemandAdvance>, WorthQueryOutputDemandDenial> {
        match result {
            Err(_) if self.draining() => Ok(self.take_caller()),
            result => result,
        }
    }

    /// Pop until one item becomes a frame or this advance has visited the
    /// whole queue. Each item is popped at most once unless a new cause or
    /// acknowledgement requeues it.
    pub(super) fn next_frame<Schema>(
        &mut self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        wave: &RequiredWaveSelection<'_, Schema>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<SelectedReadyReadmission>, WorthQueryOutputDemandDenial> {
        while !self.drained {
            let Some(work) = runtime.output_demands.next_required_work_for_selected(
                wave.shared.selected().product().read_lease_ref(),
                &wave.positioned,
                &runtime
                    .primary_provider
                    .graph
                    .source_owner
                    .invalidation_owner,
                admission,
            )?
            else {
                self.drained = true;
                break;
            };
            if let SelectedRequiredWorkKind::Native { observer, .. } = work.kind() {
                match observer.state() {
                    // The publication never changed the image: its cause is void.
                    CompanionPublicationCompletion::Aborted => {
                        drop(work.acknowledge_admitted(admission)?);
                        continue;
                    }
                    CompanionPublicationCompletion::Prepared => {
                        self.hold(work, admission)?;
                        continue;
                    }
                    // Installed moved the image first. Stamp the settled hints
                    // before the head check below, so only publications at or
                    // before this wave's position are covered.
                    CompanionPublicationCompletion::Installed => {
                        work.stamp_settled_hints(admission)?;
                    }
                }
            }
            // Every cause was raised before this pop. While the head is still
            // this wave's position, the wave's Current proof covers it.
            if !head_is_selected(runtime, wave, admission)? {
                self.hold(work, admission)?;
                continue;
            }
            let Some(ready) = runtime
                .output_demands
                .selected_ready_completion(&work, admission)?
            else {
                self.hold(work, admission)?;
                continue;
            };
            if ready.same_record(&wave.caller_ready, admission)? {
                // The caller's proof covers its own cause only while the wave
                // is still where the caller settled. An unfunded
                // acknowledgement leaves the cause queued.
                if self.caller_current {
                    drop(work.acknowledge_subsumed_admitted(admission));
                } else {
                    self.hold(work, admission)?;
                }
                continue;
            }
            self.frame = Some(work);
            return Ok(Some(ready));
        }
        Ok(None)
    }

    /// The queue frame itself is Current on this wave.
    pub(super) fn discharge_frame(
        &mut self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let work = self
            .frame
            .take()
            .expect("an active queue chain has its item");
        drop(work.acknowledge_subsumed_admitted(admission)?);
        Ok(())
    }

    /// The queue chain cannot finish on this wave. Budget exhaustion ends the
    /// queue work and returns the caller's outcome; any other stop keeps the
    /// item for a later advance and moves on to the next one.
    pub(super) fn hold_frame(
        &mut self,
        stop: Option<WorthQueryOutputDemandDenial>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        if let Some(stop) = stop.filter(is_budget) {
            return Err(stop);
        }
        let work = self
            .frame
            .take()
            .expect("an active queue chain has its item");
        self.hold(work, admission)
    }

    fn hold(
        &mut self,
        work: SelectedRequiredWork,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        admission
            .charge_external_work(2)
            .map_err(|_| work_denial())?;
        self.held.try_reserve(1).map_err(|_| capacity_denial())?;
        self.held.push(work);
        Ok(())
    }
}

fn is_budget(denial: &WorthQueryOutputDemandDenial) -> bool {
    matches!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
            | WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
    )
}

fn head_is_selected<Schema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    wave: &RequiredWaveSelection<'_, Schema>,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, WorthQueryOutputDemandDenial> {
    match runtime
        .primary_provider
        .graph
        .source_owner
        .invalidation_owner
        .selected_installed_discontinuity(&wave.positioned, admission)
    {
        Ok(_) => Ok(true),
        Err(CompanionPreflightStop::SelectedSourceMismatch) => Ok(false),
        Err(stop) => Err(admission_denial(stop)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unrelated_stop() -> WorthQueryOutputDemandDenial {
        WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::ForeignSource,
            "a queued record's own stop",
        )
    }

    #[test]
    fn a_queue_stop_after_the_caller_finished_returns_the_caller_outcome() {
        let mut queue = RequiredQueueFrames::new();
        queue.finish_caller(WorthQueryOutputDemandAdvance::Pending);
        assert!(matches!(
            queue.conclude(Err(unrelated_stop())),
            Ok(Some(WorthQueryOutputDemandAdvance::Pending))
        ));
    }

    #[test]
    fn a_stop_before_the_caller_finished_is_the_callers_own() {
        let mut queue = RequiredQueueFrames::new();
        assert!(matches!(
            queue.conclude(Err(unrelated_stop())),
            Err(stop) if stop.kind() == WorthQueryOutputDemandDenialKind::ForeignSource
        ));
    }
}
