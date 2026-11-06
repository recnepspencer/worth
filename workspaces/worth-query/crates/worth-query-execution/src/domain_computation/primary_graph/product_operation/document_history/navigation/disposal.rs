//! Release all pending read/publication pins before closing any managed head.
use super::*;
use crate::domain_computation::{
    primary_graph::{
        WorthQueryAdvancementPhase, WorthQueryApplicationProductBranchCleanupFailure,
        WorthQueryApplicationProductBranchCloseDenial, WorthQueryOwnedProductHead,
        WorthQueryOwnedProductHeadCloseFailure, WorthQueryPrimaryGraphApplicationRuntime,
    },
    WorthQueryProductUnpublishedRecovery, WorthQueryProductUnpublishedRecoveryReleaseFailure,
};
use worth_query_installation::facade::ApplicationSchema;

pub(super) struct NavigationCleanup<'runtime, S> {
    head: Option<WorthQueryOwnedProductHead<'runtime, S>>,
    _head_denial: Option<WorthQueryApplicationProductBranchCloseDenial>,
    retired: Option<WorthQueryApplicationProductBranchCleanupFailure>,
    fork: Option<custody::ForkCleanup>,
    recovery: Option<WorthQueryProductUnpublishedRecovery>,
    prior: Option<WorthQueryProductUnpublishedRecoveryReleaseFailure>,
}

fn retry_release<S: ApplicationSchema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    phase: &WorthQueryAdvancementPhase<'_>,
    pending: WorthQueryProductUnpublishedRecoveryReleaseFailure,
) -> Result<(), WorthQueryProductUnpublishedRecoveryReleaseFailure> {
    match pending {
        WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(failure) => application
            .close_product_publication_recovery_in_advancement(phase, failure.into_recovery())
            .map(|_| ()),
        WorthQueryProductUnpublishedRecoveryReleaseFailure::OwnerCleanup(failure) => failure
            .into_cleanup()
            .retry_in_advancement(phase)
            .map(|_| ())
            .map_err(WorthQueryProductUnpublishedRecoveryReleaseFailure::OwnerCleanup),
    }
}

impl<'runtime, S: ApplicationSchema> NavigationProgress<'runtime, S> {
    pub(super) fn retry_prior_cleanup(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<(), WorthQueryHistoryNavigationDenial> {
        if let Some(prior) = self.prior_cleanup.take()
            && let Err(prior) = retry_release(self.application, phase, prior)
        {
            self.prior_cleanup = Some(prior);
            return Err(WorthQueryHistoryNavigationDenial::Cleanup);
        }
        Ok(())
    }

    fn prepare_disposal(&mut self) {
        let state = std::mem::replace(&mut self.phase, NavigationPhase::Discarded);
        let mut cleanup = NavigationCleanup {
            head: None,
            _head_denial: None,
            retired: None,
            fork: None,
            recovery: None,
            prior: self.prior_cleanup.take(),
        };
        match state {
            NavigationPhase::Prepared
            | NavigationPhase::Selected(_)
            | NavigationPhase::Discarded => {}
            NavigationPhase::ForkRecovery(fork) => cleanup.fork = Some(fork),
            NavigationPhase::Candidate(candidate) => {
                cleanup.head = Some(candidate.into_owned_head())
            }
            NavigationPhase::Publication {
                candidate,
                publication,
            } => {
                drop(publication);
                cleanup.head = Some(candidate.into_owned_head());
            }
            NavigationPhase::Unpublished {
                candidate,
                recovery,
            } => {
                cleanup.head = Some(candidate.into_owned_head());
                cleanup.recovery = Some(recovery);
            }
            NavigationPhase::Selecting {
                candidate,
                exact,
                _publication,
                _outputs,
            } => {
                drop(_outputs);
                drop(exact);
                drop(_publication);
                cleanup.head = Some(candidate.into_owned_head());
            }
            NavigationPhase::AwaitingOutputs {
                candidate,
                exact,
                _publication,
            } => {
                drop(exact);
                drop(_publication);
                cleanup.head = Some(candidate.into_owned_head());
            }
            NavigationPhase::Discarding(pending) => cleanup = pending,
        }
        drop(self.source.take());
        drop(self.output_source.take());
        drop(self.output_source_memory.take());
        drop(self.output_basis_memory.take());
        self.phase = NavigationPhase::Discarding(cleanup);
    }

    pub(super) fn finish_disposal(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<usize, WorthQueryHistoryNavigationDenial> {
        let NavigationPhase::Discarding(cleanup) = &mut self.phase else {
            return Ok(0);
        };
        let closed = cleanup.retry(self.application, phase)?;
        self.phase = NavigationPhase::Discarded;
        drop(self.pending_slot.take());
        Ok(closed)
    }
}

impl<S: ApplicationSchema> NavigationCleanup<'_, S> {
    fn retry(
        &mut self,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<usize, WorthQueryHistoryNavigationDenial> {
        use WorthQueryHistoryNavigationDenial as Denial;
        let _execution = phase
            .checked_execution_request_for(&application.runtime.product_runtime)
            .map_err(Denial::Stopped)?;
        if let Some(fork) = self.fork.take()
            && let Err(fork) = fork.release(phase)
        {
            self.fork = Some(fork);
            return Err(Denial::ForkRecovery);
        }
        if let Some(prior) = self.prior.take()
            && let Err(prior) = retry_release(application, phase, prior)
        {
            self.prior = Some(prior);
            return Err(Denial::Cleanup);
        }
        if let Some(recovery) = self.recovery.take()
            && let Err(prior) =
                application.close_product_publication_recovery_in_advancement(phase, recovery)
        {
            self.prior = Some(prior);
            return Err(Denial::Cleanup);
        }
        if let Some(failure) = self.retired.take() {
            return match failure.into_cleanup().retry_in_advancement(phase) {
                Ok(_) => Ok(1),
                Err(failure) => {
                    self.retired = Some(failure);
                    Err(Denial::Cleanup)
                }
            };
        }
        if let Some(head) = self.head.take() {
            self._head_denial = None;
            return match head.close(phase) {
                Ok(_) => Ok(1),
                Err(WorthQueryOwnedProductHeadCloseFailure::Retained { head, denial }) => {
                    self.head = Some(head);
                    self._head_denial = Some(denial);
                    Err(Denial::Cleanup)
                }
                Err(WorthQueryOwnedProductHeadCloseFailure::Retired { failure, .. }) => {
                    self.retired = Some(failure);
                    Err(Denial::Cleanup)
                }
            };
        }
        Ok(0)
    }
}

pub(in crate::domain_computation::primary_graph::product_operation::document_history) struct NavigationDisposal<
    'runtime,
    S,
> {
    remaining: std::collections::btree_map::IntoValues<
        [u8; 32],
        registration::RegisteredAttempt<'runtime, S>,
    >,
    pending: Option<registration::RegisteredAttempt<'runtime, S>>,
    closed_unreported: usize,
}

impl<'runtime, S: ApplicationSchema> NavigationJournal<'runtime, S> {
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn into_disposal(
        self,
    ) -> NavigationDisposal<'runtime, S> {
        // A complete release pass precedes any retirement. Otherwise a later
        // pending attempt's H/C pin could block an earlier managed head forever.
        for entry in self.attempts.values() {
            entry.attempt.lock().prepare_disposal();
        }
        NavigationDisposal {
            remaining: self.attempts.into_values(),
            pending: None,
            closed_unreported: 0,
        }
    }
}

impl<S: ApplicationSchema> NavigationDisposal<'_, S> {
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn retry(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<usize, WorthQueryHistoryNavigationDenial> {
        loop {
            let Some(entry) = self.pending.take().or_else(|| self.remaining.next()) else {
                return Ok(std::mem::take(&mut self.closed_unreported));
            };
            let outcome = entry.attempt.lock().finish_disposal(phase);
            match outcome {
                Ok(count) => self.closed_unreported += count,
                Err(cause) => {
                    self.pending = Some(entry);
                    return Err(cause);
                }
            }
        }
    }
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn is_empty(
        &self,
    ) -> bool {
        self.pending.is_none() && self.remaining.len() == 0
    }
}

impl<S: ApplicationSchema> WorthQueryDocumentHistoryNavigation<'_, S> {
    /// Only disposal may use this correspondence without current READ/NAV.
    /// SDK cleanup runs outside the attempt guard and precedes Native retirement.
    pub fn discard_after_output_cleanup<E>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        cleanup: impl FnOnce([u8; 32]) -> Result<(), E>,
    ) -> Result<Result<(), E>, WorthQueryHistoryNavigationDenial> {
        use WorthQueryHistoryNavigationDenial as Denial;
        let key = {
            let progress = self.attempt.progress.try_lock().map_err(|_| Denial::Busy)?;
            phase
                .checked_execution_request_for(&progress.application.runtime.product_runtime)
                .map_err(Denial::Stopped)?;
            if matches!(progress.phase, NavigationPhase::Selected(_)) {
                return Err(Denial::AlreadySelected);
            }
            progress.meaning.canonical_key()
        };
        if let Err(cause) = cleanup(key) {
            return Ok(Err(cause));
        }
        self.discard(phase, access).map(Ok)
    }

    pub fn discard(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
    ) -> Result<(), WorthQueryHistoryNavigationDenial> {
        let mut progress = match self.attempt.progress.try_lock() {
            Ok(progress) => progress,
            Err(std::sync::TryLockError::WouldBlock) => {
                return Err(WorthQueryHistoryNavigationDenial::Busy)
            }
            Err(std::sync::TryLockError::Poisoned(poison)) => poison.into_inner(),
        };
        let _execution = phase
            .checked_execution_request_for(&progress.application.runtime.product_runtime)
            .map_err(WorthQueryHistoryNavigationDenial::Stopped)?;
        if matches!(progress.phase, NavigationPhase::Selected(_)) {
            return Err(WorthQueryHistoryNavigationDenial::AlreadySelected);
        }
        progress.prepare_disposal();
        progress.finish_disposal(phase)?;
        if let Some(slot) = progress.head_slot.take()
            && let Some(cell) = self.cell.upgrade()
            && let Some(mut selected) = cell.live()
        {
            selected.journal.cancel_head(slot);
        }
        Ok(())
    }
}
