//! Read-only retirement binding to the same original history owners.
use super::{
    canonical_publication_routes::RelationalCanonicalPublicationRoutes,
    history_canonical_routes::RelationalPublicationHistory,
};
use crate::branch::registry::RelationalBranchReferenceRegistry;
use crate::branch::registry::RelationalSourceRootSlot;
use crate::branch::root::RelationalBranchRoot;
use crate::history::retention::RelationalBranchRetentionOwner;
use std::sync::Arc;

pub(crate) struct RelationalSourceHistoryBinding {
    branches: RelationalBranchReferenceRegistry,
    retention: RelationalBranchRetentionOwner,
    catalog: Arc<RelationalCanonicalPublicationRoutes>,
    ledger: super::RuntimeOwnedState<super::HistoryLedger>,
}
pub(crate) fn bind_source_retirement(
    history: &RelationalPublicationHistory,
) -> RelationalSourceHistoryBinding {
    bind_detached_source_retirement(history, history.canonical_publication_routes.clone())
}

pub(crate) fn bind_detached_source_retirement(
    history: &super::HistorySubsystem,
    catalog: Arc<RelationalCanonicalPublicationRoutes>,
) -> RelationalSourceHistoryBinding {
    RelationalSourceHistoryBinding {
        branches: history.branch_cells.clone(),
        retention: history.retention_owner.clone(),
        catalog,
        ledger: history.ledger.share(),
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
            Arc<crate::history::commit::RelationalCommitEnvelopeResidency>,
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
            Arc<crate::history::commit::RelationalCommitEnvelopeResidency>,
        )>,
        (),
    > {
        Ok(self
            .ledger
            .try_read()
            .ok_or(())?
            .next_retained_envelope(after))
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
    pub(crate) fn retained_close_holders(
        &self,
    ) -> crate::change_source::read_custody::RelationalReadCustodyHolders {
        crate::change_source::read_custody::RelationalReadCustodyHolders::detached(
            self.retention.counters(),
        )
    }
    pub(crate) fn external_custody_retained(&self) -> bool {
        self.retention.has_detached_obligations()
    }
}
