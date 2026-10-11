//! Entrywise cleanup: a paused head remains queued; completed heads stay done.
use super::super::refreshed_rejoin::{
    occurrence_rows, price_mutation_with_source_allowance, OccurrenceSearchCharge, WalkKind,
};
use super::*;
use worth_runtime_world::facade::CompositeCommitIdentity;

/// Foreign maintenance is optional once an admission has paid its own edit.
pub(in super::super) const FOREIGN_CUSTODY_DRAIN_QUOTA: usize = 2;

impl DemandRegistryState {
    pub(in super::super) fn drain_completed_custody(
        &mut self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let visits = self.custody_cleanup.len;
        for _ in 0..visits {
            let Some(commit) = self.custody_cleanup.head.clone() else {
                break;
            };
            self.drain_custody_entry(&commit, admission)?;
        }
        Ok(())
    }

    pub(in super::super) fn drain_custody_entry(
        &mut self,
        commit: &CompositeCommitIdentity,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        if !self
            .source_custody
            .get(commit)
            .is_some_and(|entry| entry.cleanup.queued)
        {
            return Ok(());
        }
        let kind = WalkKind::TerminalCleanup;
        let allowance = super::custody_price::entry_allowance(self, commit, admission, kind)?;
        price_mutation_with_source_allowance(self, admission, &[], kind, allowance)?
            .prune_custody_entry(commit);
        Ok(())
    }

    pub(in super::super) fn drain_custody_for_occurrence(
        &mut self,
        key: &WorthQueryOutputDemandKey,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let mut visited = None;
        while self.pending_custody_prune() {
            let mut found = None;
            for (_, row) in occurrence_rows(&self.records, key, admission)? {
                admission.inspect_row()?;
                for commit in &row.source_commits {
                    admission.inspect_row()?;
                    if visited.as_ref().is_none_or(|prior| commit > prior)
                        && found.as_ref().is_none_or(|next| commit < next)
                        && self
                            .source_custody
                            .get(commit)
                            .is_some_and(|entry| entry.cleanup.queued)
                    {
                        found = Some(commit.clone());
                    }
                }
            }
            let Some(commit) = found else { break };
            self.drain_custody_entry(&commit, admission)?;
            visited = Some(commit);
        }
        Ok(())
    }

    /// A maintenance budget pause has a retry holder: the linked head. The
    /// own edit is already paid, so this pause cannot refuse that admission.
    pub(in super::super) fn drain_foreign_custody_quota(
        &mut self,
        admission: &mut InvalidationEditAdmission,
    ) {
        let visits = self.custody_cleanup.len.min(FOREIGN_CUSTODY_DRAIN_QUOTA);
        for _ in 0..visits {
            let Some(commit) = self.custody_cleanup.head.clone() else {
                break;
            };
            match self.drain_custody_entry(&commit, admission) {
                Ok(()) => {}
                Err(denial) => {
                    debug_assert_eq!(
                        denial.kind(),
                        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
                    );
                    // No entry was unlinked: the next admission resumes here.
                    break;
                }
            }
        }
    }
}
