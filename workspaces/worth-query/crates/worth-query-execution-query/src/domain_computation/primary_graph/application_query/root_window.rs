//! A keyset window over a one-shot query's ordered roots: the rows strictly
//! after a boundary in the query's total order (declared ordering, then root
//! identity), at most `width` of them, and the boundary for the next window.
//!
//! The window is a request mode of the ordinary one-shot read. Its rows obey
//! the same admission, disclosure and basis as the complete read; only the
//! roots materialized change. Selection and the ordering keys still cover
//! every candidate root, so each window costs work linear in the candidates
//! while materialization (and its result ceiling) covers only the window.
pub mod request_binding;

pub use request_binding::compute_admitted_request_binding_digest;

use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use sha2::{Digest, Sha256};
use worth_foundational::facade::AspectValue;
use worth_relational::facade::identity::{EntityId, VersionId};

const RESUME_ANCHOR_PROOF_DOMAIN: &[u8] = b"worth.query.root_window.recomputed_anchor.v1";

/// Computes the opaque 256-bit cryptographic proof that a root window genuinely
/// recomputed its resume boundary from `root` on `snapshot_version` for
/// `request_binding_digest` under the caller's per-page `anchor_nonce`.
pub fn compute_resume_anchor_proof(
    anchor_nonce: &[u8; 32],
    root: EntityId,
    snapshot_version: VersionId,
    request_binding_digest: &[u8; 32],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(RESUME_ANCHOR_PROOF_DOMAIN);
    hasher.update(anchor_nonce);
    hasher.update(root.partition_id.0.to_le_bytes());
    hasher.update(root.local_slot.0.to_le_bytes());
    hasher.update(root.generation.0.to_le_bytes());
    hasher.update(snapshot_version.0.to_le_bytes());
    hasher.update(request_binding_digest);
    hasher.finalize().into()
}

/// Internal start position for a root window read.
#[derive(Clone)]
pub(crate) enum RootWindowAfter {
    Boundary(WorthQueryApplicationRootWindowBoundary),
    RootEntity {
        root: EntityId,
        anchor_nonce: [u8; 32],
    },
}

/// One window request. Clones share the progress its read reports.
#[derive(Clone)]
pub struct WorthQueryApplicationRootWindow {
    after: Option<RootWindowAfter>,
    width: NonZeroUsize,
    max_projected_bytes: Option<usize>,
    progress: Arc<Mutex<Option<WorthQueryApplicationRootWindowProgress>>>,
    oversized_row_bytes: Arc<Mutex<Option<usize>>>,
}

impl std::fmt::Debug for WorthQueryApplicationRootWindow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryApplicationRootWindow")
            .field("has_after", &self.after.is_some())
            .field("width", &self.width)
            .field("max_projected_bytes", &self.max_projected_bytes)
            .finish()
    }
}

/// The position after the last root of a window: its ordering values (by
/// ordering slot) and root identity. Strictly opaque in memory: withheld
/// ordering values never leave `worth-query-execution-query` in any serialized
/// or encrypted form.
#[derive(Clone, PartialEq, Eq)]
pub struct WorthQueryApplicationRootWindowBoundary {
    pub(crate) ordering: Vec<(Arc<str>, AspectValue)>,
    pub(crate) root: EntityId,
}

/// What a completed window read found after its rows.
///
/// Sealed to `worth-query-execution-query`: only `read_prepared_root_rows` /
/// `select_root_window` can construct this progress receipt, binding the exact
/// installed query contract digest, full admitted request-binding digest
/// (query + parameters + scope + principal), pinned snapshot version, input
/// root anchor, returned row count, and exact last-row root entity identity.
#[derive(Clone)]
pub struct WorthQueryApplicationRootWindowProgress {
    contract_digest: [u8; 32],
    request_binding_digest: [u8; 32],
    snapshot_version: VersionId,
    returned_rows: usize,
    started_at_beginning: bool,
    resume_anchor_proof: Option<[u8; 32]>,
    last_root_entity: Option<EntityId>,
    next: Option<WorthQueryApplicationRootWindowBoundary>,
}

/// Owner-sealed execution output pairing completed projected rows with the
/// exact [`WorthQueryApplicationRootWindowProgress`] receipt and per-row
/// projected byte footprints from that same committed window read.
///
/// External crates cannot construct or re-pair rows with a mismatched receipt.
pub struct WorthQueryApplicationRootWindowPage<Row> {
    rows: Vec<Row>,
    progress: WorthQueryApplicationRootWindowProgress,
    row_projected_bytes: Vec<usize>,
}

impl<Row> WorthQueryApplicationRootWindowPage<Row> {
    pub(crate) fn from_executed(
        rows: Vec<Row>,
        progress: WorthQueryApplicationRootWindowProgress,
        row_projected_bytes: Vec<usize>,
    ) -> Self {
        debug_assert_eq!(rows.len(), progress.returned_rows());
        debug_assert_eq!(rows.len(), row_projected_bytes.len());
        Self {
            rows,
            progress,
            row_projected_bytes,
        }
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub const fn progress(&self) -> &WorthQueryApplicationRootWindowProgress {
        &self.progress
    }

    pub fn row_projected_bytes(&self) -> &[usize] {
        &self.row_projected_bytes
    }

    pub fn into_parts(
        self,
    ) -> (
        Vec<Row>,
        WorthQueryApplicationRootWindowProgress,
        Vec<usize>,
    ) {
        (self.rows, self.progress, self.row_projected_bytes)
    }
}

impl WorthQueryApplicationRootWindow {
    /// The first `width` roots in the query's order.
    pub fn first(width: NonZeroUsize) -> Self {
        Self::new(None, width)
    }

    /// The `width` roots that follow `after` in the query's order.
    pub fn after(after: WorthQueryApplicationRootWindowBoundary, width: NonZeroUsize) -> Self {
        Self::new(Some(RootWindowAfter::Boundary(after)), width)
    }

    /// The `width` roots that strictly follow `root` in the query's total order
    /// on an immutable retained snapshot, cryptographically binding the
    /// recomputed-anchor proof in [`WorthQueryApplicationRootWindowProgress`]
    /// to `anchor_nonce` so the receipt never exposes the decrypted `EntityId`.
    /// Fails closed if `root` is absent from the admitted candidate set.
    pub fn after_root_entity_bound(
        root: EntityId,
        anchor_nonce: [u8; 32],
        width: NonZeroUsize,
    ) -> Self {
        Self::new(
            Some(RootWindowAfter::RootEntity { root, anchor_nonce }),
            width,
        )
    }

    /// Bounds the cumulative projected byte footprint materialized for this
    /// window, refusing even a single row that exceeds `max_bytes` before
    /// decoding `QueryResult`.
    pub const fn with_max_projected_bytes(mut self, max_bytes: usize) -> Self {
        self.max_projected_bytes = Some(max_bytes);
        self
    }

    fn new(after: Option<RootWindowAfter>, width: NonZeroUsize) -> Self {
        Self {
            after,
            width,
            max_projected_bytes: None,
            progress: Arc::new(Mutex::new(None)),
            oversized_row_bytes: Arc::new(Mutex::new(None)),
        }
    }

    pub const fn width(&self) -> NonZeroUsize {
        self.width
    }

    pub const fn max_projected_bytes(&self) -> Option<usize> {
        self.max_projected_bytes
    }

    pub(crate) const fn after_position(&self) -> Option<&RootWindowAfter> {
        self.after.as_ref()
    }

    pub(crate) fn report_oversized_row(&self, row_bytes: usize) {
        *self
            .oversized_row_bytes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(row_bytes);
    }

    /// If a window read refused because its very first row exceeded
    /// `max_projected_bytes`, reports that exact projected row byte size.
    pub fn take_oversized_row_bytes(&self) -> Option<usize> {
        self.oversized_row_bytes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn report(
        &self,
        contract_digest: [u8; 32],
        request_binding_digest: [u8; 32],
        snapshot_version: VersionId,
        returned_rows: usize,
        started_at_beginning: bool,
        resume_anchor_proof: Option<[u8; 32]>,
        last_boundary: Option<WorthQueryApplicationRootWindowBoundary>,
        next: Option<WorthQueryApplicationRootWindowBoundary>,
    ) -> WorthQueryApplicationRootWindowProgress {
        let last_root_entity = last_boundary.as_ref().map(|boundary| boundary.root);
        let progress = WorthQueryApplicationRootWindowProgress {
            contract_digest,
            request_binding_digest,
            snapshot_version,
            returned_rows,
            started_at_beginning,
            resume_anchor_proof,
            last_root_entity,
            next,
        };
        *self
            .progress
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(progress.clone());
        progress
    }

    /// Updates a previously reported window progress after typed row projection
    /// truncates the window to fewer rows under `max_projected_bytes`.
    pub(crate) fn update_progress(&self, progress: WorthQueryApplicationRootWindowProgress) {
        *self
            .progress
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(progress);
    }

    /// The progress of the read this window was executed by; `None` until a
    /// window read completed (a refused read reports nothing).
    pub fn take_progress(&self) -> Option<WorthQueryApplicationRootWindowProgress> {
        self.progress
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
    }
}

impl WorthQueryApplicationRootWindowProgress {
    /// Whether roots remain after this window.
    pub const fn has_more(&self) -> bool {
        self.next.is_some()
    }

    /// Installed graph read contract digest executed by this window.
    pub const fn contract_digest(&self) -> &[u8; 32] {
        &self.contract_digest
    }

    /// Full admitted request-binding digest covering installed contract, query
    /// identity, canonical parameters, resolved scope, and external principal.
    pub const fn request_binding_digest(&self) -> &[u8; 32] {
        &self.request_binding_digest
    }

    /// Exact Relational snapshot version observed by this window.
    pub const fn snapshot_version(&self) -> VersionId {
        self.snapshot_version
    }

    /// Exact number of roots materialized for this window.
    pub const fn returned_rows(&self) -> usize {
        self.returned_rows
    }

    /// `true` strictly when this window started from page 1 (`First`), and
    /// `false` for both supplied boundaries and recomputed root-entity resumes.
    pub const fn started_at_beginning(&self) -> bool {
        self.started_at_beginning
    }

    /// Opaque 256-bit cryptographic proof emitted strictly when this window
    /// recomputed its resume boundary from `RootWindowAfter::RootEntity` on
    /// the observed snapshot (`None` for `First` and `SuppliedBoundary`).
    pub const fn resume_anchor_proof(&self) -> Option<[u8; 32]> {
        self.resume_anchor_proof
    }

    /// Root entity identity of the last row returned by this window, if non-empty.
    pub const fn last_root_entity(&self) -> Option<EntityId> {
        self.last_root_entity
    }

    /// Truncates this progress receipt to `kept_rows` after typed projection
    /// stops early under `max_projected_bytes`, setting `last_root_entity` and
    /// `next` to `kept_last_boundary`.
    pub(crate) fn truncate_to_projected_rows(
        &mut self,
        kept_rows: usize,
        kept_last_boundary: Option<WorthQueryApplicationRootWindowBoundary>,
    ) {
        self.returned_rows = kept_rows;
        self.last_root_entity = kept_last_boundary.as_ref().map(|b| b.root);
        self.next = kept_last_boundary;
    }

    /// The opaque in-memory boundary to resume after, when roots remain.
    pub fn into_next(self) -> Option<WorthQueryApplicationRootWindowBoundary> {
        self.next
    }
}
