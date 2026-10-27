#[path = "history_preparation_binding/merge_inspection.rs"]
mod merge_inspection;

use std::sync::atomic::AtomicU64;
#[cfg(test)]
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::branch::{
    PreparedRelationalBranchRootCapture, RelationalBranchReferenceCell,
    RelationalBranchReferenceRegistry, RelationalBranchRoot, RelationalBranchRootCaptureDenial,
    RelationalBranchRootIdentityIssuer,
};
use crate::history::data::{BranchId, CanonicalCommitEnvelope, CommitId};
use crate::identity::data::VersionId;

use super::{reserve_identity, RelationalCanonicalPublicationRoutes};

/// Live preparation-only binding to branch truth and identity reservation.
#[derive(Debug, Clone)]
pub(crate) struct RelationalPreparationHistory {
    branches: RelationalBranchReferenceRegistry,
    commit_ids: Arc<AtomicU64>,
    version_ids: Arc<AtomicU64>,
    commit_floor: u64,
    version_floor: u64,
    canonical_routes: Arc<RelationalCanonicalPublicationRoutes>,
    root_ids: RelationalBranchRootIdentityIssuer,
    #[cfg(test)]
    root_capture_sabotage: Arc<std::sync::atomic::AtomicBool>,
}

impl RelationalPreparationHistory {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        branches: RelationalBranchReferenceRegistry,
        commit_ids: Arc<AtomicU64>,
        version_ids: Arc<AtomicU64>,
        commit_floor: u64,
        version_floor: u64,
        canonical_routes: Arc<RelationalCanonicalPublicationRoutes>,
        root_ids: RelationalBranchRootIdentityIssuer,
        #[cfg(test)] root_capture_sabotage: Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        Self {
            branches,
            commit_ids,
            version_ids,
            commit_floor,
            version_floor,
            canonical_routes,
            root_ids,
            #[cfg(test)]
            root_capture_sabotage,
        }
    }

    pub(crate) fn branch_cell(&self, branch: &BranchId) -> Option<RelationalBranchReferenceCell> {
        self.branches.get(branch)
    }

    pub(crate) fn reserve_commit_id(&self) -> Option<CommitId> {
        reserve_identity(&self.commit_ids, self.commit_floor).map(CommitId)
    }

    pub(crate) fn reserve_version_id(&self) -> Option<VersionId> {
        reserve_identity(&self.version_ids, self.version_floor).map(VersionId)
    }

    pub(crate) fn current_version_id(&self) -> VersionId {
        use std::sync::atomic::Ordering;
        VersionId(self.version_ids.load(Ordering::Relaxed).saturating_sub(1))
    }

    pub(crate) fn canonical_envelope_for_version(
        &self,
        version: VersionId,
    ) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.canonical_routes.by_version(version)
    }

    pub(crate) fn canonical_envelope(
        &self,
        commit: CommitId,
    ) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.canonical_routes.by_commit(commit)
    }

    pub(crate) fn canonical_catalog_len(&self) -> usize {
        self.canonical_routes.catalog_len_bound()
    }

    pub(crate) fn publication_requires_settlement(&self, commit_id: CommitId) -> bool {
        self.canonical_routes.has_unsettled_performed_publication() == Some(commit_id)
    }

    pub(crate) fn validate_new_publication_envelope(
        &self,
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<(), String> {
        let existing = self.canonical_routes.by_commit(envelope.commit.commit_id);
        crate::history::RelationalCommitCatalog::validate_new_envelope_against(
            existing.as_deref(),
            envelope,
        )
        .map_err(|denial| format!("publication catalog admission denied: {denial:?}"))
    }

    pub(crate) fn validate_recovery_publication_envelope(
        &self,
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<(), String> {
        let existing = self.canonical_routes.by_commit(envelope.commit.commit_id);
        crate::history::RelationalCommitCatalog::validate_envelope_against(
            existing.as_deref(),
            envelope,
        )
        .map_err(|denial| format!("publication catalog admission denied: {denial:?}"))
    }

    pub(crate) fn reserve_canonical_publication_route(
        &self,
        envelope: Arc<crate::history::RelationalCommitEnvelopeResidency>,
        root: Arc<RelationalBranchRoot>,
    ) -> Result<super::PreparedCanonicalPublicationRoute, &'static str> {
        RelationalCanonicalPublicationRoutes::reserve(&self.canonical_routes, envelope, root)
    }

    pub(crate) fn record_transaction_validation_attempt(&self, branch: &BranchId) {
        self.record_cost(branch, |costs| {
            costs.transaction_validation_attempts =
                costs.transaction_validation_attempts.saturating_add(1);
        });
    }

    pub(crate) fn record_retained_history_head_lookup(&self, branch: &BranchId) {
        self.record_cost(branch, |costs| {
            costs.retained_history_head_lookups =
                costs.retained_history_head_lookups.saturating_add(1);
        });
    }

    pub(crate) fn record_candidate_preparation(&self, branch: &BranchId) {
        self.record_cost(branch, |costs| {
            costs.candidate_preparations = costs.candidate_preparations.saturating_add(1);
        });
    }

    pub(crate) fn record_candidate_discard(&self, branch: &BranchId) {
        self.record_cost(branch, |costs| {
            costs.candidate_discards = costs.candidate_discards.saturating_add(1);
        });
    }

    pub(crate) fn record_publication_attempt(&self, branch: &BranchId) {
        self.record_cost(branch, |costs| {
            costs.publication_attempts = costs.publication_attempts.saturating_add(1);
        });
    }

    pub(crate) fn prepare_branch_root_capture<P: crate::storage::overlay::PartitionAccess>(
        &self,
        partitions: &P,
        published_delta: &crate::storage::RelationalPublishedPartitionDelta,
        previous: Option<&Arc<RelationalBranchRoot>>,
        envelope: Arc<CanonicalCommitEnvelope>,
        registry: &crate::schema::data::RelationalSchemaRegistry,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> Result<PreparedRelationalBranchRootCapture, RelationalBranchRootCaptureDenial> {
        #[cfg(test)]
        if self.root_capture_sabotage.swap(false, Ordering::Relaxed) {
            return Err(RelationalBranchRootCaptureDenial::UnresolvedContentSymbol(
                crate::symbols::data::Symbol(u32::MAX),
            ));
        }
        RelationalBranchRoot::prepare_capture(
            &self.root_ids,
            partitions,
            published_delta,
            previous,
            envelope,
            registry,
            symbols,
        )
    }

    pub(crate) fn validate_branch_root_capture(
        &self,
        touched_regions: usize,
    ) -> Result<(), RelationalBranchRootCaptureDenial> {
        self.root_ids.validate_capture_capacity(touched_regions)
    }

    fn record_cost(
        &self,
        branch: &BranchId,
        update: impl FnOnce(&mut super::RelationalBranchSharingCostCounters),
    ) {
        if let Some(cell) = self.branch_cell(branch) {
            cell.publication_cell().record_sharing_cost(update);
        }
    }
}
