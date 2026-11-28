//! Compile-time lifetime-tied [`ReaderLoan`], [`BorrowedHistoricalRow`], and
//! [`HistoricalCursor`] over a retained historical basis.
//!
//! [`ReaderLoanGuard`] and [`HistoricalCursorStream`] are strictly sealed inside
//! `worth-store-application`, [`StoreLoanCustody`] is a concrete private Store
//! custody enum (never arbitrary `T` / `()`), and [`ReaderLoan`] /
//! [`HistoricalCursor`] have zero public constructors.

use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use worth_execution::ExecutionRequest;
use worth_relational::facade::history::CommitId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::storage::RelationalStorageSubject;
use worth_runtime_world::facade::{CanonicalCursorKey, HistoricalBasisRow, HistoricalReadDenial};
use worth_store::physical_runtime::stability::durable_pins::PhysicalRootPinLoan;

use super::authority::HistoricalCursorAuthority;
use super::memory_support::MemoryLoanCustody;
use super::token::RetainedBasisToken;

mod sealed {
    pub trait Sealed {}
}

/// Concrete Store-owned reclaim-fence custody carried inside [`ReaderLoan`].
///
/// External crates cannot construct or substitute arbitrary custody types
/// (such as `()`).
#[allow(dead_code)]
pub(super) enum StoreLoanCustody<'a> {
    Durable(PhysicalRootPinLoan<'a>),
    Memory(MemoryLoanCustody),
    #[cfg(feature = "certification-test-authority")]
    CertificationSkipLoan,
}

/// Sealed backend custody guard released immediately when [`ReaderLoan`] drops.
///
/// External crates cannot implement this trait.
pub trait ReaderLoanGuard: sealed::Sealed + Send + Sync {
    /// Whether this loan is currently protecting its root under the reclaim fence.
    fn is_active(&self) -> bool {
        true
    }
}

pub(super) struct OwnedLoanGuard<'a> {
    _custody: StoreLoanCustody<'a>,
    active: Arc<AtomicBool>,
}

impl<'a> OwnedLoanGuard<'a> {
    pub(super) fn new(custody: StoreLoanCustody<'a>, active: Arc<AtomicBool>) -> Self {
        Self {
            _custody: custody,
            active,
        }
    }
}

impl Drop for OwnedLoanGuard<'_> {
    fn drop(&mut self) {
        self.active.store(false, Ordering::Release);
    }
}

impl sealed::Sealed for OwnedLoanGuard<'_> {}

impl ReaderLoanGuard for OwnedLoanGuard<'_> {
    fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }
}

/// Reclaim-fenced in-process reader loan.
///
/// Has no public constructor: minted exclusively by `worth-store-application`.
pub struct ReaderLoan<'a> {
    guard: Box<dyn ReaderLoanGuard + 'a>,
    _owner: PhantomData<&'a ()>,
}

impl<'a> ReaderLoan<'a> {
    pub(super) fn from_guard(guard: Box<dyn ReaderLoanGuard + 'a>) -> Self {
        Self {
            guard,
            _owner: PhantomData,
        }
    }

    pub fn is_active(&self) -> bool {
        self.guard.is_active()
    }
}

impl std::fmt::Debug for ReaderLoan<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReaderLoan")
            .field("active", &self.guard.is_active())
            .finish()
    }
}

/// Borrowed view tied at compile time to the lifetime of its [`HistoricalCursor`]
/// and underlying [`ReaderLoan`]. Returning it past `drop(cursor)` fails to compile.
#[derive(Debug)]
pub struct BorrowedHistoricalRow<'loan, Row> {
    row: &'loan Row,
}

impl<'loan, Row> BorrowedHistoricalRow<'loan, Row> {
    pub const fn get(&self) -> &'loan Row {
        self.row
    }
}

/// One page yielded by [`HistoricalCursor::next_page`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoricalCursorPage<Row> {
    rows: Vec<Row>,
    has_more: bool,
    token: RetainedBasisToken,
}

impl<Row> HistoricalCursorPage<Row> {
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn into_rows(self) -> Vec<Row> {
        self.rows
    }

    pub const fn has_more(&self) -> bool {
        self.has_more
    }

    pub const fn checkpoint_token(&self) -> &RetainedBasisToken {
        &self.token
    }
}

/// Sealed backend page stream bound to one active [`ReaderLoan`].
///
/// External crates cannot implement this trait.
pub trait HistoricalCursorStream<Row>: sealed::Sealed + Send {
    /// Reads up to `limit` canonical rows strictly after `after`, charging all
    /// resident pages to the isolated `Historical` working-set pool.
    fn read_page(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        after: &CanonicalCursorKey,
        limit: usize,
    ) -> Result<(Vec<(CanonicalCursorKey, Row)>, bool), HistoricalReadDenial>;
}

type PageReadClosure<'a, Row> = Box<
    dyn FnMut(
            &ExecutionRequest<'_, '_>,
            &CanonicalCursorKey,
            usize,
        ) -> Result<(Vec<(CanonicalCursorKey, Row)>, bool), HistoricalReadDenial>
        + Send
        + 'a,
>;

pub(super) struct ClosureCursorStream<'a, Row> {
    read: PageReadClosure<'a, Row>,
}

impl<'a, Row> ClosureCursorStream<'a, Row> {
    pub(super) fn new(read: PageReadClosure<'a, Row>) -> Self {
        Self { read }
    }
}

impl<Row> sealed::Sealed for ClosureCursorStream<'_, Row> {}

impl<Row> HistoricalCursorStream<Row> for ClosureCursorStream<'_, Row> {
    fn read_page(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        after: &CanonicalCursorKey,
        limit: usize,
    ) -> Result<(Vec<(CanonicalCursorKey, Row)>, bool), HistoricalReadDenial> {
        (self.read)(execution, after, limit)
    }
}

/// Resumable historical cursor holding an atomic reclaim-fenced [`ReaderLoan`].
///
/// Has no public constructor: admitted exclusively by `worth-store-application`.
pub struct HistoricalCursor<'a, Row = HistoricalBasisRow> {
    loan: ReaderLoan<'a>,
    token: RetainedBasisToken,
    authority: HistoricalCursorAuthority,
    stream: Box<dyn HistoricalCursorStream<Row> + 'a>,
    last_row: Option<Row>,
    exhausted: bool,
}

impl<'a, Row> HistoricalCursor<'a, Row> {
    pub(super) fn from_owner(
        loan: ReaderLoan<'a>,
        token: RetainedBasisToken,
        authority: HistoricalCursorAuthority,
        stream: Box<dyn HistoricalCursorStream<Row> + 'a>,
    ) -> Self {
        Self {
            loan,
            token,
            authority,
            stream,
            last_row: None,
            exhausted: false,
        }
    }

    pub const fn commit_id(&self) -> CommitId {
        self.token.commit_id()
    }

    pub const fn root_semantic_commitment(&self) -> &[u8; 32] {
        self.token.root_semantic_commitment()
    }

    pub const fn canonical_key_position(&self) -> &CanonicalCursorKey {
        self.token.canonical_key_position()
    }

    pub fn loan_active(&self) -> bool {
        self.loan.is_active()
    }

    /// Returns a resumable opaque [`RetainedBasisToken`] capturing the exact
    /// retained basis and current canonical logical key position.
    pub fn checkpoint_token(&self) -> RetainedBasisToken {
        self.token.clone()
    }

    /// Borrows the most recently yielded row tied strictly to this cursor's
    /// [`ReaderLoan`] lifetime (`E0505` / `E0515` if returned past `drop(cursor)`).
    pub fn peek_borrowed_row(&self) -> Option<BorrowedHistoricalRow<'_, Row>> {
        self.last_row
            .as_ref()
            .map(|row| BorrowedHistoricalRow { row })
    }

    /// Reads the next bounded page of canonical rows without promoting history
    /// into the `Head` working set.
    ///
    /// Validates strict monotonic ordering across the ENTIRE candidate page
    /// before committing any advancement to `self.token`, `self.last_row`, or
    /// `self.exhausted`, so any mid-page order violation leaves cursor state
    /// completely unchanged.
    pub fn next_page(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        limit: usize,
    ) -> Result<HistoricalCursorPage<Row>, HistoricalReadDenial>
    where
        Row: Clone,
    {
        if execution.consult().is_err() {
            return Err(HistoricalReadDenial::RequestStopped);
        }
        if limit == 0 || self.exhausted {
            return Ok(HistoricalCursorPage {
                rows: Vec::new(),
                has_more: !self.exhausted,
                token: self.token.clone(),
            });
        }
        let (entries, has_more) =
            self.stream
                .read_page(execution, self.token.canonical_key_position(), limit)?;
        let mut candidate_token = self.token.clone();
        let mut rows = Vec::with_capacity(entries.len());
        for (key, row) in entries {
            candidate_token = self.authority.advance_token(&candidate_token, key).ok_or(
                HistoricalReadDenial::CorruptStorage {
                    subject: RelationalStorageSubject::Region(PartitionId(0)),
                },
            )?;
            rows.push(row);
        }
        self.token = candidate_token;
        if let Some(last) = rows.last() {
            self.last_row = Some(last.clone());
        }
        if !has_more {
            self.exhausted = true;
        }
        Ok(HistoricalCursorPage {
            rows,
            has_more,
            token: self.token.clone(),
        })
    }
}
