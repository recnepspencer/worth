//! Eager custody visibility, independent of whether its prune is still queued.
use super::refreshed_rejoin::source_rows;
use super::*;
use worth_runtime_world::facade::CompositeCommitIdentity;
impl DemandRegistryState {
    pub(super) fn custody_prunable(&self, commit: &CompositeCommitIdentity) -> bool {
        let Some(custody) = self.source_custody.get(commit) else {
            return true;
        };
        if custody.token_count != 0 {
            return false;
        }
        if custody.retired.is_some() {
            return true;
        }
        let Some(bound) = &custody.bound_sources else {
            return false;
        };
        let fully_superseded = !bound.is_empty()
            && bound
                .iter()
                .all(|source| custody.source_denial(&source.identity).is_some());
        if !custody.completed && !fully_superseded {
            return false;
        }
        if !bound.iter().all(|source| {
            custody.consumed_sources.contains(&source.identity)
                || custody.source_denial(&source.identity).is_some()
        }) {
            return false;
        }
        for source in bound {
            for (key, record) in source_rows(&self.records, &source.identity) {
                if !record.source_commits.contains(commit) {
                    continue;
                }
                let awaited = claim_readthrough::observe_row(&self.records, key).awaited;
                let unfinished = match record.idle_disposition(awaited) {
                    IdleRowDisposition::InFlight | IdleRowDisposition::HeldForStaleOwner => true,
                    IdleRowDisposition::KeptAnswer => false,
                    IdleRowDisposition::Retired(reason) => !reason.releases_finished_work(),
                };
                if record.interests != 0
                    || !record.performed_obligations.is_empty()
                    || unfinished
                    || awaited
                {
                    return false;
                }
            }
        }
        true
    }
}
