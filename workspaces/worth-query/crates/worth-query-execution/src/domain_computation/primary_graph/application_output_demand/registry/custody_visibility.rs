//! Eager custody visibility, independent of whether its prune is still queued.
use super::refreshed_rejoin::{source_rows, OccurrenceSearchCharge};
use super::*;
use worth_runtime_world::facade::CompositeCommitIdentity;
impl DemandRegistryState {
    pub(super) fn custody_prunable<C: OccurrenceSearchCharge>(
        &self,
        commit: &CompositeCommitIdentity,
        charge: &mut C,
    ) -> Result<bool, C::Refusal> {
        charge.inspect_row()?;
        let Some(custody) = self.source_custody.get(commit) else {
            return Ok(true);
        };
        if custody.token_count != 0 {
            return Ok(false);
        }
        if custody.retired.is_some() {
            return Ok(true);
        }
        let Some(bound) = &custody.bound_sources else {
            return Ok(false);
        };
        let fully_superseded = !bound.is_empty()
            && bound
                .iter()
                .all(|source| custody.source_denial(&source.identity).is_some());
        if !custody.completed && !fully_superseded {
            return Ok(false);
        }
        if !bound.iter().all(|source| {
            custody.consumed_sources.contains(&source.identity)
                || custody.source_denial(&source.identity).is_some()
        }) {
            return Ok(false);
        }
        for source in bound {
            for (key, record) in source_rows(&self.records, &source.identity, charge)? {
                charge.inspect_row()?;
                if !record.source_commits.contains(commit) {
                    continue;
                }
                if record.interests != 0
                    || !record.performed_obligations.is_empty()
                    || !matches!(
                        &record.state,
                        DemandState::Output(WorthQueryOutputProgress {
                            checkpoint: Some(WorthQueryOutputCheckpoint::Ready(_)),
                            ..
                        }) | DemandState::Failed(_)
                    )
                    || claim_readthrough::observe_row(&self.records, key, charge)?.awaited
                {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
}
