//! Entrywise cleanup: a paused head remains queued; completed heads stay done.
use super::super::refreshed_rejoin::occurrence_rows;
use super::*;
use worth_runtime_world::facade::CompositeCommitIdentity;

impl DemandRegistryState {
    #[cfg(test)]
    pub(in super::super) fn drain_completed_custody(&mut self) {
        let visits = self.custody_cleanup.len;
        for _ in 0..visits {
            let Some(commit) = self.custody_cleanup.head.clone() else {
                break;
            };
            self.drain_custody_entry(&commit);
        }
    }

    pub(in super::super) fn drain_custody_entry(&mut self, commit: &CompositeCommitIdentity) {
        if !self
            .source_custody
            .get(commit)
            .is_some_and(|entry| entry.cleanup.queued)
        {
            return;
        }
        self.prune_custody_entry(commit);
    }

    pub(in super::super) fn drain_custody_for_occurrence(
        &mut self,
        key: &WorthQueryOutputDemandKey,
    ) {
        let mut visited = None;
        while self.pending_custody_prune() {
            let mut found = None;
            for (_, row) in occurrence_rows(&self.records, key) {
                for commit in &row.source_commits {
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
            self.drain_custody_entry(&commit);
            visited = Some(commit);
        }
    }

    /// Reclaim the next queued source at its existing cleanup owner.
    pub(in super::super) fn drain_next_foreign_custody(&mut self) {
        if let Some(commit) = self.custody_cleanup.head.clone() {
            self.prune_custody_entry(&commit);
        }
    }
}
