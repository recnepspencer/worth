//! Owner-verified Query-Host window cursor admission and retained branch
//! registration on [`MemoryHistoricalBasisStore`] (Store Milestone 4, Phase M4.4).

use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use worth_execution::ExecutionRequest;
use worth_query_execution::facade::primary_graph::KnownProjectedHeapBytes;
use worth_relational::facade::history::CommitId;
use worth_relational::facade::identity::VersionId;
use worth_runtime_world::facade::{
    BranchOriginIdentity, HistoricalBasisRetentionKind, HistoricalQueryContract,
    HistoricalReadDenial, SchemaVersionIdentity,
};
use worth_store::pin_table::{RootPinDenial, RootPinDropPolicy, RootPinId, RootPinState};

use super::catalog::RetainedBasisRecord;
use super::cursor::{HistoricalCursor, StoreLoanCustody};
use super::durable_query::RetainedRevisionReceipt;
use super::durable_reconcile::check_duplicate_retain;
use super::memory::MemoryHistoricalBasisStore;
use super::memory_support::{corrupt, MemoryLoanCustody};
use super::query_window::{
    AdmittedQueryWindowBinding, AdmittedQueryWindowSource, AdmittedQueryWindowStream,
};
use super::stream::{prepare_open_token, validate_resume_preconditions, validate_resumed_entry};
use super::token::RetainedBasisToken;

impl MemoryHistoricalBasisStore {
    pub fn pending_retire_branches(&self) -> Vec<String> {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .catalog
            .pending_retire_branches
            .clone()
    }

    pub fn inflight_retain_branches(&self) -> Vec<String> {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .catalog
            .inflight_retain_branches
            .clone()
    }

    pub fn is_retained_branch(&self, branch_name: &str) -> bool {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .retained_branches
            .contains(branch_name)
    }

    pub fn record_inflight_retain_intent(&self, branch_name: &str) {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        if !state
            .catalog
            .inflight_retain_branches
            .iter()
            .any(|name| name == branch_name)
        {
            state
                .catalog
                .inflight_retain_branches
                .push(branch_name.to_owned());
        }
    }

    pub fn abandon_inflight_retain_intent(&self, branch_name: &str) {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        state
            .catalog
            .inflight_retain_branches
            .retain(|name| name != branch_name);
        if !state
            .catalog
            .pending_retire_branches
            .iter()
            .any(|name| name == branch_name)
        {
            state
                .catalog
                .pending_retire_branches
                .push(branch_name.to_owned());
        }
    }

    pub fn record_pending_branch_intent(&self, branch_name: &str) {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        if !state
            .catalog
            .pending_retire_branches
            .iter()
            .any(|name| name == branch_name)
        {
            state
                .catalog
                .pending_retire_branches
                .push(branch_name.to_owned());
        }
    }

    pub fn mark_branch_retired(&self, branch_name: &str) {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        state
            .catalog
            .pending_retire_branches
            .retain(|name| name != branch_name);
        state
            .catalog
            .inflight_retain_branches
            .retain(|name| name != branch_name);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn retain_query_branch_basis(
        &self,
        commit_id: CommitId,
        retained_branch_name: String,
        retained_snapshot_version: VersionId,
        branch_origin: BranchOriginIdentity,
        schema_version: SchemaVersionIdentity,
        retention_kind: HistoricalBasisRetentionKind,
        root_semantic_commitment: [u8; 32],
    ) -> Result<RetainedRevisionReceipt, HistoricalReadDenial> {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(existing) = state.catalog.bases.get(&commit_id) {
            check_duplicate_retain(
                existing,
                root_semantic_commitment,
                &branch_origin,
                &schema_version,
                retention_kind,
                Some(retained_branch_name.as_str()),
                Some(retained_snapshot_version.0),
            )?;
            let pin_id = RootPinId::from_ordinal(existing.pin_ordinal);
            if self.inner.pins.state(pin_id) == Some(RootPinState::Active) {
                return Ok(RetainedRevisionReceipt {
                    commit_id,
                    root_semantic_commitment: existing.root_semantic_commitment,
                    pin_id,
                    branch_origin: existing.branch_origin.clone(),
                    schema_version: existing.schema_version.clone(),
                    retained_branch_name: existing.retained_branch_name.clone(),
                });
            }
        }
        let seq = state.next_root_seq;
        state.next_root_seq = state.next_root_seq.saturating_add(1);
        let pin_id = self
            .inner
            .pins
            .pin(
                seq,
                None,
                RootPinDropPolicy::RetainUntilChildrenDrain,
                root_semantic_commitment.to_vec(),
            )
            .map_err(|_| corrupt())?;
        let previous_branch = state
            .catalog
            .bases
            .insert(
                commit_id,
                Arc::new(RetainedBasisRecord {
                    commit_id,
                    root_semantic_commitment,
                    branch_origin: branch_origin.clone(),
                    schema_version: schema_version.clone(),
                    retention_kind,
                    pin_ordinal: pin_id.ordinal(),
                    previous_pin_ordinal: None,
                    consolidation_epoch: 0,
                    retained_branch_name: Some(retained_branch_name.clone()),
                    retained_snapshot_version: Some(retained_snapshot_version.0),
                    basis_manifest_member: None,
                    basis_manifest_locator: None,
                }),
            )
            .and_then(|old| old.retained_branch_name.clone());
        state
            .retained_branches
            .decrement(previous_branch.as_deref());
        state
            .retained_branches
            .increment(Some(retained_branch_name.as_str()));
        state
            .catalog
            .inflight_retain_branches
            .retain(|name| name != &retained_branch_name);
        state
            .catalog
            .pending_retire_branches
            .retain(|name| name != &retained_branch_name);
        state
            .basis_manifests
            .insert(commit_id, Arc::from(Vec::new()));
        state
            .basis_members
            .insert(commit_id, Arc::new(BTreeMap::new()));
        Ok(RetainedRevisionReceipt {
            commit_id,
            root_semantic_commitment,
            pin_id,
            branch_origin,
            schema_version,
            retained_branch_name: Some(retained_branch_name),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn open_admitted_query_cursor<'a, Row: KnownProjectedHeapBytes + 'a>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        commit_id: CommitId,
        root_semantic_commitment: [u8; 32],
        query: HistoricalQueryContract,
        expected_binding: AdmittedQueryWindowBinding,
        supported_schema: &SchemaVersionIdentity,
        build_source: impl FnOnce(
            Option<&str>,
        ) -> Result<
            Box<dyn AdmittedQueryWindowSource<Row> + 'a>,
            HistoricalReadDenial,
        >,
    ) -> Result<HistoricalCursor<'a, Row>, HistoricalReadDenial> {
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
            false,
        )?;
        self.admit_query_window_locked(entry, token, expected_binding, build_source)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn resume_admitted_query_cursor_wire<'a, Row: KnownProjectedHeapBytes + 'a>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        wire: &str,
        expected_query: HistoricalQueryContract,
        expected_binding: AdmittedQueryWindowBinding,
        supported_schema: &SchemaVersionIdentity,
        build_source: impl FnOnce(
            Option<&str>,
        ) -> Result<
            Box<dyn AdmittedQueryWindowSource<Row> + 'a>,
            HistoricalReadDenial,
        >,
    ) -> Result<HistoricalCursor<'a, Row>, HistoricalReadDenial> {
        let token = RetainedBasisToken::decode_wire(wire)?;
        self.resume_admitted_query_cursor(
            execution,
            &token,
            expected_query,
            expected_binding,
            supported_schema,
            build_source,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn resume_admitted_query_cursor<'a, Row: KnownProjectedHeapBytes + 'a>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        token: &RetainedBasisToken,
        expected_query: HistoricalQueryContract,
        expected_binding: AdmittedQueryWindowBinding,
        supported_schema: &SchemaVersionIdentity,
        build_source: impl FnOnce(
            Option<&str>,
        ) -> Result<
            Box<dyn AdmittedQueryWindowSource<Row> + 'a>,
            HistoricalReadDenial,
        >,
    ) -> Result<HistoricalCursor<'a, Row>, HistoricalReadDenial> {
        validate_resume_preconditions(
            &self.inner.authority,
            execution,
            self.inner.home_identity,
            self.inner.home_incarnation,
            token,
            &expected_query,
            supported_schema,
            false,
            true,
            true,
        )?;
        let state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        let entry = state.catalog.bases.get(&token.commit_id()).ok_or(
            HistoricalReadDenial::HistoryUnavailable {
                commit_id: token.commit_id(),
            },
        )?;
        validate_resumed_entry(entry, token, supported_schema)?;
        self.admit_query_window_locked(entry, token.clone(), expected_binding, build_source)
    }

    fn admit_query_window_locked<'a, Row: KnownProjectedHeapBytes + 'a>(
        &self,
        entry: &RetainedBasisRecord,
        token: RetainedBasisToken,
        expected_binding: AdmittedQueryWindowBinding,
        build_source: impl FnOnce(
            Option<&str>,
        ) -> Result<
            Box<dyn AdmittedQueryWindowSource<Row> + 'a>,
            HistoricalReadDenial,
        >,
    ) -> Result<HistoricalCursor<'a, Row>, HistoricalReadDenial> {
        if entry.retained_snapshot_version != Some(expected_binding.snapshot_version.0) {
            return Err(corrupt());
        }
        let pin = RootPinId::from_ordinal(entry.pin_ordinal);
        self.inner.pins.loan(pin).map_err(|err| match err {
            RootPinDenial::LeaseExpired | RootPinDenial::PinNotFound => {
                HistoricalReadDenial::HistoryUnavailable {
                    commit_id: entry.commit_id,
                }
            }
            _ => corrupt(),
        })?;
        let custody = MemoryLoanCustody {
            pins: Arc::clone(&self.inner.pins),
            pin,
        };
        let source = build_source(entry.retained_branch_name.as_deref())?;
        let streaming_window =
            (self.inner.residency.budget().streaming_window_bytes as usize).max(1);
        let mut window_stream = AdmittedQueryWindowStream::new(
            source,
            self.inner.authority.clone(),
            self.inner.home_identity,
            entry.commit_id,
            *token.query_digest(),
            entry.schema_version.clone(),
            expected_binding,
            token.canonical_key_position(),
            streaming_window,
        )?;
        let active = Arc::new(AtomicBool::new(true));
        Ok(self.inner.authority.admit_cursor(
            token,
            StoreLoanCustody::Memory(custody),
            active,
            Box::new(move |execution, after, limit| {
                window_stream.read_page(execution, after, limit)
            }),
        ))
    }
}
