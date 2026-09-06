use crate::publication::RuntimeWorldCancellationToken;
use crate::publication::{
    CompositeAttemptProgress, OwnerExecutionOutcome, PreparedCompositePublicationWithSignal,
    PreparedCompositePublicationWithoutSignal, ReservedCompositePublicationAttempt,
    SignalAttemptProgress,
};
use crate::recovery::ProductUnpublishedCause;

use worth_signal::facade::{SignalError, SignalTransaction};

use super::RuntimeWorldOwnerRoot;

#[path = "execution_service/admission.rs"]
mod admission;
#[path = "execution_service/relational.rs"]
mod relational;
#[path = "execution_service/signal.rs"]
mod signal;
#[path = "execution_service/successor.rs"]
mod successor;
#[path = "execution_service/terminal.rs"]
mod terminal;

#[cfg(test)]
#[path = "execution_service/rehearsal.rs"]
mod rehearsal;
#[cfg(test)]
#[path = "execution_service/tests.rs"]
mod tests;

use relational::RelationalExecutionFailure;
use signal::{SignalExecutionFailure, SignalExecutionRequest, UntouchedSignalMutation};

pub(crate) use signal::map_fork_no_effect;

impl<D, I, E, Ctx, T> super::super::ports::RuntimeWorldOwnerExecutionService
    for RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    type SignalDefinition = D;
    type SignalIdentity = I;
    type SignalEvent = E;
    type SignalContext = Ctx;
    type SignalTransactionKey = T;

    fn execute_without_signal(
        &self,
        prepared: PreparedCompositePublicationWithoutSignal,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> OwnerExecutionOutcome {
        self.execute_publication(
            prepared.into_attempt(),
            cancellation,
            SignalExecutionRequest::<Ctx, UntouchedSignalMutation<D, I, E, Ctx, T>>::RetainExact,
        )
    }

    fn execute_with_signal<F>(
        &self,
        prepared: PreparedCompositePublicationWithSignal,
        runtime_ctx: &mut Ctx,
        cancellation: &RuntimeWorldCancellationToken,
        apply: F,
    ) -> OwnerExecutionOutcome
    where
        F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>,
    {
        self.execute_publication(
            prepared.into_attempt(),
            cancellation,
            SignalExecutionRequest::AdvanceExact { runtime_ctx, apply },
        )
    }
}

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// The one publication execution body. Both public entry points reach it
    /// with the same reserved attempt; only the Signal borrow differs, so the
    /// two stages can never drift into two different orderings.
    fn execute_publication<F>(
        &self,
        mut attempt: ReservedCompositePublicationAttempt,
        cancellation: &RuntimeWorldCancellationToken,
        signal_request: SignalExecutionRequest<'_, Ctx, F>,
    ) -> OwnerExecutionOutcome
    where
        F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>,
    {
        attempt.begin_owner_execution();
        if let Err(cause) = self.admit_owner_execution(&attempt, cancellation) {
            return self.no_effect(attempt, cause);
        }

        let relational = match self.execute_relational(&mut attempt) {
            Ok(progress) => progress,
            Err(RelationalExecutionFailure {
                cause,
                no_effect,
                partial,
            }) => {
                let progress =
                    CompositeAttemptProgress::new(partial, SignalAttemptProgress::untouched());
                return self.retain_or_no_effect(attempt, progress, cause, no_effect);
            }
        };
        let relational_successor = relational.successor_basis().cloned();
        let mut progress =
            CompositeAttemptProgress::new(relational, SignalAttemptProgress::untouched());
        attempt.record_progress(&progress);

        if progress.relational_requires_settlement() {
            return self.settlement_pending(attempt, progress, relational_successor);
        }

        if let Err((cause, no_effect)) =
            self.pre_advance_signal_gate(attempt.expected_head(), attempt.deadline(), cancellation)
        {
            if cause == ProductUnpublishedCause::CancellationAfterEffect {
                attempt.observe_cancellation();
            }
            return self.retain_or_no_effect(attempt, progress, cause, no_effect);
        }

        match self.execute_signal(&mut attempt, &mut progress, signal_request, cancellation) {
            Ok(()) => {}
            Err(SignalExecutionFailure {
                cause,
                no_effect,
                partial,
            }) => {
                let progress = CompositeAttemptProgress::new(progress.into_relational(), partial);
                return self.retain_or_no_effect(attempt, progress, cause, no_effect);
            }
        };
        self.publish_settled_progress(attempt, progress, cancellation)
    }

    /// A Relational leg that still owes settlement never reaches the Signal
    /// owner. The attempt is retained with the successor basis the settlement
    /// will need, and the product head stays exactly where it was.
    fn settlement_pending(
        &self,
        attempt: ReservedCompositePublicationAttempt,
        progress: CompositeAttemptProgress,
        relational_successor: Option<
            worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        >,
    ) -> OwnerExecutionOutcome {
        let signal_expected = attempt.plan().signal().expected().clone();
        let successor = self.issue_successor_basis(
            relational_successor.expect("pending Relational progress carries its basis"),
            signal_expected,
            attempt.predecessor_basis().correspondence_basis().clone(),
        );
        OwnerExecutionOutcome::ProductUnpublished(attempt.settle(progress).retain_with_cause(
            successor,
            ProductUnpublishedCause::SettlementPending,
            None,
        ))
    }
}
