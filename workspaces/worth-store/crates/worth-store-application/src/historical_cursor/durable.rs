//! Directory-backed durable retained-basis historical cursor runtime wired directly
//! to Store's `ServingPhysicalRuntime` (`PhysicalRootPins` + `AnchoredRecords`).
//!
//! Construction is strictly `pub(crate)` (Home-owner only via `HomeContext` /
//! `ReadmittedApplicationHome::historical_basis_store`). Shared per Home via
//! `Clone` over one `Arc<DurableStoreInner>` (ONE catalog, ONE publication/reclaim
//! fence, ONE residency pool, ONE Home-keyed MAC authority).

use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};

use worth_execution::ExecutionRequest;
use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_relational::facade::config::ResidencyBudget;
use worth_relational::facade::history::CommitId;
use worth_runtime_world::facade::{
    HistoricalBasisRow, HistoricalQueryContract, HistoricalReadDenial, SchemaVersionIdentity,
};
use worth_store::physical_runtime::ServingPhysicalRuntime;
use worth_store::pin_table::{RootPinDenial, RootPinId};

use super::authority::{generate_home_mac_key, HistoricalCursorAuthority};
use super::aux_index::AuxiliaryIndexRegistry;
use super::catalog::{HistoricalBasisCatalog, RetainedBasisRecord, RetainedBranchIndex};
use super::chunk_format::ChunkMemberId;
use super::cursor::{HistoricalCursor, StoreLoanCustody};
use super::durable_io::{
    anchor_name, corrupt, publish_catalog, DurableCatalogState, DurableMemberReader,
    MAX_CATALOG_BYTES,
};
use super::durable_reconcile::reconcile_on_open;
#[cfg(feature = "certification-test-authority")]
use super::durable_reconcile::DurableHistoricalCrashCut;
use super::residency::HistoricalWorkingSetPool;
#[cfg(feature = "certification-test-authority")]
use super::stream::HistoricalCursorCertificationMutant;
use super::stream::{
    prepare_open_token, validate_resume_preconditions, validate_resumed_entry,
    CanonicalChunkCursorStream,
};
use super::token::RetainedBasisToken;

pub(super) struct DurableStoreInner {
    pub(super) serving: Arc<ServingPhysicalRuntime>,
    pub(super) home_identity: HomeIdentity,
    pub(super) home_incarnation: HomeIncarnation,
    pub(super) authority: HistoricalCursorAuthority,
    pub(super) residency: HistoricalWorkingSetPool,
    pub(super) aux_indexes: Arc<AuxiliaryIndexRegistry>,
    pub(super) corrupted_overrides: Arc<RwLock<BTreeMap<ChunkMemberId, Vec<u8>>>>,
    pub(super) state: Mutex<DurableCatalogState>,
    #[cfg(feature = "certification-test-authority")]
    pub(super) mutant: Mutex<HistoricalCursorCertificationMutant>,
    #[cfg(feature = "certification-test-authority")]
    pub(super) crash_cut: Mutex<Option<DurableHistoricalCrashCut>>,
}

/// Shared handle to one Home's durable retained-basis historical cursor store.
#[derive(Clone)]
pub struct DurableHistoricalBasisStore {
    pub(super) inner: Arc<DurableStoreInner>,
}

impl DurableHistoricalBasisStore {
    /// Crate-private Home-owner constructor: opens or restores the durable
    /// retained-basis catalog from Store's `"home.historical-bases"` anchor
    /// and reconciles it against Store's already-restored `PhysicalRootPins`.
    pub(crate) fn open_or_restore(
        serving: Arc<ServingPhysicalRuntime>,
        home_identity: HomeIdentity,
        home_incarnation: HomeIncarnation,
        budget: ResidencyBudget,
    ) -> Result<Self, HistoricalReadDenial> {
        let records = serving.anchored_records().map_err(|_| corrupt())?;
        let aux_indexes = Arc::new(AuxiliaryIndexRegistry::default());

        let (anchor_generation, catalog) = match records
            .read(anchor_name(), MAX_CATALOG_BYTES)
            .map_err(|_| corrupt())?
        {
            None => (
                0,
                HistoricalBasisCatalog::fresh(generate_home_mac_key(&home_identity.bytes())),
            ),
            Some(record) => {
                let decoded = HistoricalBasisCatalog::decode(record.bytes())?;
                (record.version().generation(), decoded)
            }
        };

        let authority = HistoricalCursorAuthority::from_home_key(catalog.mac_key);
        let retained_branches = RetainedBranchIndex::from_catalog(&catalog);
        let mut durable_state = DurableCatalogState {
            anchor_generation,
            basis_stage_generation: 0,
            catalog,
            retained_branches,
            needs_reconcile: false,
        };
        reconcile_on_open(&serving, &mut durable_state)?;

        Ok(Self {
            inner: Arc::new(DurableStoreInner {
                serving,
                home_identity,
                home_incarnation,
                authority,
                residency: HistoricalWorkingSetPool::new(budget),
                aux_indexes,
                corrupted_overrides: Arc::new(RwLock::new(BTreeMap::new())),
                state: Mutex::new(durable_state),
                #[cfg(feature = "certification-test-authority")]
                mutant: Mutex::new(HistoricalCursorCertificationMutant::None),
                #[cfg(feature = "certification-test-authority")]
                crash_cut: Mutex::new(None),
            }),
        })
    }

    /// Cold Home open helper: if a persisted `"home.historical-bases"` catalog
    /// exists, reconciles crash-cut pins and promotes unclaimed
    /// `inflight_retain_branches` into `pending_retire_branches`, returning the
    /// unretained branches that should be closed immediately during Home startup
    /// without locking in a premature `ResidencyBudget`.
    pub(crate) fn recover_persisted_branches_on_open(
        serving: &ServingPhysicalRuntime,
    ) -> Result<Vec<String>, HistoricalReadDenial> {
        let records = serving.anchored_records().map_err(|_| corrupt())?;
        let Some(record) = records
            .read(anchor_name(), MAX_CATALOG_BYTES)
            .map_err(|_| corrupt())?
        else {
            return Ok(Vec::new());
        };
        let catalog = HistoricalBasisCatalog::decode(record.bytes())?;
        let retained_branches = RetainedBranchIndex::from_catalog(&catalog);
        let mut state = DurableCatalogState {
            anchor_generation: record.version().generation(),
            basis_stage_generation: 0,
            catalog,
            retained_branches,
            needs_reconcile: false,
        };
        reconcile_on_open(serving, &mut state)?;
        let to_retire = state
            .catalog
            .pending_retire_branches
            .into_iter()
            .filter(|name| !state.retained_branches.contains(name))
            .collect();
        Ok(to_retire)
    }

    /// Cold Home open helper: clears `branch_name` from durable
    /// `pending_retire_branches` and `inflight_retain_branches` ONLY after
    /// `close_branch` confirms closure.
    pub(crate) fn mark_persisted_branch_retired_on_open(
        serving: &ServingPhysicalRuntime,
        branch_name: &str,
    ) -> Result<(), HistoricalReadDenial> {
        let records = serving.anchored_records().map_err(|_| corrupt())?;
        let Some(record) = records
            .read(anchor_name(), MAX_CATALOG_BYTES)
            .map_err(|_| corrupt())?
        else {
            return Ok(());
        };
        let catalog = HistoricalBasisCatalog::decode(record.bytes())?;
        let retained_branches = RetainedBranchIndex::from_catalog(&catalog);
        let mut state = DurableCatalogState {
            anchor_generation: record.version().generation(),
            basis_stage_generation: 0,
            catalog,
            retained_branches,
            needs_reconcile: false,
        };
        let mut next = state.catalog.clone();
        next.pending_retire_branches
            .retain(|name| name != branch_name);
        next.inflight_retain_branches
            .retain(|name| name != branch_name);
        if next != state.catalog {
            publish_catalog(serving, &mut state, &next)?;
        }
        Ok(())
    }

    pub fn open_historical_cursor<'a>(
        &'a self,
        execution: &ExecutionRequest<'_, '_>,
        commit_id: CommitId,
        root_semantic_commitment: [u8; 32],
        query: HistoricalQueryContract,
        supported_schema: &SchemaVersionIdentity,
    ) -> Result<HistoricalCursor<'a, HistoricalBasisRow>, HistoricalReadDenial> {
        if execution.consult().is_err() {
            return Err(HistoricalReadDenial::RequestStopped);
        }
        let state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        let entry = state
            .catalog
            .bases
            .get(&commit_id)
            .ok_or(HistoricalReadDenial::HistoryUnavailable { commit_id })?;
        let token = prepare_open_token(
            &self.inner.authority,
            execution,
            self.inner.home_identity,
            self.inner.home_incarnation,
            commit_id,
            root_semantic_commitment,
            entry,
            &query,
            supported_schema,
            true,
        )?;
        self.admit_canonical_locked(execution, entry, token, query)
    }

    pub fn resume_cursor_wire<'a>(
        &'a self,
        execution: &ExecutionRequest<'_, '_>,
        wire: &str,
        expected_query: HistoricalQueryContract,
        supported_schema: &SchemaVersionIdentity,
    ) -> Result<HistoricalCursor<'a, HistoricalBasisRow>, HistoricalReadDenial> {
        let token = RetainedBasisToken::decode_wire(wire)?;
        self.resume_cursor(execution, &token, expected_query, supported_schema)
    }

    pub fn resume_cursor<'a>(
        &'a self,
        execution: &ExecutionRequest<'_, '_>,
        token: &RetainedBasisToken,
        expected_query: HistoricalQueryContract,
        supported_schema: &SchemaVersionIdentity,
    ) -> Result<HistoricalCursor<'a, HistoricalBasisRow>, HistoricalReadDenial> {
        #[cfg(feature = "certification-test-authority")]
        let mutant = self.active_mutant();
        #[cfg(feature = "certification-test-authority")]
        let (verify_mac, verify_query) = (
            mutant != HistoricalCursorCertificationMutant::SkipTokenMacVerification,
            mutant != HistoricalCursorCertificationMutant::SkipQueryDigestValidation,
        );
        #[cfg(not(feature = "certification-test-authority"))]
        let (verify_mac, verify_query) = (true, true);

        validate_resume_preconditions(
            &self.inner.authority,
            execution,
            self.inner.home_identity,
            self.inner.home_incarnation,
            token,
            &expected_query,
            supported_schema,
            true,
            verify_mac,
            verify_query,
        )?;
        let state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        let entry = match state.catalog.bases.get(&token.commit_id()) {
            Some(found) => found,
            None => {
                #[cfg(feature = "certification-test-authority")]
                if mutant == HistoricalCursorCertificationMutant::RebindToHeadOnPruned
                    && let Some((_, head_entry)) = state.catalog.bases.iter().next_back()
                {
                    return self.admit_canonical_locked(
                        execution,
                        head_entry,
                        token.clone(),
                        expected_query,
                    );
                }
                return Err(HistoricalReadDenial::HistoryUnavailable {
                    commit_id: token.commit_id(),
                });
            }
        };
        validate_resumed_entry(entry, token, supported_schema)?;
        self.admit_canonical_locked(execution, entry, token.clone(), expected_query)
    }

    fn admit_canonical_locked<'a>(
        &'a self,
        _execution: &ExecutionRequest<'_, '_>,
        entry: &RetainedBasisRecord,
        token: RetainedBasisToken,
        query: HistoricalQueryContract,
    ) -> Result<HistoricalCursor<'a, HistoricalBasisRow>, HistoricalReadDenial> {
        let projection = query.admitted_canonical_projection().ok_or_else(|| {
            HistoricalReadDenial::UnsupportedHistoricalMeaning {
                schema_version: query.schema_version().clone(),
            }
        })?;
        #[cfg(feature = "certification-test-authority")]
        let mutant = self.active_mutant();
        #[cfg(feature = "certification-test-authority")]
        let skip_loan = mutant == HistoricalCursorCertificationMutant::SkipPinLoanAcquisition;
        #[cfg(not(feature = "certification-test-authority"))]
        let skip_loan = false;

        let (custody, member_reader): (StoreLoanCustody<'a>, DurableMemberReader) = if skip_loan {
            #[cfg(feature = "certification-test-authority")]
            {
                let pinned_reader = self.inner.serving.records().map_err(|_| corrupt())?;
                let reader = DurableMemberReader::new(
                    pinned_reader,
                    Arc::clone(&self.inner.corrupted_overrides),
                );
                (StoreLoanCustody::CertificationSkipLoan, reader)
            }
            #[cfg(not(feature = "certification-test-authority"))]
            {
                unreachable!()
            }
        } else {
            let pins = self.inner.serving.root_pins().map_err(|_| corrupt())?;
            let pin_loan = pins
                .loan(RootPinId::from_ordinal(entry.pin_ordinal))
                .map_err(|err| match err {
                    RootPinDenial::LeaseExpired | RootPinDenial::PinNotFound => {
                        HistoricalReadDenial::HistoryUnavailable {
                            commit_id: entry.commit_id,
                        }
                    }
                    _ => corrupt(),
                })?;
            let pinned_reader = pin_loan.records().map_err(|_| corrupt())?;
            let reader = DurableMemberReader::new(
                pinned_reader,
                Arc::clone(&self.inner.corrupted_overrides),
            );
            (StoreLoanCustody::Durable(pin_loan), reader)
        };

        let manifest_ref = entry
            .basis_manifest_member
            .map(|id| (id, entry.basis_manifest_locator));
        let mut stream = CanonicalChunkCursorStream::new(
            entry.commit_id,
            entry.root_semantic_commitment,
            Arc::from(Vec::new()),
            query,
            projection,
            self.inner.residency.clone(),
            Arc::clone(&self.inner.aux_indexes),
            Box::new(member_reader),
            #[cfg(feature = "certification-test-authority")]
            mutant,
        )
        .with_manifest_ref(manifest_ref);
        let active = Arc::new(AtomicBool::new(true));
        Ok(self.inner.authority.admit_cursor(
            token,
            custody,
            active,
            Box::new(move |execution, after, limit| stream.read_page(execution, after, limit)),
        ))
    }
}
