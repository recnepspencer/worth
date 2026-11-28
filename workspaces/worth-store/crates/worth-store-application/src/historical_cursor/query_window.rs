//! Owner-verified Query window stream binding [`WorthQueryApplicationRootWindowPage`]
//! directly into [`super::cursor::HistoricalCursor`] (Store Milestone 4, Phase M4.4).
//!
//! Security & disclosure invariants enforced here:
//! 1. External callers CANNOT pair arbitrary `Vec<Row>` with a mismatched progress
//!    receipt: [`AdmittedQueryWindowPage`] is constructible ONLY from a sealed
//!    [`WorthQueryApplicationRootWindowPage`] minted inside `worth-query-execution`.
//! 2. Store receives the expected installed query contract digest, full admitted
//!    request-binding digest, and retained snapshot version BEFORE page 1 (and on
//!    resume) via [`AdmittedQueryWindowBinding`] and validates EVERY page strictly
//!    against all three — NEVER learning them from receipts.
//! 3. Withheld ordering values NEVER leave `worth-query-execution-query` at all.
//!    Store seals ONLY the fixed-size `(ordinal: u64, root: EntityId)` cursor
//!    position under its crate-private `HistoricalCursorAuthority` (`self.mac_key`
//!    + `aad(home_identity, commit_id, query_digest, request_binding_digest)`), so
//!    zero plaintext ordering values OR raw `EntityId` bytes appear in `whc1_...`,
//!    `canonical_key_position()`, or `Debug`.
//! 4. Every row type MUST implement [`KnownProjectedHeapBytes`] so BOTH derived
//!    nested rows (`Vec<Child>`) and handwritten projections enforce exact
//!    `streaming_window_bytes` accounting with zero silent `0` fallback.

use std::num::NonZeroUsize;

use worth_execution::ExecutionRequest;
use worth_foundational::facade::HomeIdentity;
use worth_query_execution::facade::primary_graph::{
    compute_resume_anchor_proof, KnownProjectedHeapBytes, WorthQueryApplicationRootWindow,
    WorthQueryApplicationRootWindowPage,
};
use worth_relational::facade::history::CommitId;
use worth_relational::facade::identity::{EntityId, PartitionId, VersionId};
use worth_relational::facade::storage::RelationalStorageSubject;
use worth_runtime_world::facade::{
    CanonicalCursorKey, CanonicalKeyDomain, HistoricalReadDenial, SchemaVersionIdentity,
};

use super::authority::{HistoricalCursorAuthority, SEALED_QUERY_POSITION_BYTES};

const MAX_WINDOW_PREALLOC: usize = 64;
const MAX_QUERY_WINDOW_ROWS: usize = 256;

/// Expected admitted query contract + full admitted request-binding digest
/// (query + canonical parameters + scope + principal) + retained snapshot
/// identity verified on EVERY page (including page 1 and resume) — never
/// learned from receipts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmittedQueryWindowBinding {
    pub contract_digest: [u8; 32],
    pub request_binding_digest: [u8; 32],
    pub snapshot_version: VersionId,
}

/// One owner-verified page yielded by an admitted Query window read.
///
/// Can ONLY be constructed from a sealed [`WorthQueryApplicationRootWindowPage`]
/// minted directly by `worth-query-execution`, binding the exact completed rows,
/// their measured projected byte footprints, and the committed progress receipt
/// together so callers cannot substitute or re-pair rows.
pub struct AdmittedQueryWindowPage<Row: KnownProjectedHeapBytes> {
    inner: WorthQueryApplicationRootWindowPage<Row>,
}

impl<Row: KnownProjectedHeapBytes> AdmittedQueryWindowPage<Row> {
    /// Wraps an execution-minted [`WorthQueryApplicationRootWindowPage`] after
    /// verifying row count and byte-measurement alignment.
    pub fn from_executed_page(
        page: WorthQueryApplicationRootWindowPage<Row>,
    ) -> Result<Self, HistoricalReadDenial> {
        if page.rows().len() != page.progress().returned_rows()
            || page.rows().len() != page.row_projected_bytes().len()
        {
            return Err(corrupt());
        }
        Ok(Self { inner: page })
    }
}

/// Sealed-page Query window reader implemented by Query Host over a pinned
/// retained branch observation.
///
/// Store constructs the opaque [`WorthQueryApplicationRootWindow`] internally
/// so external sources NEVER observe the decrypted anchor [`EntityId`]; they
/// can only forward `window` into governed/session execution.
pub trait AdmittedQueryWindowSource<Row: KnownProjectedHeapBytes>: Send {
    fn read_window(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        window: WorthQueryApplicationRootWindow,
    ) -> Result<AdmittedQueryWindowPage<Row>, HistoricalReadDenial>;
}

/// Store-owned stream adapter that validates every sealed Query window page
/// against the upfront [`AdmittedQueryWindowBinding`], enforces actual projected
/// typed-row byte ceilings via [`KnownProjectedHeapBytes`], and seals fixed-size
/// `(ordinal, root)` positions into strictly monotone opaque [`CanonicalCursorKey`]s.
pub(super) struct AdmittedQueryWindowStream<'a, Row: KnownProjectedHeapBytes> {
    source: Box<dyn AdmittedQueryWindowSource<Row> + 'a>,
    authority: HistoricalCursorAuthority,
    schema_version: SchemaVersionIdentity,
    expected_binding: AdmittedQueryWindowBinding,
    aad: Vec<u8>,
    streaming_window_bytes: usize,
    ordinal: u64,
    current_root: Option<EntityId>,
}

impl<'a, Row: KnownProjectedHeapBytes> AdmittedQueryWindowStream<'a, Row> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        source: Box<dyn AdmittedQueryWindowSource<Row> + 'a>,
        authority: HistoricalCursorAuthority,
        home_identity: HomeIdentity,
        commit_id: CommitId,
        query_digest: [u8; 32],
        schema_version: SchemaVersionIdentity,
        expected_binding: AdmittedQueryWindowBinding,
        initial_position: &CanonicalCursorKey,
        streaming_window_bytes: usize,
    ) -> Result<Self, HistoricalReadDenial> {
        let aad = build_position_aad(
            home_identity,
            commit_id,
            &query_digest,
            &expected_binding.request_binding_digest,
        );
        let (ordinal, current_root) =
            decode_sealed_query_position(initial_position, &authority, &aad)?;
        Ok(Self {
            source,
            authority,
            schema_version,
            expected_binding,
            aad,
            streaming_window_bytes: streaming_window_bytes.max(1),
            ordinal,
            current_root,
        })
    }

    pub(super) fn read_page(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        after: &CanonicalCursorKey,
        limit: usize,
    ) -> Result<(Vec<(CanonicalCursorKey, Row)>, bool), HistoricalReadDenial> {
        if execution.consult().is_err() {
            return Err(HistoricalReadDenial::RequestStopped);
        }
        let (expected_ordinal, expected_root) =
            decode_sealed_query_position(after, &self.authority, &self.aad)?;
        if expected_ordinal != self.ordinal || expected_root != self.current_root {
            return Err(corrupt());
        }
        let bounded_limit = limit.clamp(1, MAX_QUERY_WINDOW_ROWS);
        let width = NonZeroUsize::new(bounded_limit).expect("non-zero");

        let (window, expected_proof) = match self.current_root {
            Some(root) => {
                let anchor_nonce = self
                    .authority
                    .derive_resume_anchor_nonce(self.ordinal, &self.aad);
                let proof = compute_resume_anchor_proof(
                    &anchor_nonce,
                    root,
                    self.expected_binding.snapshot_version,
                    &self.expected_binding.request_binding_digest,
                );
                (
                    WorthQueryApplicationRootWindow::after_root_entity_bound(
                        root,
                        anchor_nonce,
                        width,
                    ),
                    Some(proof),
                )
            }
            None => (WorthQueryApplicationRootWindow::first(width), None),
        };
        let window = window.with_max_projected_bytes(self.streaming_window_bytes);
        let page = self.source.read_window(execution, window)?;
        let (rows, progress, row_projected_bytes) = page.inner.into_parts();

        // Validate EVERY receipt against upfront admitted query + retained basis
        // binding from page 1 and on resume (NEVER learn from receipts).
        if rows.len() != progress.returned_rows()
            || rows.len() != row_projected_bytes.len()
            || rows.len() > bounded_limit
        {
            return Err(corrupt());
        }
        if *progress.contract_digest() != self.expected_binding.contract_digest
            || *progress.request_binding_digest() != self.expected_binding.request_binding_digest
            || progress.snapshot_version() != self.expected_binding.snapshot_version
        {
            return Err(HistoricalReadDenial::UnsupportedHistoricalMeaning {
                schema_version: self.schema_version.clone(),
            });
        }
        let expected_at_beginning = self.current_root.is_none();
        if progress.started_at_beginning() != expected_at_beginning
            || progress.resume_anchor_proof() != expected_proof
        {
            return Err(corrupt());
        }

        // Enforce exact projected byte footprints (max of intermediate tree bytes
        // and final typed `size_of::<Row>() + row.known_heap_bytes()`) strictly
        // against `streaming_window_bytes` (even on row 1).
        let stack_bytes = std::mem::size_of::<Row>();
        let mut total_projected_bytes = 0usize;
        for (row, &measured_bytes) in rows.iter().zip(row_projected_bytes.iter()) {
            let typed_bytes = stack_bytes.saturating_add(row.known_heap_bytes());
            let row_bytes = measured_bytes.max(typed_bytes);
            total_projected_bytes = total_projected_bytes.saturating_add(row_bytes);
            if row_bytes > self.streaming_window_bytes
                || total_projected_bytes > self.streaming_window_bytes
            {
                return Err(HistoricalReadDenial::MemoryExhausted {
                    requested_bytes: total_projected_bytes.max(row_bytes) as u64,
                    allowance_bytes: self.streaming_window_bytes as u64,
                });
            }
        }

        let has_more = progress.has_more();
        if rows.is_empty() {
            if has_more {
                return Err(corrupt());
            }
            return Ok((Vec::new(), false));
        }

        let count = rows.len();
        let final_ordinal = self.ordinal.checked_add(count as u64).ok_or_else(corrupt)?;
        let final_root = progress.last_root_entity().ok_or_else(corrupt)?;
        let sealed_position =
            self.authority
                .seal_query_cursor_position(final_ordinal, final_root, &self.aad);
        let final_key = encode_query_cursor_position(final_ordinal, Some(&sealed_position))
            .ok_or_else(corrupt)?;
        let (rt_ord, rt_root) =
            decode_sealed_query_position(&final_key, &self.authority, &self.aad)?;
        if rt_ord != final_ordinal || rt_root != Some(final_root) {
            return Err(corrupt());
        }

        let mut out = Vec::with_capacity(count.min(MAX_WINDOW_PREALLOC));
        for (idx, row) in rows.into_iter().enumerate() {
            let next_ord = self
                .ordinal
                .checked_add((idx + 1) as u64)
                .ok_or_else(corrupt)?;
            let key = if idx + 1 == count {
                final_key.clone()
            } else {
                encode_query_cursor_position(next_ord, None).ok_or_else(corrupt)?
            };
            out.push((key, row));
        }

        self.ordinal = final_ordinal;
        self.current_root = Some(final_root);
        Ok((out, has_more))
    }
}

fn build_position_aad(
    home_identity: HomeIdentity,
    commit_id: CommitId,
    query_digest: &[u8; 32],
    request_binding_digest: &[u8; 32],
) -> Vec<u8> {
    let mut aad = Vec::with_capacity(16 + 8 + 32 + 32);
    aad.extend_from_slice(&home_identity.bytes());
    aad.extend_from_slice(&commit_id.0.to_le_bytes());
    aad.extend_from_slice(query_digest);
    aad.extend_from_slice(request_binding_digest);
    aad
}

fn encode_query_cursor_position(
    ordinal: u64,
    sealed_position: Option<&[u8]>,
) -> Option<CanonicalCursorKey> {
    let extra = sealed_position.map_or(0, <[u8]>::len);
    let mut logical_key = Vec::with_capacity(8 + extra);
    logical_key.extend_from_slice(&ordinal.to_be_bytes());
    if let Some(bytes) = sealed_position {
        logical_key.extend_from_slice(bytes);
    }
    CanonicalCursorKey::after(PartitionId(0), CanonicalKeyDomain::Entity, logical_key)
}

fn decode_sealed_query_position(
    key: &CanonicalCursorKey,
    authority: &HistoricalCursorAuthority,
    aad: &[u8],
) -> Result<(u64, Option<EntityId>), HistoricalReadDenial> {
    match key {
        CanonicalCursorKey::Start => Ok((0, None)),
        CanonicalCursorKey::After {
            partition_id,
            domain,
            logical_key,
        } => {
            if *partition_id != 0
                || *domain != CanonicalKeyDomain::Entity
                || logical_key.len() != 8 + SEALED_QUERY_POSITION_BYTES
            {
                return Err(corrupt());
            }
            let prefix_ordinal = u64::from_be_bytes(logical_key[..8].try_into().expect("8 bytes"));
            if prefix_ordinal == 0 {
                return Err(corrupt());
            }
            let sealed = &logical_key[8..];
            let (opened_ordinal, root) = authority
                .open_query_cursor_position(sealed, aad)
                .ok_or_else(corrupt)?;
            if opened_ordinal != prefix_ordinal {
                return Err(corrupt());
            }
            Ok((opened_ordinal, Some(root)))
        }
    }
}

fn corrupt() -> HistoricalReadDenial {
    HistoricalReadDenial::CorruptStorage {
        subject: RelationalStorageSubject::Region(PartitionId(0)),
    }
}
