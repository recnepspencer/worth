//! Read-only retirement binding to the same original history owners.
use super::{HistorySubsystem, RelationalCanonicalPublicationRoutes};
use crate::branch::{
    RelationalBranchReferenceRegistry, RelationalBranchRoot, RelationalSourceRootSlot,
};
use crate::history::retention::RelationalBranchRetentionOwner;
use std::sync::Arc;

pub(crate) struct RelationalSourceHistoryBinding {
    branches: RelationalBranchReferenceRegistry,
    retention: RelationalBranchRetentionOwner,
    pub(super) catalog: Arc<RelationalCanonicalPublicationRoutes>,
    ledger: super::RuntimeOwnedState<super::HistoryLedger>,
}
impl HistorySubsystem {
    pub(crate) fn source_retirement_binding(&self) -> RelationalSourceHistoryBinding {
        RelationalSourceHistoryBinding {
            branches: self.branch_cells.clone(),
            retention: self.retention_owner.clone(),
            catalog: self.canonical_publication_routes.clone(),
            ledger: self.ledger.share(),
        }
    }
}
impl RelationalSourceHistoryBinding {
    pub(crate) fn performed_walk(
        &self,
    ) -> super::canonical_publication_routes::RelationalPerformedSourceWalk {
        self.catalog.source_retirement_walk()
    }
    pub(crate) fn checkpoint_envelope(
        &self,
        after: Option<crate::history::data::CommitId>,
    ) -> Result<
        Option<(
            crate::history::data::CommitId,
            Arc<crate::history::RelationalCommitEnvelopeResidency>,
        )>,
        (),
    > {
        self.catalog.source_retirement_checkpoint_envelope(after)
    }
    pub(crate) fn ledger_envelope(
        &self,
        after: Option<crate::history::data::CommitId>,
    ) -> Result<
        Option<(
            crate::history::data::CommitId,
            Arc<crate::history::RelationalCommitEnvelopeResidency>,
        )>,
        (),
    > {
        use std::ops::Bound::{Excluded, Unbounded};
        let ledger = self.ledger.try_read().ok_or(())?;
        let next = match after {
            None => ledger.commit_envelopes.first_key_value(),
            Some(key) => ledger
                .commit_envelopes
                .range((Excluded(key), Unbounded))
                .next(),
        };
        Ok(next.map(|(&id, cell)| (id, cell.clone())))
    }
    pub(crate) fn live_root(&self, index: usize) -> Result<RelationalSourceRootSlot, ()> {
        self.branches.source_retirement_root(index)
    }
    pub(crate) fn retired_root(
        &self,
        after: Option<usize>,
    ) -> Result<Option<(usize, Arc<RelationalBranchRoot>)>, ()> {
        self.retention.next_source_retirement_root(after)
    }
    pub(crate) fn retire_head(&self, index: usize) -> Result<Option<bool>, ()> {
        self.branches.try_retire_stopped_head(index)
    }
    pub(crate) fn external_custody_retained(&self) -> bool {
        self.retention.has_detached_obligations()
    }
}
