//! One exact sealed-history slot per request-governed cleanup step.
use super::*;
use crate::lifecycle::RuntimeWorldResultSnapshotCleanupDenial as Denial;
use std::ops::Bound;
use worth_relational::facade::transactions::CommitResult;

pub(crate) struct ResultSnapshotCleanupRow {
    pub(crate) identity: CompositeCommitIdentity,
    pub(crate) result: Option<Arc<CommitResult>>,
    pub(crate) last: bool,
}

impl CompositeHistoryCatalog {
    pub(crate) fn try_result_snapshot_cleanup_frontier(
        &self,
    ) -> Result<RuntimeWorldPublicationFrontier, Denial> {
        let state = self.state.try_lock().map_err(|_| Denial::Busy)?;
        let revision = state.publication_revision.current();
        if revision == u64::MAX || state.entries.len() != state.inspection_order.len() {
            return Err(Denial::InconsistentSealedCatalog);
        }
        Ok(RuntimeWorldPublicationFrontier {
            owner: state.owner,
            catalog_affinity: self.allocation_affinity(),
            revision,
            upper_bound: state.inspection_order.last().cloned(),
        })
    }

    pub(crate) fn try_result_snapshot_cleanup_row(
        &self,
        frontier: &RuntimeWorldPublicationFrontier,
        after: Option<&CompositeCommitIdentity>,
    ) -> Result<ResultSnapshotCleanupRow, Denial> {
        let state = self.state.try_lock().map_err(|_| Denial::Busy)?;
        // Do not call validate_owner: it also takes the counters mutex.
        if frontier.owner != state.owner
            || frontier.owner.get() != state.owner.get()
            || frontier.catalog_affinity != self.allocation_affinity()
            || frontier.revision != state.publication_revision.current()
            || frontier.revision == u64::MAX
        {
            return Err(Denial::InconsistentSealedCatalog);
        }
        let upper = frontier
            .upper_bound
            .as_ref()
            .ok_or(Denial::InconsistentSealedCatalog)?;
        let identity = state
            .inspection_order
            .range((
                after.map_or(Bound::Unbounded, Bound::Excluded),
                Bound::Included(upper),
            ))
            .next()
            .ok_or(Denial::InconsistentSealedCatalog)?;
        let entry = state
            .entries
            .get(identity)
            .and_then(|slot| slot.get())
            .ok_or(Denial::InconsistentSealedCatalog)?;
        if entry.identity() != identity {
            return Err(Denial::InconsistentSealedCatalog);
        }
        let result = match &entry.publication {
            None if matches!(
                entry.commit().parent(),
                crate::history::CompositeCommitParent::Root
            ) && state.root.as_ref() == Some(identity) =>
            {
                None
            }
            Some(crate::history::PublicationEvidence::Readmitted(_)) => None,
            Some(crate::history::PublicationEvidence::Live(envelope)) => envelope
                .facts()
                .ok_or(Denial::InconsistentSealedCatalog)?
                .component_results
                .result_for_snapshot_cleanup()
                .map_err(|()| Denial::InconsistentSealedCatalog)?,
            None => return Err(Denial::InconsistentSealedCatalog),
        };
        Ok(ResultSnapshotCleanupRow {
            identity: identity.clone(),
            result,
            last: identity == upper,
        })
    }
}

impl RuntimeWorldPublicationFrontier {
    pub(crate) fn cleanup_is_empty(&self) -> bool {
        self.upper_bound.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn result_cleanup_cursor_refuses_catalog_busy_missing_and_unpopulated_reserved_slot() {
        let mut construction =
            crate::lifecycle::owner::RuntimeWorldOwnerConstructionContract::new().unwrap();
        let owner = construction.owner_identity();
        let limits = crate::budget::RuntimeWorldBudgets::without_policy_limits();
        let history = CompositeHistoryCatalog::new(
            owner,
            RuntimeWorldHistoryCatalogContract::installed(
                limits.retained_composite_commits(),
                limits.history_metadata_bytes(),
            ),
        );
        let identity = construction.issuer_mut().composite_commit().unwrap();
        {
            let mut state = history.state.lock().unwrap();
            // Admission indexes the original ID before populating its slot.
            state.inspection_order.insert(identity.clone());
            state
                .entries
                .insert(identity.clone(), Arc::new(OnceLock::new()));
            state.publication_revision.advance();
        }
        let frontier = history.try_result_snapshot_cleanup_frontier().unwrap();
        let held = history.state.lock().unwrap();
        assert!(matches!(
            history.try_result_snapshot_cleanup_frontier(),
            Err(Denial::Busy)
        ));
        assert!(matches!(
            history.try_result_snapshot_cleanup_row(&frontier, None),
            Err(Denial::Busy)
        ));
        drop(held);
        assert!(matches!(
            history.try_result_snapshot_cleanup_row(&frontier, None),
            Err(Denial::InconsistentSealedCatalog)
        ));
        assert_eq!(frontier.upper_bound.as_ref(), Some(&identity));
        history.state.lock().unwrap().entries.remove(&identity);
        assert!(matches!(
            history.try_result_snapshot_cleanup_row(&frontier, None),
            Err(Denial::InconsistentSealedCatalog)
        ));
        assert_eq!(
            history.state.lock().unwrap().inspection_order.first(),
            Some(&identity)
        );
    }
}
