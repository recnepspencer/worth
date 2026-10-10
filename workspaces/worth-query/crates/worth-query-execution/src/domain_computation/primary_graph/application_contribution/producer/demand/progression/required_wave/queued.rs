//! Selected queue tokens hold required roots for one advancement.
//! Only a Current proof acknowledges a token; every unfinished token requeues.

use worth_relational::facade::mvcc::CompanionPublicationCompletion;

use super::super::WorthQueryOutputDemandAdvance;
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::{
    SelectedRequiredWork, SelectedRequiredWorkKind,
};

/// Popped work for one advance. Each token backs its selected required root.
/// Held items requeue when this advance ends: only a cause
/// covered by a Current proof on this wave is acknowledged.
pub(super) struct QueuedRequiredRoots {
    drained: bool,
    roots: Vec<Option<SelectedRequiredWork>>,
    held: Vec<SelectedRequiredWork>,
    /// A certified caller outcome cannot be replaced by a queued root's failure.
    caller: Option<WorthQueryOutputDemandAdvance>,
    caller_current: bool,
}

impl QueuedRequiredRoots {
    pub(super) const fn new() -> Self {
        Self {
            drained: false,
            roots: Vec::new(),
            held: Vec::new(),
            caller: None,
            caller_current: false,
        }
    }

    /// Whether the caller's own Current proof has decided its outcome.
    pub(super) const fn draining(&self) -> bool {
        self.caller.is_some()
    }

    /// The caller is certified; queue failures retain their own scheduling custody.
    pub(super) fn finish_caller(&mut self, outcome: WorthQueryOutputDemandAdvance) {
        self.caller_current = matches!(&outcome, WorthQueryOutputDemandAdvance::Settled(_));
        self.caller = Some(outcome);
    }

    pub(super) fn wave_moved(&mut self) {
        self.caller_current = false;
        // The selected queue is visited once per advance, even as its basis moves.
    }

    /// The caller's outcome once queue work ends, in any way.
    pub(super) fn take_caller(&mut self) -> Option<WorthQueryOutputDemandAdvance> {
        self.caller.take()
    }

    /// Once the caller's outcome is decided, nothing a queued root meets can
    /// replace it: held items requeue on drop and their errors surface on
    /// their own demands. Earlier errors are the caller's own.
    pub(super) fn conclude(
        &mut self,
        result: Result<Option<WorthQueryOutputDemandAdvance>, WorthQueryOutputDemandDenial>,
    ) -> Result<Option<WorthQueryOutputDemandAdvance>, WorthQueryOutputDemandDenial> {
        match result {
            Err(stop)
                if self.draining()
                    && !matches!(
                        stop.kind(),
                        WorthQueryOutputDemandDenialKind::ExecutionRequest(_)
                    ) =>
            {
                Ok(self.take_caller())
            }
            result => result,
        }
    }

    /// Pop until one item becomes a root or this advance has visited the
    /// whole queue. A moved basis never restarts that collection in this advance.
    pub(super) fn next_root<Schema>(
        &mut self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        wave: &RequiredWaveSelection<'_, Schema>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<(usize, SelectedReadyReadmission)>, WorthQueryOutputDemandDenial> {
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
            // The caller already certified its own occurrence. A moved basis
            // cannot use that old certification to discharge a newer cause.
            if ready.same_record(&wave.anchor_ready, admission)? {
                if self.caller_current {
                    drop(work.acknowledge_subsumed_admitted(admission));
                } else {
                    self.hold(work, admission)?;
                }
                continue;
            }
            admission
                .charge_external_work(std::mem::size_of::<Option<SelectedRequiredWork>>() as u64)
                .map_err(|_| work_denial())?;
            if self.roots.len() == self.roots.capacity() {
                let next = self
                    .roots
                    .capacity()
                    .checked_mul(2)
                    .ok_or_else(capacity_denial)?
                    .max(1);
                let item = std::mem::size_of::<Option<SelectedRequiredWork>>();
                admission
                    .admit_read_scratch(
                        u64::try_from(next.checked_mul(item).ok_or_else(capacity_denial)?)
                            .map_err(|_| capacity_denial())?,
                    )
                    .map_err(admission_denial)?;
                admission
                    .charge_external_work(
                        u64::try_from(self.roots.len().checked_mul(item).ok_or_else(work_denial)?)
                            .map_err(|_| work_denial())?,
                    )
                    .map_err(|_| work_denial())?;
                let mut replacement = Vec::new();
                replacement
                    .try_reserve_exact(next)
                    .map_err(|_| capacity_denial())?;
                if replacement.capacity() != next {
                    return Err(capacity_denial());
                }
                replacement.append(&mut self.roots);
                self.roots = replacement;
            }
            let index = self.roots.len();
            self.roots.push(Some(work));
            return Ok(Some((index, ready)));
        }
        Ok(None)
    }

    /// Revalidate a collected selection before its member has published.
    pub(super) fn retains_selection<Schema>(
        &mut self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        index: usize,
        ready: &SelectedReadyReadmission,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let work = self.roots[index]
            .as_ref()
            .expect("a queued root owns its selection");
        let current = runtime
            .output_demands
            .selected_ready_completion(work, admission)?;
        if let Some(current) = current {
            if current.same_ready_cell(ready, admission)? {
                return Ok(true);
            }
        }
        // A publication can supersede a selection collected for a later wave.
        // Dropping its issued token requeues the cause; only a fresh owner join
        // can provide the next Ready pin and selected version.
        drop(self.roots[index].take());
        Ok(false)
    }

    /// The queued root itself is Current on this basis.
    pub(super) fn acknowledge_root(
        &mut self,
        index: usize,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let work = self.roots[index]
            .take()
            .expect("a selected queued root has its token");
        drop(work.acknowledge_subsumed_admitted(admission)?);
        Ok(())
    }

    /// Hold this failed queued member while independent members proceed.
    /// Query admission-budget exhaustion ends queue work; request stops return
    /// through the driver's request-stop door before this holding door.
    pub(super) fn hold_root(
        &mut self,
        index: usize,
        stop: Option<WorthQueryOutputDemandDenial>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        if let Some(stop) = stop.filter(is_budget) {
            return Err(stop);
        }
        let work = self.roots[index]
            .take()
            .expect("a selected queued root has its token");
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
        let mut queue = QueuedRequiredRoots::new();
        queue.finish_caller(WorthQueryOutputDemandAdvance::Pending);
        assert!(matches!(
            queue.conclude(Err(unrelated_stop())),
            Ok(Some(WorthQueryOutputDemandAdvance::Pending))
        ));
    }

    #[test]
    fn a_stop_before_the_caller_finished_is_the_callers_own() {
        let mut queue = QueuedRequiredRoots::new();
        assert!(matches!(
            queue.conclude(Err(unrelated_stop())),
            Err(stop) if stop.kind() == WorthQueryOutputDemandDenialKind::ForeignSource
        ));
    }
}
