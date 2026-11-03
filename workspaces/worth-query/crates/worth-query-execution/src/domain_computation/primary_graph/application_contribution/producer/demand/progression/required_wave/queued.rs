//! Selected queue tokens hold required roots for one advancement.
//! Only a Current proof acknowledges a token; every unfinished token requeues.

mod acknowledgement;

use worth_relational::facade::mvcc::CompanionPublicationCompletion;

use super::super::WorthQueryOutputDemandAdvance;
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::RequiredWorkClaim;
use crate::domain_computation::primary_graph::application_output_demand::SelectedRequiredWork;
use crate::domain_computation::primary_graph::application_output_demand::SelectedRequiredWorkKind;

/// Popped work for one advance. Each token backs its selected required root.
/// Held items requeue when this advance ends: only a cause
/// covered by a Current proof on this wave is acknowledged.
pub(super) struct QueuedRequiredRoots {
    drained: bool,
    roots: Vec<Option<SelectedRequiredWork>>,
    held: Vec<SelectedRequiredWork>,
    /// A certified caller outcome cannot be replaced by a queued root's failure.
    caller: Option<CallerOutcome>,
}

enum CallerOutcome {
    Current(performed::CurrentProof<Arc<WorthQueryOutputDemandSettlement>>),
    Decided(Arc<WorthQueryOutputDemandSettlement>),
    Pending,
}

impl QueuedRequiredRoots {
    pub(super) const fn new() -> Self {
        Self {
            drained: false,
            roots: Vec::new(),
            held: Vec::new(),
            caller: None,
        }
    }

    /// Whether the caller's outcome is decided, Settled or Pending.
    pub(super) const fn draining(&self) -> bool {
        self.caller.is_some()
    }

    /// The caller is decided; queue failures retain their own scheduling custody.
    pub(super) fn finish_caller(
        &mut self,
        proof: performed::CurrentProof<Arc<WorthQueryOutputDemandSettlement>>,
    ) {
        self.caller = Some(CallerOutcome::Current(proof));
    }

    pub(super) fn defer_caller(&mut self) {
        self.caller = Some(CallerOutcome::Pending);
    }

    pub(super) fn wave_moved(&mut self) {
        self.caller = self.caller.take().map(|outcome| match outcome {
            CallerOutcome::Current(proof) => CallerOutcome::Decided(proof.into_value()),
            CallerOutcome::Decided(settlement) => CallerOutcome::Decided(settlement),
            CallerOutcome::Pending => CallerOutcome::Pending,
        });
        // The selected queue is visited once per advance, even as its basis moves.
    }

    /// The caller's outcome once queue work ends, in any way.
    pub(super) fn take_caller(&mut self) -> Option<WorthQueryOutputDemandAdvance> {
        self.caller.take().map(|outcome| match outcome {
            CallerOutcome::Current(proof) => {
                WorthQueryOutputDemandAdvance::Settled(proof.into_value())
            }
            CallerOutcome::Decided(settlement) => {
                WorthQueryOutputDemandAdvance::Settled(settlement)
            }
            CallerOutcome::Pending => WorthQueryOutputDemandAdvance::Pending,
        })
    }

    /// Ordinary queued diagnostics cannot replace a decided caller. Actual
    /// native cleanup custody must instead reach its caller's owning handoff;
    /// holding a queue token or cloning diagnostics cannot preserve that terminal.
    pub(super) fn conclude(
        &mut self,
        result: Result<Option<WorthQueryOutputDemandAdvance>, WorthQueryOutputDemandDenial>,
    ) -> Result<Option<WorthQueryOutputDemandAdvance>, WorthQueryOutputDemandDenial> {
        match result {
            Err(mut stop) => {
                if let Some(cleanup) = stop.take_managed_cleanup() {
                    return Err(stop.with_managed_cleanup(cleanup));
                }
                if self.draining()
                    && !matches!(
                        stop.kind(),
                        WorthQueryOutputDemandDenialKind::ExecutionRequest(_)
                    )
                {
                    Ok(self.take_caller())
                } else {
                    Err(stop)
                }
            }
            result => result,
        }
    }

    /// Claim every queued row once, without selecting its version or Ready.
    pub(super) fn next_claim<Schema>(
        &mut self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        wave: &RequiredWaveSelection<'_, Schema>,
    ) -> Result<Option<(usize, RequiredWorkClaim)>, WorthQueryOutputDemandDenial> {
        if self.drained {
            runtime.output_demands.maintain_selected_required_cleanup(
                wave.shared.selected().product().read_lease_ref(),
                &wave.positioned,
                &runtime
                    .primary_provider
                    .graph
                    .source_owner
                    .invalidation_owner,
            )?;
            return Ok(None);
        }
        let Some(claim) = runtime.output_demands.next_required_claim_for_selected(
            wave.shared.selected().product().read_lease_ref(),
            &wave.positioned,
            &runtime
                .primary_provider
                .graph
                .source_owner
                .invalidation_owner,
        )?
        else {
            self.drained = true;
            return Ok(None);
        };

        if self.roots.len() == self.roots.capacity() {
            let next = self
                .roots
                .capacity()
                .checked_mul(2)
                .ok_or_else(capacity_denial)?
                .max(1);

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
        self.roots.push(None);
        Ok(Some((index, claim)))
    }
    /// The owner's first inspection binds the claim to the current cause and pin.
    pub(super) fn inspect_claim<Schema>(
        &mut self,
        index: usize,
        claim: &mut RequiredWorkClaim,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        wave: &RequiredWaveSelection<'_, Schema>,
    ) -> Result<Option<SelectedReadyReadmission>, WorthQueryOutputDemandDenial> {
        let Some(work) = self.prepare_selection(claim, runtime, wave)? else {
            return Ok(None);
        };
        let Some(ready) = runtime.output_demands.selected_ready_completion(&work)? else {
            self.hold(work)?;
            return Ok(None);
        };
        // The caller already certified its own occurrence. A moved basis
        // cannot use that old certification to discharge a newer cause.
        if ready.same_record(&wave.anchor_ready)? {
            match self.caller.take() {
                Some(CallerOutcome::Current(proof)) => {
                    self.caller = Some(CallerOutcome::Decided(Arc::clone(proof.value())));
                    drop(acknowledgement::acknowledge_caller_claim(work, proof)?);
                }
                other => {
                    self.caller = other;
                    self.hold(work)?;
                }
            }
            return Ok(None);
        }
        self.roots[index] = Some(work);
        Ok(Some(ready))
    }

    /// A published successor has its own issued pin; its original claim still
    /// selects the current cause at this inspection, never the pre-publication version.
    pub(super) fn select_successor_cause<Schema>(
        &mut self,
        index: usize,
        claim: &mut RequiredWorkClaim,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        wave: &RequiredWaveSelection<'_, Schema>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let work = self.prepare_selection(claim, runtime, wave)?;
        self.roots[index] = work;
        Ok(())
    }
    fn prepare_selection<Schema>(
        &mut self,
        claim: &mut RequiredWorkClaim,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        wave: &RequiredWaveSelection<'_, Schema>,
    ) -> Result<Option<SelectedRequiredWork>, WorthQueryOutputDemandDenial> {
        let Some(work) = runtime
            .output_demands
            .select_required_claim(claim, &wave.positioned)?
        else {
            return Ok(None);
        };
        if let SelectedRequiredWorkKind::Native { observer, .. } = work.kind() {
            match observer.state() {
                // The publication never changed the image: its cause is void.
                CompanionPublicationCompletion::Aborted => {
                    drop(work.discard_aborted_native()?);
                    return Ok(None);
                }
                CompanionPublicationCompletion::Prepared => {
                    self.hold(work)?;
                    return Ok(None);
                }
                // Installed moved the image first. Stamp the settled hints
                // before the head check below, so only publications at or
                // before this wave's position are covered.
                CompanionPublicationCompletion::Installed => {
                    work.stamp_settled_hints()?;
                }
            }
        }
        // Every cause was raised before this inspection. While the head is still
        // this wave's position, the wave's Current proof covers it.
        if !head_is_selected(runtime, wave)? {
            self.hold(work)?;
            return Ok(None);
        }
        Ok(Some(work))
    }

    pub(super) fn discard_selection(&mut self, index: usize) {
        drop(self.roots[index].take());
    }

    /// Hold this failed queued member while independent members proceed.
    /// Query admission-budget exhaustion ends queue work; request stops return
    /// through the driver's request-stop door before this holding door.
    pub(super) fn hold_root(
        &mut self,
        index: usize,
        stop: Option<WorthQueryOutputDemandDenial>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let stop = match stop {
            Some(mut stop) => {
                if let Some(cleanup) = stop.take_managed_cleanup() {
                    return Err(stop.with_managed_cleanup(cleanup));
                }
                Some(stop)
            }
            None => None,
        };
        if let Some(stop) = stop.filter(is_budget) {
            return Err(stop);
        }
        let Some(work) = self.roots[index].take() else {
            return Ok(());
        };
        self.hold(work)
    }

    fn hold(&mut self, work: SelectedRequiredWork) -> Result<(), WorthQueryOutputDemandDenial> {
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
) -> Result<bool, WorthQueryOutputDemandDenial> {
    match runtime
        .primary_provider
        .graph
        .source_owner
        .invalidation_owner
        .selected_installed_discontinuity(&wave.positioned)
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
        queue.defer_caller();
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
