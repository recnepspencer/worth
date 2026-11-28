use contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use worth_relational_contracts::internal_port as contracts;
#[path = "history_preparation_binding/admitted_version.rs"]
pub mod admitted_version;
#[path = "history_preparation_binding/candidate_root.rs"]
pub mod candidate_root;

use std::sync::atomic::AtomicU64;
#[cfg(test)]
use std::sync::atomic::Ordering;
use std::sync::Arc;

use contracts::branch::root_capture_denial::RelationalBranchRootCaptureDenial;
use contracts::history::data::coordinates::BranchId;
use contracts::history::data::coordinates::CommitId;
use contracts::identity::data::primitives::VersionId;
use worth_relational_branch::internal_port::branch::reference::RelationalBranchReferenceCell;
use worth_relational_branch::internal_port::branch::registry::RelationalBranchReferenceRegistry;
use worth_relational_branch::internal_port::branch::root::identity::RelationalBranchRootIdentityIssuer;
use worth_relational_branch::internal_port::branch::root::{
    PreparedRelationalBranchRootCapture, RelationalBranchRoot,
};
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;

use super::canonical_publication_routes::RelationalCanonicalPublicationRoutes;
use worth_relational_branch::internal_port::runtime::state::subsystems::history::history_identity_allocation::reserve_identity;

/// Live preparation-only binding to branch truth and identity reservation.
#[derive(Debug, Clone)]
pub struct RelationalPreparationHistory {
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
    pub fn index_definition_frontier(&self) -> u64 {
        self.canonical_routes.index_definition_frontier()
    }

    pub fn detached_for_preflight(&self) -> Self {
        use std::sync::atomic::Ordering;
        Self {
            branches: self.branches.detached_owner_snapshot(),
            commit_ids: Arc::new(AtomicU64::new(self.commit_ids.load(Ordering::Relaxed))),
            version_ids: Arc::new(AtomicU64::new(self.version_ids.load(Ordering::Relaxed))),
            commit_floor: self.commit_floor,
            version_floor: self.version_floor,
            canonical_routes: Arc::clone(&self.canonical_routes),
            root_ids: self.root_ids.detached_owner_snapshot(),
            #[cfg(test)]
            root_capture_sabotage: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
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

    pub fn branch_cell(&self, branch: &BranchId) -> Option<RelationalBranchReferenceCell> {
        self.branches.get(branch)
    }

    pub fn reserve_commit_id(&self) -> Option<CommitId> {
        reserve_identity(&self.commit_ids, self.commit_floor).map(CommitId)
    }

    pub fn reserve_version_id(&self) -> Option<VersionId> {
        reserve_identity(&self.version_ids, self.version_floor).map(VersionId)
    }

    pub fn current_version_id(&self) -> VersionId {
        use std::sync::atomic::Ordering;
        VersionId(self.version_ids.load(Ordering::Relaxed).saturating_sub(1))
    }

    pub fn demand_canonical_envelope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: CommitId,
    ) -> Result<
        Option<Arc<CanonicalCommitEnvelope>>,
        contracts::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        contracts::execution::request_custody::check_request(execution)?;
        self.canonical_routes
            .demand_envelope(execution, id)
            .map_err(Into::into)
    }

    pub(crate) fn commit_for_version_in_context(
        &self,
        version: VersionId,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<Option<CommitId>, contracts::execution::denial_cause::RelationalExecutionDenialCause>
    {
        self.canonical_routes
            .commit_for_version_in_context(version, context)
    }

    pub(crate) fn canonical_catalog_len(&self) -> usize {
        self.canonical_routes.catalog_len_bound()
    }

    pub(crate) fn canonical_stream_position(
        &self,
        commit: CommitId,
    ) -> Option<contracts::publication::patch::data::patch_position::PatchStreamPosition> {
        self.canonical_routes.stream_position(commit)
    }

    pub(crate) fn publication_requires_settlement(&self, commit_id: CommitId) -> bool {
        self.canonical_routes.has_unsettled_performed_publication() == Some(commit_id)
    }

    pub(crate) fn validate_new_publication_envelope(
        &self,
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<(), String> {
        let existing = self.canonical_routes.by_commit(envelope.commit.commit_id);
        worth_relational_branch::internal_port::history::commit::catalog::RelationalCommitCatalog::validate_new_envelope_against(
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
        worth_relational_branch::internal_port::history::commit::catalog::RelationalCommitCatalog::validate_envelope_against(
            existing.as_deref(),
            envelope,
        )
        .map_err(|denial| format!("publication catalog admission denied: {denial:?}"))
    }

    pub(crate) fn reserve_canonical_publication_route(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        envelope: Arc<worth_relational_branch::internal_port::history::commit::envelope_residency::RelationalCommitEnvelopeResidency>,
        root: Arc<RelationalBranchRoot>,
    ) -> Result<
        super::canonical_publication_routes::PreparedCanonicalPublicationRoute,
        contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
    > {
        let resident = envelope.get().ok_or_else(|| {
            contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::publication(
                contracts::publication::data::publication_error::PublicationError::new(
                    contracts::publication::bundle::publication_status::PublicationStage::BundleAssembly,
                    "prepared publication envelope is not resident",
                ),
            )
        })?;
        let generation = self
            .canonical_routes
            .prepare_authored_generation(execution, &resident)
            .map_err(super::canonical_publication_routes::authored_write_commit_denial)?;
        let original_payload = worth_relational_durability::internal_port::durability::canonical_commit_artifact::prepared_payload::PreparedOriginalCanonicalPayload::capture(resident, &execution)
            .map_err(|cause| match cause {
                contracts::durability::data::recovery_stop::RelationalRecoveryStop::Content(cause) => worth_relational_branch::internal_port::content::publication::commit_denial(cause),
                contracts::durability::data::recovery_stop::RelationalRecoveryStop::Request(cause) => contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::execution(cause.into()),
                contracts::durability::data::recovery_stop::RelationalRecoveryStop::Allocation(cause) => contracts::execution::denial::allocation_commit_error(contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::conflict(contracts::transactions::data::outcomes::commit_conflict::CommitConflict::new(contracts::transactions::data::outcomes::conflict_class::ConflictClass::ExecutionAllocationDenied { denial: cause }))),
                contracts::durability::data::recovery_stop::RelationalRecoveryStop::Durability(cause) => contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::publication(contracts::publication::data::publication_error::PublicationError::new(contracts::publication::bundle::publication_status::PublicationStage::BundleAssembly, cause.detail)),
                contracts::durability::data::recovery_stop::RelationalRecoveryStop::AdmissionImage(cause) => contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::publication(contracts::publication::data::publication_error::PublicationError::new(contracts::publication::bundle::publication_status::PublicationStage::BundleAssembly, format!("original payload capture: {cause:?}"))),
            })?;
        RelationalCanonicalPublicationRoutes::reserve(
            &self.canonical_routes,
            envelope,
            root,
            generation,
            original_payload,
        )
        .map_err(|detail| {
            contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::publication(
                contracts::publication::data::publication_error::PublicationError::new(
                    contracts::publication::bundle::publication_status::PublicationStage::BundleAssembly,
                    detail,
                ),
            )
        })
    }

    pub fn record_transaction_validation_attempt(&self, branch: &BranchId) {
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

    pub fn record_candidate_discard(&self, branch: &BranchId) {
        self.record_cost(branch, |costs| {
            costs.candidate_discards = costs.candidate_discards.saturating_add(1);
        });
    }

    pub fn record_publication_attempt(&self, branch: &BranchId) {
        self.record_cost(branch, |costs| {
            costs.publication_attempts = costs.publication_attempts.saturating_add(1);
        });
    }

    pub(crate) fn prepare_branch_root_capture<
        P: worth_relational_storage::internal_port::storage::overlay::access::PartitionAccess,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: &P,
        published_delta: &worth_relational_storage::internal_port::storage::overlay::published_delta::RelationalPublishedPartitionDelta,
        previous: Option<&Arc<RelationalBranchRoot>>,
        previous_residency: &worth_relational_storage::internal_port::storage::residency::OperationResidency,
        envelope: Arc<CanonicalCommitEnvelope>,
        content: &worth_relational_branch::internal_port::content::publication::ContentPublication,
        registry: &contracts::schema::data::RelationalSchemaRegistry,
        symbols: &worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
        planned_schema: Option<&Arc<worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority>>,
    ) -> Result<PreparedRelationalBranchRootCapture, RelationalBranchRootCaptureDenial> {
        #[cfg(test)]
        if self.root_capture_sabotage.swap(false, Ordering::Relaxed) {
            return Err(RelationalBranchRootCaptureDenial::UnresolvedContentSymbol(
                contracts::symbols::data::Symbol(u32::MAX),
            ));
        }
        RelationalBranchRoot::prepare_capture(
            execution,
            &self.root_ids,
            partitions,
            published_delta,
            previous,
            previous_residency,
            envelope,
            content,
            registry,
            symbols,
            planned_schema,
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
        update: impl FnOnce(&mut worth_relational_branch::internal_port::runtime::state::subsystems::history::history_costs::RelationalBranchSharingCostCounters),
    ) {
        if let Some(cell) = self.branch_cell(branch) {
            cell.publication_cell().record_sharing_cost(update);
        }
    }
}

pub fn bind_preparation(
    history: &super::history_canonical_routes::RelationalPublicationHistory,
) -> RelationalPreparationHistory {
    let ledger = history.ledger.read();
    RelationalPreparationHistory::new(
        history.branch_cells.clone(),
        Arc::clone(&history.commit_identity_allocator),
        Arc::clone(&history.version_identity_allocator),
        ledger.next_commit_id,
        ledger.next_version_id,
        Arc::clone(&history.canonical_publication_routes),
        history.root_identity_issuer.clone(),
        #[cfg(test)]
        history.root_capture_switch_for_test(),
    )
}
