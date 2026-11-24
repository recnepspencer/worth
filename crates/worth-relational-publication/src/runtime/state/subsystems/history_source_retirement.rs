//! Read-only retirement binding to the same original history owners.
use super::{
    canonical_publication_routes::RelationalCanonicalPublicationRoutes,
    history_canonical_routes::RelationalPublicationHistory,
};
use contracts::change_source::read_custody::RelationalReadCustodyHoldersOwnerPort as _;
use std::sync::Arc;
use worth_relational_branch::internal_port::branch::registry::source_retirement::RelationalSourceRootSlot;
use worth_relational_branch::internal_port::branch::registry::RelationalBranchReferenceRegistry;
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;
use worth_relational_branch::internal_port::history::retention::owner::RelationalBranchRetentionOwner;
use worth_relational_contracts::internal_port as contracts;

pub struct RelationalSourceHistoryBinding {
    branches: RelationalBranchReferenceRegistry,
    retention: RelationalBranchRetentionOwner,
    catalog: Arc<RelationalCanonicalPublicationRoutes>,
    ledger: worth_relational_storage::internal_port::storage::substrate::owned_state::RuntimeOwnedState<worth_relational_branch::internal_port::runtime::state::subsystems::history::history_ledger::HistoryLedger>,
}
pub fn bind_source_retirement(
    history: &RelationalPublicationHistory,
) -> RelationalSourceHistoryBinding {
    bind_detached_source_retirement(history, history.canonical_publication_routes.clone())
}

pub fn bind_detached_source_retirement(
    history: &worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem,
    catalog: Arc<RelationalCanonicalPublicationRoutes>,
) -> RelationalSourceHistoryBinding {
    RelationalSourceHistoryBinding {
        branches: history.branch_cells.clone(),
        retention: history.retention_owner.clone(),
        catalog,
        ledger: history.ledger.share(),
    }
}
#[derive(Debug)]
pub struct SourceHistoryContended;

impl RelationalSourceHistoryBinding {
    pub fn performed_walk(
        &self,
    ) -> super::canonical_publication_routes::RelationalPerformedSourceWalk {
        self.catalog.source_retirement_walk()
    }
    pub fn checkpoint_envelope(
        &self,
        after: Option<contracts::history::data::coordinates::CommitId>,
    ) -> Result<
        Option<(
            contracts::history::data::coordinates::CommitId,
            Arc<worth_relational_branch::internal_port::history::commit::envelope_residency::RelationalCommitEnvelopeResidency>,
        )>,
        SourceHistoryContended,
    >{
        self.catalog
            .source_retirement_checkpoint_envelope(after)
            .map_err(|_| SourceHistoryContended)
    }
    pub fn ledger_envelope(
        &self,
        after: Option<contracts::history::data::coordinates::CommitId>,
    ) -> Result<
        Option<(
            contracts::history::data::coordinates::CommitId,
            Arc<worth_relational_branch::internal_port::history::commit::envelope_residency::RelationalCommitEnvelopeResidency>,
        )>,
        SourceHistoryContended,
    >{
        Ok(self
            .ledger
            .try_read()
            .ok_or(SourceHistoryContended)?
            .next_retained_envelope(after))
    }
    pub fn live_root(
        &self,
        index: usize,
    ) -> Result<RelationalSourceRootSlot, SourceHistoryContended> {
        self.branches
            .source_retirement_root(index)
            .map_err(|_| SourceHistoryContended)
    }
    pub fn retired_root(
        &self,
        after: Option<usize>,
    ) -> Result<Option<(usize, Arc<RelationalBranchRoot>)>, SourceHistoryContended> {
        self.retention
            .next_source_retirement_root(after)
            .map_err(|_| SourceHistoryContended)
    }
    pub fn retire_head(&self, index: usize) -> Result<Option<bool>, SourceHistoryContended> {
        self.branches
            .try_retire_stopped_head(index)
            .map_err(|_| SourceHistoryContended)
    }
    pub fn retained_close_holders(
        &self,
    ) -> contracts::change_source::read_custody::RelationalReadCustodyHolders {
        contracts::change_source::read_custody::RelationalReadCustodyHolders::detached(
            self.retention.counters(),
        )
    }
    pub fn external_custody_retained(&self) -> bool {
        self.retention.has_detached_obligations()
    }
}
