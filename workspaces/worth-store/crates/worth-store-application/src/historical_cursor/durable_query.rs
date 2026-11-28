//! Real Relational revision retention and owner-verified Query-Host window
//! cursor admission on [`DurableHistoricalBasisStore`] (Store Milestone 4, Phase M4.4).
//!
//! Projected Query cursor admission accepts ONLY an [`AdmittedQueryWindowSource`]
//! whose pages carry real execution-minted [`WorthQueryApplicationRootWindowPage`]s
//! verified from page 1 and on resume against an upfront [`AdmittedQueryWindowBinding`].

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use worth_execution::ExecutionRequest;
use worth_query_execution::facade::primary_graph::KnownProjectedHeapBytes;
use worth_relational::facade::history::{BranchId, CommitId};
use worth_relational::facade::identity::{KindId, VersionId};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::schema::{SchemaId, SchemaVersionId};
use worth_runtime_world::facade::{
    BranchOriginIdentity, HistoricalBasisRetentionKind, HistoricalQueryContract,
    HistoricalReadDenial, SchemaVersionIdentity,
};
use worth_store::pin_table::{RootPinDenial, RootPinDropPolicy, RootPinId, RootPinState};

use super::catalog::RetainedBasisRecord;
use super::chunk_format::HistoricalChunkGeometry;
use super::cursor::{HistoricalCursor, StoreLoanCustody};
use super::durable::DurableHistoricalBasisStore;
use super::durable_io::{corrupt, encode_pin_payload, pin_write, publish_catalog};
use super::durable_reconcile::{check_duplicate_retain, ensure_reconciled_locked};
use super::query_window::{
    AdmittedQueryWindowBinding, AdmittedQueryWindowSource, AdmittedQueryWindowStream,
};
use super::relational_capture::capture_relational_branch_revision;
use super::stream::{prepare_open_token, validate_resume_preconditions, validate_resumed_entry};
use super::token::RetainedBasisToken;

/// Receipt returned when retaining a real Relational or Query-Host branch revision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainedRevisionReceipt {
    pub commit_id: CommitId,
    pub root_semantic_commitment: [u8; 32],
    pub pin_id: RootPinId,
    pub branch_origin: BranchOriginIdentity,
    pub schema_version: SchemaVersionIdentity,
    pub retained_branch_name: Option<String>,
}

impl DurableHistoricalBasisStore {
    /// Captures and durably retains a real committed Relational branch revision
    /// directly from `runtime` (`CHK24`/`REG8` truth) under Store's `PhysicalRootPins`.
    #[allow(clippy::too_many_arguments)]
    pub fn retain_relational_branch_revision(
        &self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        branch_id: &BranchId,
        schema_id: SchemaId,
        schema_version_id: SchemaVersionId,
        entity_kinds: &[KindId],
        relation_kinds: &[KindId],
        retention_kind: HistoricalBasisRetentionKind,
        geometry: HistoricalChunkGeometry,
    ) -> Result<RetainedRevisionReceipt, HistoricalReadDenial> {
        let captured = capture_relational_branch_revision(
            runtime,
            execution,
            branch_id,
            schema_id,
            schema_version_id,
            entity_kinds,
            relation_kinds,
        )?;
        let (comm, pin_id) = self.retain_basis_internal(
            captured.commit_id,
            captured.branch_origin.clone(),
            captured.schema_version.clone(),
            retention_kind,
            Some(branch_id.0.clone()),
            captured.rows,
            geometry,
        )?;
        Ok(RetainedRevisionReceipt {
            commit_id: captured.commit_id,
            root_semantic_commitment: comm,
            pin_id,
            branch_origin: captured.branch_origin,
            schema_version: captured.schema_version,
            retained_branch_name: Some(branch_id.0.clone()),
        })
    }

    /// Registers a real durable Query/Relational branch revision (backed by a live
    /// or archived branch in the Home's `CHK24`/`REG8` store) under Store's `PhysicalRootPins`.
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
        let _ = ensure_reconciled_locked(&self.inner.serving, &mut state, &self.inner.aux_indexes)?;
        let pins = self.inner.serving.root_pins().map_err(|_| corrupt())?;
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
            if pins.state(pin_id) == Some(RootPinState::Active) {
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
        let mut next_catalog = state.catalog.clone();
        next_catalog.bases.insert(
            commit_id,
            Arc::new(RetainedBasisRecord {
                commit_id,
                root_semantic_commitment,
                branch_origin: branch_origin.clone(),
                schema_version: schema_version.clone(),
                retention_kind,
                pin_ordinal: 0,
                previous_pin_ordinal: None,
                consolidation_epoch: 0,
                retained_branch_name: Some(retained_branch_name.clone()),
                retained_snapshot_version: Some(retained_snapshot_version.0),
                basis_manifest_member: None,
                basis_manifest_locator: None,
            }),
        );
        state.needs_reconcile = true;
        publish_catalog(&self.inner.serving, &mut state, &next_catalog)?;
        let pin_id = pins
            .pin_current(
                None,
                RootPinDropPolicy::RetainUntilChildrenDrain,
                encode_pin_payload(commit_id, &root_semantic_commitment, 0),
                pin_write()?,
            )
            .map_err(|_| corrupt())?
            .pin();
        if let Some(entry_arc) = next_catalog.bases.get_mut(&commit_id) {
            let mut updated = (**entry_arc).clone();
            updated.pin_ordinal = pin_id.ordinal();
            *entry_arc = Arc::new(updated);
        }
        // Atomically claim the planned branch out of in-flight retain intents
        // now that the durable pin and catalog basis entry are committed.
        next_catalog
            .inflight_retain_branches
            .retain(|name| name != &retained_branch_name);
        next_catalog
            .pending_retire_branches
            .retain(|name| name != &retained_branch_name);
        let previous_branch = state
            .catalog
            .bases
            .get(&commit_id)
            .and_then(|e| e.retained_branch_name.clone());
        publish_catalog(&self.inner.serving, &mut state, &next_catalog)?;
        state.catalog = next_catalog;
        state
            .retained_branches
            .decrement(previous_branch.as_deref());
        state
            .retained_branches
            .increment(Some(retained_branch_name.as_str()));
        state.needs_reconcile = false;
        Ok(RetainedRevisionReceipt {
            commit_id,
            root_semantic_commitment,
            pin_id,
            branch_origin,
            schema_version,
            retained_branch_name: Some(retained_branch_name),
        })
    }

    /// Opens an owner-verified Query window cursor backed by a real Store
    /// `PhysicalRootPinLoan`, Home-MAC-authenticated token, and upfront
    /// [`AdmittedQueryWindowBinding`] validated from page 1.
    #[allow(clippy::too_many_arguments)]
    pub fn open_admitted_query_cursor<'a, Row: KnownProjectedHeapBytes + 'a>(
        &'a self,
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

    /// Resumes an owner-verified Query window cursor from an opaque `whc1_...` wire token.
    #[allow(clippy::too_many_arguments)]
    pub fn resume_admitted_query_cursor_wire<'a, Row: KnownProjectedHeapBytes + 'a>(
        &'a self,
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

    /// Resumes an owner-verified Query window cursor after validating Home-MAC,
    /// schema, derived query digest, upfront [`AdmittedQueryWindowBinding`],
    /// and acquiring a real Store `PhysicalRootPinLoan`.
    #[allow(clippy::too_many_arguments)]
    pub fn resume_admitted_query_cursor<'a, Row: KnownProjectedHeapBytes + 'a>(
        &'a self,
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
        &'a self,
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
            StoreLoanCustody::Durable(pin_loan),
            active,
            Box::new(move |execution, after, limit| {
                window_stream.read_page(execution, after, limit)
            }),
        ))
    }
}
