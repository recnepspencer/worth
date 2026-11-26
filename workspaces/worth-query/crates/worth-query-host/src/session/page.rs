//! Paged typed reads: a collection larger than its query's result ceiling is
//! read in keyset pages, each at most the ceiling, so no collection size is a
//! hard refusal.
//!
//! Pages follow the query's total order (its declared ordering, then row
//! identity). A cursor names the position after a page's last row, so the next
//! page neither skips nor repeats a row of the same basis: read every page
//! through one [`WorthQuerySessionSnapshot`] for a consistent walk. Through
//! the session each page reads the current basis, so rows committed meanwhile
//! appear (or vanish) by their position.
//!
//! Cost: every page still selects and orders all candidate rows (reading each
//! one's ordering fields); only the page's rows are projected and returned.
use std::marker::PhantomData;
use std::num::NonZeroUsize;

use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationRootWindow, WorthQueryApplicationRootWindowBoundary,
};
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationReadObservation, WorthQueryApplicationRequestQueryDenial,
};

use super::{WorthQueryHostSession, WorthQuerySessionQuery};

/// The position after one page's last row, for the page that follows.
///
/// Opaque (it may carry ordering values the caller is not shown) and bound to
/// its query type. Valid with any session or snapshot of the same Home.
pub struct WorthQueryPageCursor<Intent> {
    boundary: WorthQueryApplicationRootWindowBoundary,
    _intent: PhantomData<fn() -> Intent>,
}

impl<Intent> Clone for WorthQueryPageCursor<Intent> {
    fn clone(&self) -> Self {
        Self {
            boundary: self.boundary.clone(),
            _intent: PhantomData,
        }
    }
}

impl<Intent> std::fmt::Debug for WorthQueryPageCursor<Intent> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("WorthQueryPageCursor(..)")
    }
}

/// One page of typed rows and the cursor for the next, if rows remain.
pub struct WorthQueryPage<Row, Intent> {
    rows: Vec<Row>,
    next_cursor: Option<WorthQueryPageCursor<Intent>>,
}

impl<Row: std::fmt::Debug, Intent> std::fmt::Debug for WorthQueryPage<Row, Intent> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryPage")
            .field("rows", &self.rows)
            .field("has_more", &self.has_more())
            .finish()
    }
}

impl<Row, Intent> WorthQueryPage<Row, Intent> {
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// Whether rows remain after this page.
    pub const fn has_more(&self) -> bool {
        self.next_cursor.is_some()
    }

    /// The cursor to pass as `after` for the next page; `None` at the end.
    pub const fn next_cursor(&self) -> Option<&WorthQueryPageCursor<Intent>> {
        self.next_cursor.as_ref()
    }

    pub fn into_rows(self) -> Vec<Row> {
        self.rows
    }

    pub fn into_parts(self) -> (Vec<Row>, Option<WorthQueryPageCursor<Intent>>) {
        (self.rows, self.next_cursor)
    }
}

impl<Schema: ApplicationSchema, Program> WorthQueryHostSession<'_, Schema, Program> {
    /// Reads one page of at most `width` rows of `intent`, after `after` (or
    /// from the start), at the branch's current basis.
    ///
    /// `width` is bounded by the query's result ceiling (and this session's
    /// result limit); the collection is not. A query that cannot page (not
    /// `Many`, a managed-index selection, or another read shape) refuses
    /// `RequestMode`; [`Self::query`] is unchanged.
    pub fn query_page<Intent: WorthQuerySessionQuery<Schema>>(
        &self,
        intent: Intent,
        width: NonZeroUsize,
        after: Option<&WorthQueryPageCursor<Intent>>,
    ) -> Result<WorthQueryPage<Intent::Row, Intent>, WorthQueryApplicationRequestQueryDenial> {
        read_page(self, None, intent, width, after)
    }
}

/// One window read through `session`, at `at` when a snapshot pins it.
pub(super) fn read_page<Schema, Program, Intent>(
    session: &WorthQueryHostSession<'_, Schema, Program>,
    at: Option<&WorthQueryApplicationReadObservation>,
    intent: Intent,
    width: NonZeroUsize,
    after: Option<&WorthQueryPageCursor<Intent>>,
) -> Result<WorthQueryPage<Intent::Row, Intent>, WorthQueryApplicationRequestQueryDenial>
where
    Schema: ApplicationSchema,
    Intent: WorthQuerySessionQuery<Schema>,
{
    let window = match after {
        Some(cursor) => WorthQueryApplicationRootWindow::after(cursor.boundary.clone(), width),
        None => WorthQueryApplicationRootWindow::first(width),
    };
    let rows = intent
        .execute_window_in_session(session, at, window.clone())?
        .into_rows();
    // A kernel that did not page reports no progress: never present its
    // complete rows as a page.
    let progress = window
        .take_progress()
        .ok_or(WorthQueryApplicationRequestQueryDenial::RequestMode)?;
    Ok(WorthQueryPage {
        rows,
        next_cursor: progress.into_next().map(|boundary| WorthQueryPageCursor {
            boundary,
            _intent: PhantomData,
        }),
    })
}
