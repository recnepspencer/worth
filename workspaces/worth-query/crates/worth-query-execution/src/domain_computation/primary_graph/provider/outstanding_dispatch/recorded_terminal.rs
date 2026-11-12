//! A source-backed readmitted terminal releases original membership without live delivery.
use super::super::{WorthQueryCanonicalInboundCompletion, WorthQueryPrimaryGraphProvider};
use super::*;

impl OutstandingDispatchOwner {
    fn release_recorded_terminal(
        &self,
        original: &WorthQueryCommittedDispatchOutboxObservation,
        terminal: &WorthQueryCanonicalInboundCompletion,
        basis: &mut super::super::session_commit::WorthQueryReceiptBasisRetentionStore,
    ) -> Result<(), WorthQueryTerminalDispatchReleaseDenial> {
        use WorthQueryTerminalDispatchReleaseDenial as Denial;
        let publication = original.committed_product_publication();
        if !terminal.matches_original_dispatch(
            original.record(),
            original.commit_reference(),
            publication.composite_commit(),
            publication.product_incarnation(),
        ) {
            return Err(Denial::OriginalMismatch);
        }
        let mut entries = self.entries.lock().unwrap_or_else(|p| p.into_inner());
        let Some(entry) = entries
            .by_correlation
            .get_mut(original.record().correlation())
        else {
            return Ok(());
        };
        if entry
            .original
            .as_ref()
            .is_none_or(|source| !source.matches(publication))
        {
            return Err(Denial::OriginalMismatch);
        }
        let commit = entry.commit.ok_or(Denial::PendingPublication)?;
        let live_basis = entry
            .original
            .as_ref()
            .is_some_and(OutstandingDispatchOriginal::has_live_basis);
        if live_basis && !basis.has_mandatory(commit) {
            return Err(Denial::ExactCommitUnavailable);
        }
        entry.terminal_world = Some(terminal.completion_world_commit().clone());
        if entry.in_flight != 0 {
            return Ok(());
        }
        let removed = entries
            .by_correlation
            .remove(original.record().correlation())
            .expect("validated original");
        decrement(&mut entries.counts, &removed.contract);
        decrement(&mut entries.by_branch, &removed.incarnation);
        if live_basis {
            basis.release_mandatory(commit);
        }
        Ok(())
    }
}

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn release_recorded_terminal_dispatch_provenance(
        &self,
        original: &WorthQueryCommittedDispatchOutboxObservation,
        terminal: &WorthQueryCanonicalInboundCompletion,
    ) -> Result<(), WorthQueryTerminalDispatchReleaseDenial> {
        let canonical = self
            .lookup_completed_inbound(original.record().correlation())
            .map_err(|_| WorthQueryTerminalDispatchReleaseDenial::TerminalIndexUnavailable)?
            .ok_or(WorthQueryTerminalDispatchReleaseDenial::TerminalIndexUnavailable)?;
        if canonical.completion_world_commit() != terminal.completion_world_commit() {
            return Err(WorthQueryTerminalDispatchReleaseDenial::CompletionWorldMismatch);
        }
        self.outstanding_dispatch.release_recorded_terminal(
            original,
            terminal,
            &mut self
                .receipt_basis_retention
                .lock()
                .unwrap_or_else(|p| p.into_inner()),
        )
    }
}
