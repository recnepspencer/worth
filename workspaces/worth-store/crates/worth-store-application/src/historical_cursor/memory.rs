//! Memory-backend parity retained-basis historical cursor runtime backed by
//! Store's verified `VolatileRootPinTable` (`PinTableKernel`).

use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};

use worth_execution::ExecutionRequest;
use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_relational::facade::config::ResidencyBudget;
use worth_relational::facade::history::CommitId;
use worth_runtime_world::facade::{
    HistoricalBasisRow, HistoricalQueryContract, HistoricalReadDenial,
    HistoricalResidencyPoolSnapshot, SchemaVersionIdentity,
};
use worth_store::pin_table::{RootPinDenial, RootPinId, RootPinState, VolatileRootPinTable};

use super::authority::{generate_home_mac_key, HistoricalCursorAuthority};
use super::aux_index::AuxiliaryIndexRegistry;
use super::catalog::{HistoricalBasisCatalog, RetainedBasisRecord, RetainedBranchIndex};
use super::chunk_format::ChunkMemberId;
use super::cursor::{HistoricalCursor, StoreLoanCustody};
use super::memory_support::{
    build_restarted_state, corrupt, MemoryCatalogState, MemoryLoanCustody, MemoryMemberReader,
};
use super::residency::{HeadWorkingSetGuard, HistoricalPoolBorrow, HistoricalWorkingSetPool};
#[cfg(feature = "certification-test-authority")]
use super::stream::HistoricalCursorCertificationMutant;
use super::stream::{
    prepare_open_token, validate_resume_preconditions, validate_resumed_entry,
    CanonicalChunkCursorStream,
};
use super::token::RetainedBasisToken;

pub(super) struct MemoryStoreInner {
    pub(super) home_identity: HomeIdentity,
    pub(super) home_incarnation: HomeIncarnation,
    pub(super) authority: HistoricalCursorAuthority,
    pub(super) pins: Arc<VolatileRootPinTable>,
    pub(super) residency: HistoricalWorkingSetPool,
    pub(super) aux_indexes: Arc<AuxiliaryIndexRegistry>,
    pub(super) corrupted_overrides: Arc<RwLock<BTreeMap<ChunkMemberId, Vec<u8>>>>,
    pub(super) state: Mutex<MemoryCatalogState>,
    #[cfg(feature = "certification-test-authority")]
    pub(super) mutant: Mutex<HistoricalCursorCertificationMutant>,
}

/// Shared handle to one Memory Home's retained-basis historical cursor store.
#[derive(Clone)]
pub struct MemoryHistoricalBasisStore {
    pub(super) inner: Arc<MemoryStoreInner>,
}

impl MemoryHistoricalBasisStore {
    pub fn new(
        home_identity: HomeIdentity,
        home_incarnation: HomeIncarnation,
        budget: ResidencyBudget,
    ) -> Self {
        let mac_key = generate_home_mac_key(&home_identity.bytes());
        let authority = HistoricalCursorAuthority::from_home_key(mac_key);
        Self {
            inner: Arc::new(MemoryStoreInner {
                home_identity,
                home_incarnation,
                authority,
                pins: Arc::new(VolatileRootPinTable::default()),
                residency: HistoricalWorkingSetPool::new(budget),
                aux_indexes: Arc::new(AuxiliaryIndexRegistry::default()),
                corrupted_overrides: Arc::new(RwLock::new(BTreeMap::new())),
                state: Mutex::new(MemoryCatalogState {
                    catalog: HistoricalBasisCatalog::fresh(mac_key),
                    retained_branches: RetainedBranchIndex::default(),
                    basis_manifests: BTreeMap::new(),
                    basis_members: BTreeMap::new(),
                    next_root_seq: 1,
                }),
                #[cfg(feature = "certification-test-authority")]
                mutant: Mutex::new(HistoricalCursorCertificationMutant::None),
            }),
        }
    }

    pub fn home_identity(&self) -> HomeIdentity {
        self.inner.home_identity
    }

    pub fn home_incarnation(&self) -> HomeIncarnation {
        self.inner.home_incarnation
    }

    pub fn residency_snapshot(&self) -> HistoricalResidencyPoolSnapshot {
        self.inner.residency.snapshot()
    }

    pub fn pin_head_bytes(&self, bytes: u64) -> Option<HeadWorkingSetGuard> {
        self.inner.residency.pin_head_bytes(bytes)
    }

    pub fn borrow_historical_bytes(
        &self,
        bytes: u64,
    ) -> Result<HistoricalPoolBorrow, HistoricalReadDenial> {
        self.inner.residency.borrow_historical_bytes(bytes)
    }

    pub fn discard_auxiliary_indexes(&self) -> usize {
        self.inner.aux_indexes.discard_all()
    }

    pub fn auxiliary_index_count(&self) -> usize {
        self.inner.aux_indexes.index_count()
    }

    pub fn pin_state(&self, commit_id: CommitId) -> Option<RootPinState> {
        let state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        let entry = state.catalog.bases.get(&commit_id)?;
        self.inner
            .pins
            .state(RootPinId::from_ordinal(entry.pin_ordinal))
    }

    pub fn raw_pin_state(&self, pin: RootPinId) -> Option<RootPinState> {
        self.inner.pins.state(pin)
    }

    pub fn retained_branch_name(&self, commit_id: CommitId) -> Option<String> {
        let state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        state
            .catalog
            .bases
            .get(&commit_id)
            .and_then(|entry| entry.retained_branch_name.clone())
    }

    pub fn basis_locator_summary(&self, commit_id: CommitId) -> Option<(u64, Vec<ChunkMemberId>)> {
        let state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        let entry = state.catalog.bases.get(&commit_id)?;
        let chunks = state
            .basis_manifests
            .get(&commit_id)
            .map(|c| c.iter().map(|d| d.chunk_member).collect())
            .unwrap_or_default();
        Some((entry.consolidation_epoch, chunks))
    }

    pub fn inject_corrupt_member_bytes(&self, id: ChunkMemberId, corrupt_bytes: Vec<u8>) {
        self.inner
            .corrupted_overrides
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id, corrupt_bytes);
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn set_certification_mutant(&self, mutant: HistoricalCursorCertificationMutant) {
        *self.inner.mutant.lock().unwrap_or_else(|e| e.into_inner()) = mutant;
    }

    #[cfg(feature = "certification-test-authority")]
    pub(super) fn active_mutant(&self) -> HistoricalCursorCertificationMutant {
        *self.inner.mutant.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn simulate_restart(&self, budget: ResidencyBudget) -> Self {
        let state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        let next_incarnation = self
            .inner
            .home_incarnation
            .successor()
            .unwrap_or(self.inner.home_incarnation);
        let (new_pins, aux_indexes, restarted_state) =
            build_restarted_state(&self.inner.pins, &state);
        Self {
            inner: Arc::new(MemoryStoreInner {
                home_identity: self.inner.home_identity,
                home_incarnation: next_incarnation,
                authority: self.inner.authority.clone(),
                pins: new_pins,
                residency: HistoricalWorkingSetPool::new(budget),
                aux_indexes,
                corrupted_overrides: Arc::new(RwLock::new(BTreeMap::new())),
                state: Mutex::new(restarted_state),
                #[cfg(feature = "certification-test-authority")]
                mutant: Mutex::new(HistoricalCursorCertificationMutant::None),
            }),
        }
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
        self.admit_canonical_locked(&state, entry, token, query)
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
                        &state,
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
        self.admit_canonical_locked(&state, entry, token.clone(), expected_query)
    }

    fn admit_canonical_locked<'a>(
        &'a self,
        state: &MemoryCatalogState,
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

        let custody: StoreLoanCustody<'a> = if skip_loan {
            #[cfg(feature = "certification-test-authority")]
            {
                StoreLoanCustody::CertificationSkipLoan
            }
            #[cfg(not(feature = "certification-test-authority"))]
            {
                unreachable!()
            }
        } else {
            let pin = RootPinId::from_ordinal(entry.pin_ordinal);
            self.inner.pins.loan(pin).map_err(|err| match err {
                RootPinDenial::LeaseExpired | RootPinDenial::PinNotFound => {
                    HistoricalReadDenial::HistoryUnavailable {
                        commit_id: entry.commit_id,
                    }
                }
                _ => corrupt(),
            })?;
            StoreLoanCustody::Memory(MemoryLoanCustody {
                pins: Arc::clone(&self.inner.pins),
                pin,
            })
        };
        let pinned_members = state
            .basis_members
            .get(&entry.commit_id)
            .cloned()
            .unwrap_or_else(|| Arc::new(BTreeMap::new()));
        let chunks = state
            .basis_manifests
            .get(&entry.commit_id)
            .cloned()
            .unwrap_or_else(|| Arc::from(Vec::new()));
        let mut stream = CanonicalChunkCursorStream::new(
            entry.commit_id,
            entry.root_semantic_commitment,
            chunks,
            query,
            projection,
            self.inner.residency.clone(),
            Arc::clone(&self.inner.aux_indexes),
            Box::new(MemoryMemberReader {
                pinned_members,
                corrupted_overrides: Arc::clone(&self.inner.corrupted_overrides),
            }),
            #[cfg(feature = "certification-test-authority")]
            mutant,
        );
        let active = Arc::new(AtomicBool::new(true));
        Ok(self.inner.authority.admit_cursor(
            token,
            custody,
            active,
            Box::new(move |execution, after, limit| stream.read_page(execution, after, limit)),
        ))
    }
}
