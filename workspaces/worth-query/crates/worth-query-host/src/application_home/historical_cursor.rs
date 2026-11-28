//! Retained-basis historical cursor lifecycle on [`WorthQueryApplicationHome`]
//! (Store Milestone 4, Phase M4.4).

pub(super) mod branch_custody;
mod custody_state;
mod query_stream;

use worth_execution::ExecutionRequest;
use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::primary_graph::KnownProjectedHeapBytes;
use worth_relational::facade::config::ResidencyBudget;
use worth_relational::facade::history::{BranchId, CommitId};
use worth_runtime_world::facade::{
    BranchOriginIdentity, HistoricalBasisRetentionKind, HistoricalReadDenial, SchemaVersionIdentity,
};
pub use worth_store_application::{
    AdmittedQueryWindowBinding, AdmittedQueryWindowPage, AdmittedQueryWindowSource,
    BorrowedHistoricalRow, DurableHistoricalBasisStore, HistoricalCursor, HistoricalCursorPage,
    HistoricalCursorStream, MemoryHistoricalBasisStore, ReaderLoan, ReaderLoanGuard,
    RetainedBasisToken, RetainedRevisionReceipt,
};

use super::WorthQueryApplicationHome;
use crate::session::{WorthQueryHostSession, WorthQuerySessionQuery};
use branch_custody::InflightRetainGuard;
pub(crate) use custody_state::RetainedBranchCustody;
use query_stream::{
    compute_query_basis_commitment, compute_query_origin_digest, corrupt, derive_query_contract,
    derive_query_contract_digest, derive_request_binding_digest,
};

impl<Schema, Program> WorthQueryApplicationHome<Schema, Program> {
    /// Opens or restores the Home's shared durable retained-basis historical cursor store.
    pub fn durable_historical_basis_store(
        &self,
        budget: ResidencyBudget,
    ) -> Option<Result<DurableHistoricalBasisStore, HistoricalReadDenial>> {
        self.durable_historical_store_ref(budget)
            .map(|res| res.cloned())
    }

    pub(super) fn durable_historical_store_ref(
        &self,
        budget: ResidencyBudget,
    ) -> Option<Result<&DurableHistoricalBasisStore, HistoricalReadDenial>> {
        let dir = self.directory.as_deref()?;
        if let Some(existing) = self.durable_historical_bases.get() {
            return Some(Ok(existing));
        }
        match dir.historical_basis_store(budget) {
            Ok(store) => {
                let _ = self.durable_historical_bases.set(store);
                Some(Ok(self
                    .durable_historical_bases
                    .get()
                    .expect("initialized")))
            }
            Err(err) => Some(Err(err)),
        }
    }

    /// Returns the Home's shared Memory-backend retained-basis historical cursor store.
    pub fn memory_historical_basis_store(
        &self,
        home_identity: HomeIdentity,
        budget: ResidencyBudget,
    ) -> MemoryHistoricalBasisStore {
        self.memory_historical_store_ref(home_identity, budget)
            .clone()
    }

    pub(super) fn memory_historical_store_ref(
        &self,
        home_identity: HomeIdentity,
        budget: ResidencyBudget,
    ) -> &MemoryHistoricalBasisStore {
        self.memory_historical_bases.get_or_init(|| {
            let incarnation = self
                .home_incarnation()
                .unwrap_or_else(HomeIncarnation::initial);
            MemoryHistoricalBasisStore::new(home_identity, incarnation, budget)
        })
    }
}

impl<Schema, Program> WorthQueryApplicationHome<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: Send + Sync,
{
    /// Retains the Home's exact current active committed revision as a durable
    /// O(1) CoW product branch pinned under Store's `PhysicalRootPins` (Directory)
    /// or `VolatileRootPinTable` (Memory).
    ///
    /// Durably journals the deterministic planned branch name BEFORE calling
    /// `fork_preview` so any crash between fork and catalog commit is
    /// deterministically discovered and retired on reopen.
    pub fn retain_historical_basis(
        &self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
        label: impl Into<String>,
        retention_kind: HistoricalBasisRetentionKind,
        schema_version: SchemaVersionIdentity,
        budget: ResidencyBudget,
    ) -> Result<RetainedRevisionReceipt, HistoricalReadDenial> {
        self.reconcile_pending_retired_branches(session, budget);

        let (ordinal, planned_branch_name) = self
            .primary_graph()
            .branches()
            .reserve_planned_branch_ordinal()
            .map_err(|_| corrupt())?;
        let mut inflight_guard = InflightRetainGuard::enter(self, planned_branch_name.clone());
        self.record_inflight_retain_in_store(&planned_branch_name, budget)?;

        let preview = match self.fork_preview_with_reserved_ordinal(label, ordinal) {
            Ok(preview) => preview,
            Err(_) => {
                self.mark_branch_retired_in_store(&planned_branch_name, budget);
                return Err(corrupt());
            }
        };
        let forked_branch = preview.branch();
        let source_branch_name = preview
            .expected_source()
            .branch_identity()
            .name()
            .as_str()
            .to_owned();
        let pinned_obs = match session.for_branch(forked_branch).retain_read_observation() {
            Ok(obs) => obs,
            Err(_) => {
                self.discard_unclaimed_preview(preview, &planned_branch_name, budget);
                return Err(corrupt());
            }
        };
        let retained_branch_name = pinned_obs.branch_identity().name().as_str().to_owned();
        let commit_id = CommitId(pinned_obs.selected_commit().ordinal());
        let snapshot_version = pinned_obs.relational_snapshot_version();
        drop(pinned_obs);

        if retained_branch_name != planned_branch_name {
            inflight_guard.add_branch(retained_branch_name.clone());
            let _ = self.record_inflight_retain_in_store(&retained_branch_name, budget);
            self.mark_branch_retired_in_store(&planned_branch_name, budget);
        }

        let origin_digest = compute_query_origin_digest(&source_branch_name, commit_id);
        let Some(branch_origin) =
            BranchOriginIdentity::from_owner(BranchId(source_branch_name), origin_digest)
        else {
            self.discard_unclaimed_preview(preview, &retained_branch_name, budget);
            return Err(corrupt());
        };
        let commitment = compute_query_basis_commitment(
            &retained_branch_name,
            commit_id,
            &schema_version,
            &origin_digest,
        );

        let result = if let Some(store_res) = self.durable_historical_store_ref(budget) {
            store_res.and_then(|store| {
                store.retain_query_branch_basis(
                    commit_id,
                    retained_branch_name.clone(),
                    snapshot_version,
                    branch_origin,
                    schema_version,
                    retention_kind,
                    commitment,
                )
            })
        } else {
            self.memory_historical_store_ref(self.memory_home_identity(), budget)
                .retain_query_branch_basis(
                    commit_id,
                    retained_branch_name.clone(),
                    snapshot_version,
                    branch_origin,
                    schema_version,
                    retention_kind,
                    commitment,
                )
        };

        match result {
            Ok(receipt)
                if receipt.retained_branch_name.as_deref()
                    == Some(retained_branch_name.as_str()) =>
            {
                drop(preview);
                drop(inflight_guard);
                Ok(receipt)
            }
            Ok(receipt) => {
                self.discard_unclaimed_preview(preview, &retained_branch_name, budget);
                drop(inflight_guard);
                Ok(receipt)
            }
            Err(err) => {
                self.discard_unclaimed_preview(preview, &retained_branch_name, budget);
                drop(inflight_guard);
                Err(err)
            }
        }
    }

    /// Prunes a retained historical revision: atomically expires its Store pin
    /// under the reclaim fence and records durable pending branch retirement,
    /// clearing retry state ONLY once `close_branch` confirms closure.
    pub fn prune_historical_basis(
        &self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
        commit_id: CommitId,
        budget: ResidencyBudget,
    ) -> Result<bool, HistoricalReadDenial> {
        let (branch_name, pruned) =
            if let Some(store_res) = self.durable_historical_store_ref(budget) {
                let store = store_res?;
                (
                    store.retained_branch_name(commit_id),
                    store.prune_basis(commit_id)?.is_some(),
                )
            } else {
                let store = self.memory_historical_store_ref(self.memory_home_identity(), budget);
                (
                    store.retained_branch_name(commit_id),
                    store.prune_basis(commit_id)?.is_some(),
                )
            };
        if let Some(name) = branch_name {
            self.retire_pruned_branch_custody(session, &name, budget)?;
        }
        self.reconcile_pending_retired_branches(session, budget);
        Ok(pruned)
    }

    /// Reconciles any pending retired historical branches whose loans have drained,
    /// clearing durable retry records ONLY after `close_branch` confirms closure.
    pub fn reconcile_historical_branch_retirements(
        &self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
        budget: ResidencyBudget,
    ) {
        self.reconcile_pending_retired_branches(session, budget);
    }

    /// Opens a real retained-basis Query cursor validated upfront from page 1
    /// against the admitted query binding and retained branch snapshot version.
    #[allow(clippy::too_many_arguments)]
    pub fn open_historical_query_cursor<'home, Intent>(
        &'home self,
        session: &WorthQueryHostSession<'home, Schema, Program>,
        execution: &ExecutionRequest<'_, '_>,
        commit_id: CommitId,
        root_semantic_commitment: [u8; 32],
        intent: Intent,
        supported_schema: &SchemaVersionIdentity,
        budget: ResidencyBudget,
    ) -> Result<HistoricalCursor<'home, Intent::Row>, HistoricalReadDenial>
    where
        Intent: WorthQuerySessionQuery<Schema> + Clone + Send + 'home,
        Intent::Row: KnownProjectedHeapBytes + Send + 'home,
    {
        let query = derive_query_contract(session, &intent, supported_schema)?;
        let contract_digest = derive_query_contract_digest(session, &intent, supported_schema)?;
        let request_binding_digest =
            derive_request_binding_digest(session, &intent, supported_schema)?;
        let schema_clone = supported_schema.clone();
        let make_binding = |branch: Option<&str>| AdmittedQueryWindowBinding {
            contract_digest,
            request_binding_digest,
            snapshot_version: self.resolve_branch_snapshot_version(session, branch),
        };
        let make_source = |b: Option<&str>| {
            self.build_admitted_query_source(session, commit_id, b, intent, schema_clone, budget)
        };
        if let Some(store_res) = self.durable_historical_store_ref(budget) {
            let store = store_res?;
            let binding = make_binding(store.retained_branch_name(commit_id).as_deref());
            store.open_admitted_query_cursor(
                execution,
                commit_id,
                root_semantic_commitment,
                query,
                binding,
                supported_schema,
                make_source,
            )
        } else {
            let store = self.memory_historical_store_ref(self.memory_home_identity(), budget);
            let binding = make_binding(store.retained_branch_name(commit_id).as_deref());
            store.open_admitted_query_cursor(
                execution,
                commit_id,
                root_semantic_commitment,
                query,
                binding,
                supported_schema,
                make_source,
            )
        }
    }

    /// Resumes a retained-basis Query cursor from an opaque `whc1_...` wire token.
    #[allow(clippy::too_many_arguments)]
    pub fn resume_historical_query_cursor_wire<'home, Intent>(
        &'home self,
        session: &WorthQueryHostSession<'home, Schema, Program>,
        execution: &ExecutionRequest<'_, '_>,
        wire: &str,
        intent: Intent,
        supported_schema: &SchemaVersionIdentity,
        budget: ResidencyBudget,
    ) -> Result<HistoricalCursor<'home, Intent::Row>, HistoricalReadDenial>
    where
        Intent: WorthQuerySessionQuery<Schema> + Clone + Send + 'home,
        Intent::Row: KnownProjectedHeapBytes + Send + 'home,
    {
        let token = RetainedBasisToken::decode_wire(wire)?;
        self.resume_historical_query_cursor(
            session,
            execution,
            &token,
            intent,
            supported_schema,
            budget,
        )
    }

    /// Resumes a retained-basis Query cursor from a verified [`RetainedBasisToken`].
    #[allow(clippy::too_many_arguments)]
    pub fn resume_historical_query_cursor<'home, Intent>(
        &'home self,
        session: &WorthQueryHostSession<'home, Schema, Program>,
        execution: &ExecutionRequest<'_, '_>,
        token: &RetainedBasisToken,
        intent: Intent,
        supported_schema: &SchemaVersionIdentity,
        budget: ResidencyBudget,
    ) -> Result<HistoricalCursor<'home, Intent::Row>, HistoricalReadDenial>
    where
        Intent: WorthQuerySessionQuery<Schema> + Clone + Send + 'home,
        Intent::Row: KnownProjectedHeapBytes + Send + 'home,
    {
        let expected_query = derive_query_contract(session, &intent, supported_schema)?;
        let contract_digest = derive_query_contract_digest(session, &intent, supported_schema)?;
        let request_binding_digest =
            derive_request_binding_digest(session, &intent, supported_schema)?;
        let commit_id = token.commit_id();
        let schema_clone = supported_schema.clone();
        let make_binding = |branch: Option<&str>| AdmittedQueryWindowBinding {
            contract_digest,
            request_binding_digest,
            snapshot_version: self.resolve_branch_snapshot_version(session, branch),
        };
        let make_source = |b: Option<&str>| {
            self.build_admitted_query_source(session, commit_id, b, intent, schema_clone, budget)
        };
        if let Some(store_res) = self.durable_historical_store_ref(budget) {
            let store = store_res?;
            let binding = make_binding(store.retained_branch_name(commit_id).as_deref());
            store.resume_admitted_query_cursor(
                execution,
                token,
                expected_query,
                binding,
                supported_schema,
                make_source,
            )
        } else {
            let store = self.memory_historical_store_ref(self.memory_home_identity(), budget);
            let binding = make_binding(store.retained_branch_name(commit_id).as_deref());
            store.resume_admitted_query_cursor(
                execution,
                token,
                expected_query,
                binding,
                supported_schema,
                make_source,
            )
        }
    }

    pub(super) fn memory_home_identity(&self) -> HomeIdentity {
        self.home_identity().unwrap_or_else(|| {
            HomeIdentity::from_observed_bytes([0x4D; 16]).expect("non-zero memory home id")
        })
    }
}
