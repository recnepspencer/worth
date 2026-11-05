//! Admission changes availability; maintenance discharges actual original owners.
use super::*;
use worth_query_declaration::facade::application_history::{
    ApplicationHistoryModel, ApplicationHistoryRecentSteps, ApplicationHistoryRetentionBinding,
};
use {
    crate::domain_computation::primary_graph::application_query::CompletedMetadataRead,
    crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation,
    crate::domain_computation::request_execution::WorthQueryAdvancementPhase,
};

#[derive(Clone, Copy, Debug)]
pub struct WorthQueryHistoryRetentionStatus {
    pub recent_steps: ApplicationHistoryRecentSteps,
    pub pending_window: Option<ApplicationHistoryRecentSteps>,
    pub unprotected_steps: usize,
    pub bookmarks: usize,
    pub current: WorthQueryHistoryNodeId,
    pub pending_releases: usize,
}
impl WorthQueryHistoryRetentionStatus {
    pub fn maintenance_pending(&self) -> bool {
        self.pending_window.is_some() || self.pending_releases != 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHistoryReleaseOutcome {
    NoPendingRelease,
    WaitingForNavigation,
    HeadRefused,
    CleanupRefused,
    Closed,
}

pub(in crate::domain_computation::primary_graph) enum HistoryRetentionChange {
    Window(ApplicationHistoryRecentSteps),
    Bookmark {
        node: WorthQueryHistoryNodeId,
        pin: bool,
    },
}

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> WorthQueryDocumentHistory<'_, S, H> {
    pub(in crate::domain_computation::primary_graph) fn apply_retention<
        N: ApplicationHistoryRetentionBinding<S, H>,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        read: &CompletedMetadataRead<'_, '_>,
        admission: &WorthQueryAdmittedApplicationOperation<
            S,
            N::Operation,
            N::Input,
            ApplicationHistoryModel<S, H>,
        >,
        change: HistoryRetentionChange,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryDocumentHistoryDenial> {
        use WorthQueryDocumentHistoryDenial as Denial;
        read.confirm(self, ())?;
        let product = admission
            .graph_work()
            .mutation_product()
            .ok_or(Denial::ForeignRecording)?
            .publication_binding();
        if product.observation() != read.current_product().observation() {
            return Err(Denial::HeadDiverged);
        }
        if !std::ptr::eq(read.phase(), phase) {
            return Err(Denial::ForeignPhase);
        }
        if !self.model.matches_operation(admission)
            || read.principal() != admission.operation_scope_binding().principal()
        {
            return Err(Denial::ForeignModel);
        }
        admission
            .validate_current_authority()
            .map_err(Denial::OperationAdmission)?;
        let request = admission.retain_admitted_request();
        self.pause_retention(retention_pause::RetentionChangePoint::Before);
        let execution = phase
            .checked_execution_request_for(&self.application().runtime.product_runtime)
            .map_err(Denial::Stopped)?;
        let mut selected = self.selection.lock();
        if !selected
            .expected
            .matches_observation(read.current_product().observation())
        {
            return Err(Denial::HeadDiverged);
        }
        selected.journal.quote_frontier(&execution)?;
        let mut changed = None;
        let confirmed = current_head::confirm_in_execution_with_effect(
            self.application(),
            &execution,
            read.current_product(),
            Some(&request),
            (),
            |_| {
                changed = Some(match change {
                    HistoryRetentionChange::Window(window) => selected
                        .journal
                        .begin_window(window)
                        .and_then(|_| selected.journal.advance_window()),
                    HistoryRetentionChange::Bookmark { node, pin } => {
                        selected.journal.bookmark(node, pin)
                    }
                });
            },
        );
        let status = selected.journal.retention_status();
        drop(selected);
        let (_, lapse) = confirmed.map_err(|(_, denial)| denial)?;
        if let Some(lapse) = lapse {
            return Err(Denial::OperationAdmission(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::new(lapse.denial_kind(), "history retention request")));
        }
        let removed = changed.expect("the original current request admitted the transition")?;
        drop(removed);
        self.pause_retention(retention_pause::RetentionChangePoint::After);
        phase
            .check_live_for(&self.application().runtime.product_runtime)
            .map_err(Denial::Stopped)?;
        Ok(status)
    }

    /// Cleanup needs no new payload permission. The sealed publication owner
    /// discharges selected SDK output tails before this actual branch can close.
    pub fn release_history_owner_after_output_cleanup<E>(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        before_close: impl FnOnce(crate::basis::WorthQueryProductBranch) -> Result<(), E>,
    ) -> Result<Result<WorthQueryHistoryReleaseOutcome, E>, WorthQueryDocumentHistoryDenial> {
        use head_custody::HistoryHeadCustody as State;
        let execution = phase
            .checked_execution_request_for(&self.application().runtime.product_runtime)
            .map_err(WorthQueryDocumentHistoryDenial::Stopped)?;
        let release = {
            let mut selected = self.selection.lock();
            selected.journal.quote_release(&execution)?;
            if selected.journal.retention_status().pending_releases == 0 {
                return Ok(Ok(WorthQueryHistoryReleaseOutcome::NoPendingRelease));
            }
            selected.journal.next_release()
        };
        let Some((key, slot)) = release else {
            return Ok(Ok(WorthQueryHistoryReleaseOutcome::WaitingForNavigation));
        };
        let branch = slot.lock().branch();
        if let Some(branch) = branch
            && let Err(cause) = before_close(branch)
        {
            return Ok(Err(cause));
        }
        let outcome = {
            let mut head = slot.lock();
            head.advance(phase);
            match &*head {
                State::Closed(_) => WorthQueryHistoryReleaseOutcome::Closed,
                State::Refused { .. } => WorthQueryHistoryReleaseOutcome::HeadRefused,
                State::Retired { .. } => WorthQueryHistoryReleaseOutcome::CleanupRefused,
                _ => unreachable!("a queued slot has filled actual custody"),
            }
        };
        if outcome == WorthQueryHistoryReleaseOutcome::Closed {
            self.selection.lock().journal.finish_release(key, &slot);
        }
        Ok(Ok(outcome))
    }
}
