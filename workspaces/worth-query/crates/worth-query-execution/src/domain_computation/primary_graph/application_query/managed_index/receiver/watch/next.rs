//! Fresh full Parent admission precedes availability, cause lookup and ACK.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::derived_view::PredicateNoticeDisposition,
    live_delivery::WorthQueryLivePredicatePoll,
};

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn next_application_managed_index_watch<B, I>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        mut plan: BoundPlan<'_, S, B>,
        watch: &mut WorthQueryManagedIndexWatch<S, I>,
    ) -> Result<WorthQueryManagedIndexWatchOutcome<B::Query, ResultValue<S, B>>, Denial>
    where
        B: ApplicationQueryBinding<S>,
        I: ApplicationManagedIndexBinding<S>,
        ResultValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
    {
        let execution = phase
            .execution_request_for(&self.product_runtime)
            .map_err(|_| Denial::new(ReceiverCause::Refused(Refusal::ForeignPhase), None))?;
        let allocation = SourceFactsAllocation::open(&execution)
            .map_err(|cause| Denial::new(ReceiverCause::Execution(cause), None))?;
        let outcome = (|| {
            let context = context::prepare(&execution, self, binding, &mut plan, &allocation)?;
            if !plan.query.lanes().live_enabled()
                || context
                    .association
                    .access_contract()
                    .meaning()
                    .predicate_profile()
                    .is_err()
            {
                return Err(ReceiverCause::Refused(Refusal::UnsupportedMode));
            }
            let Some(state) = watch.state.as_mut() else {
                return Ok(WorthQueryManagedIndexWatchOutcome::Closed);
            };
            let source = &self.primary_provider.live_delivery;
            if !source.admits_predicate_occurrence(
                state.queue.subscription(),
                context.selected.product().observation(),
            ) || Audience::of(&plan.graph_work)? != state.audience
                || !state
                    .selection
                    .association
                    .same_association(&context.association)
                || state.selection.scope != state.audience.scope
                || plan.controls.maximum_result_count() != state.maximum_results
                || !state.probe.matches_parameters(
                    &execution,
                    &context.association,
                    &plan.parameters,
                )?
            {
                return Err(ReceiverCause::Refused(Refusal::ForeignContext));
            }
            let same_use = match (&state.governance, plan.governance.authorization()) {
                (None, None) => true,
                (Some(original), Some(current)) => original.matches(current, &execution)?,
                _ => false,
            };
            if !same_use {
                return Err(ReceiverCause::Refused(Refusal::ForeignContext));
            }
            if !state
                .registration
                .active(&execution)
                .map_err(open::work_denial)?
            {
                watch
                    .terminate(Disposition::Cancelled)
                    .map_err(|_| ReceiverCause::Refused(Refusal::TerminalReleaseMismatch))?;
                return Ok(WorthQueryManagedIndexWatchOutcome::Closed);
            }
            if let Err(cause) = source.validate_predicate_progress(
                &execution,
                state.queue.subscription(),
                context.selected.product().observation(),
            ) {
                match cause {
                    crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveAdmissionDenial::Execution(cause) => return Err(ReceiverCause::Execution(cause)),
                    crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveAdmissionDenial::Unavailable(_) => {
                        watch.terminate(Disposition::Cancelled).map_err(|_| ReceiverCause::Refused(Refusal::TerminalReleaseMismatch))?;
                        return Ok(WorthQueryManagedIndexWatchOutcome::Unavailable);
                    }
                }
            }
            // A physical suspension/untracked movement must not become Pending.
            let graph = self
                .runtime
                .primary_graph()
                .ok_or(ReceiverCause::Refused(Refusal::ForeignGraph))?;
            let program = graph
                .layout
                .managed_source(I::NAME)
                .ok_or(ReceiverCause::Refused(Refusal::UnsupportedMode))?;
            let available = graph.with_runtime(&execution, |runtime| {
                runtime.read_truth().observe_managed_index_source(
                    &execution,
                    context.selected.application_basis().snapshot_handle(),
                    program,
                    context.selector.subject(),
                )
            });
            match available {
                Ok(observed) => drop(observed),
                Err(worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial::RootUnavailable) => {
                    watch.terminate(Disposition::Cancelled).map_err(|_| ReceiverCause::Refused(Refusal::TerminalReleaseMismatch))?;
                    return Ok(WorthQueryManagedIndexWatchOutcome::Unavailable);
                }
                Err(cause) => return Err(ReceiverCause::Provider(ProviderExecutionDenial::NativeSource(cause))),
            }
            let basis = state
                .basis
                .as_mut()
                .ok_or(ReceiverCause::Refused(Refusal::TerminalReleaseMismatch))?;
            match state
                .queue
                .fill(&execution, source, basis, state.registration.slot())?
            {
                WorthQueryLivePredicatePoll::Pending => {
                    return Ok(WorthQueryManagedIndexWatchOutcome::Pending)
                }
                WorthQueryLivePredicatePoll::Overflow(missed_commit_batches) => {
                    watch
                        .terminate(Disposition::Cancelled)
                        .map_err(|_| ReceiverCause::Refused(Refusal::TerminalReleaseMismatch))?;
                    return Ok(WorthQueryManagedIndexWatchOutcome::Overflow {
                        missed_commit_batches,
                    });
                }
                WorthQueryLivePredicatePoll::Closed => {
                    watch
                        .terminate(Disposition::Completed)
                        .map_err(|_| ReceiverCause::Refused(Refusal::TerminalReleaseMismatch))?;
                    return Ok(WorthQueryManagedIndexWatchOutcome::Closed);
                }
                WorthQueryLivePredicatePoll::Unavailable => {
                    return Err(ReceiverCause::Refused(Refusal::TerminalReleaseMismatch))
                }
                WorthQueryLivePredicatePoll::Ready => {}
            }
            let (notification, disposition) = state
                .queue
                .front()
                .expect("a ready queue owns its genuine notification");
            if disposition == PredicateNoticeDisposition::Unavailable {
                watch
                    .terminate(Disposition::Cancelled)
                    .map_err(|_| ReceiverCause::Refused(Refusal::TerminalReleaseMismatch))?;
                return Ok(WorthQueryManagedIndexWatchOutcome::Unavailable);
            }
            if context.selected.product().selected_commit() < notification.composite_commit() {
                return Err(ReceiverCause::Refused(Refusal::StaleImage));
            }
            let notification = notification.clone();
            let mut read = self.read_managed_parent::<B>(
                &execution,
                plan,
                context,
                external,
                &allocation,
                ManagedParentCapture::Watch,
            )?;
            let selection = read
                .selection
                .take()
                .ok_or(ReceiverCause::Refused(Refusal::ForeignSelection))?;
            let completed = self.finish_managed_parent::<B>(
                &execution,
                read,
                ManagedParentCompletionCapture::None,
            )?;
            // The complete result is owned before this irreversible ACK. The
            // public entry preserves Delivered across its final liveness poll.
            execution
                .consult()
                .map_err(|cause| ReceiverCause::Execution(cause.into()))?;
            state
                .queue
                .acknowledge(state.basis.as_mut().expect("open queue basis"))
                .map_err(|_| ReceiverCause::Refused(Refusal::TerminalReleaseMismatch))?;
            state.selection = selection;
            Ok(WorthQueryManagedIndexWatchOutcome::Delivered(
                WorthQueryManagedIndexWatchUpdate {
                    notification,
                    read: completed.read,
                },
            ))
        })();
        outcome.map_err(|cause| Denial::new(cause, Some(allocation)))
    }
}
