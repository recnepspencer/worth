//! Durable, bounded, newest-first correction history for restart-safe undo/redo.
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::num::NonZeroUsize;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_aftermath::ApplicationCorrectionBinding,
    application_program::ApplicationProgramDefinition, application_schema::ApplicationSchema,
};
use worth_query_execution::facade::primary_graph::WorthQueryCorrectionSourceCoordinates;
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationCorrectionHistoryDenial, WorthQueryApplicationRequestExt,
    WorthQueryCorrectionHistoryRecord, WorthQueryCorrectionHistoryState,
};

use super::{
    WorthQueryApplicationHome, WorthQueryCorrectionEntry, DEFAULT_CORRECTION_HISTORY_LIMITS,
};

/// How many correction history entries to list: at least one, at most
/// [`Self::MAXIMUM`]. An unbounded history scan is not expressible.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryCorrectionHistoryLimit(NonZeroUsize);

impl WorthQueryCorrectionHistoryLimit {
    /// The largest number of entries one listing may return.
    pub const MAXIMUM: usize = 256;

    /// `None` for zero or more than [`Self::MAXIMUM`] entries.
    pub const fn new(entries: usize) -> Option<Self> {
        match NonZeroUsize::new(entries) {
            Some(entries) if entries.get() <= Self::MAXIMUM => Some(Self(entries)),
            _ => None,
        }
    }

    pub const fn get(self) -> usize {
        self.0.get()
    }
}

/// One durable authored write on the Home's active branch.
///
/// Survives Home close and reopen: it is read from durable correction custody,
/// not from process memory. Pass [`Self::entry`] to
/// [`WorthQueryApplicationHome::undo_authored`] (when [`Self::is_applied`]) or
/// [`WorthQueryApplicationHome::redo_authored`] (when [`Self::is_reversed`]),
/// choosing the correction with [`Self::is_corrected_by`].
pub struct WorthQueryCorrectionHistoryEntry<Schema> {
    record: WorthQueryCorrectionHistoryRecord,
    entry: WorthQueryCorrectionEntry,
    _schema: PhantomData<fn() -> Schema>,
}

impl<Schema: ApplicationSchema> WorthQueryCorrectionHistoryEntry<Schema> {
    /// The handle an undo or redo of this write selects now.
    pub fn entry(&self) -> &WorthQueryCorrectionEntry {
        &self.entry
    }

    /// The original authored write's durable coordinates.
    pub fn original(&self) -> &WorthQueryCorrectionSourceCoordinates {
        self.record.original()
    }

    /// The original's durable commit sequence; larger is newer.
    pub fn sequence(&self) -> u64 {
        self.record.sequence()
    }

    pub fn state(&self) -> WorthQueryCorrectionHistoryState {
        self.record.state()
    }

    /// The write is in effect and may be undone.
    pub fn is_applied(&self) -> bool {
        matches!(
            self.state(),
            WorthQueryCorrectionHistoryState::Applied { .. }
        )
    }

    /// An undo reversed the write and it may be redone.
    pub fn is_reversed(&self) -> bool {
        matches!(
            self.state(),
            WorthQueryCorrectionHistoryState::Reversed { .. }
        )
    }

    /// Whether `C` is the declared correction for this write's action.
    pub fn is_corrected_by<C: ApplicationCorrectionBinding<Schema>>(&self) -> bool {
        self.record.source_binding_type() == std::any::TypeId::of::<C::Source>()
    }

    fn state_order(&self) -> u64 {
        match self.state() {
            WorthQueryCorrectionHistoryState::Applied { at }
            | WorthQueryCorrectionHistoryState::Reversed { at } => at,
        }
    }
}

impl<Schema> std::fmt::Debug for WorthQueryCorrectionHistoryEntry<Schema> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryCorrectionHistoryEntry")
            .field("sequence", &self.record.sequence())
            .field("state", &self.record.state())
            .finish_non_exhaustive()
    }
}

/// A bounded newest-first page of durable authored writes on the active branch.
pub struct WorthQueryCorrectionHistory<Schema> {
    entries: Vec<WorthQueryCorrectionHistoryEntry<Schema>>,
}

impl<Schema: ApplicationSchema> WorthQueryCorrectionHistory<Schema> {
    /// Listed writes, newest original first.
    pub fn entries(&self) -> &[WorthQueryCorrectionHistoryEntry<Schema>] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The top of the undo stack: the most recently applied write in effect.
    pub fn next_undo(&self) -> Option<&WorthQueryCorrectionHistoryEntry<Schema>> {
        self.entries
            .iter()
            .filter(|entry| entry.is_applied())
            .max_by_key(|entry| entry.state_order())
    }

    /// The top of the redo stack: the most recently undone write, unless a
    /// newer authored write has been made since that undo (which clears redo).
    pub fn next_redo(&self) -> Option<&WorthQueryCorrectionHistoryEntry<Schema>> {
        let undone = self
            .entries
            .iter()
            .filter(|entry| entry.is_reversed())
            .max_by_key(|entry| entry.state_order())?;
        let newer_write = self
            .entries
            .iter()
            .any(|entry| entry.is_applied() && entry.sequence() > undone.state_order());
        (!newer_write).then_some(undone)
    }
}

impl<Schema> std::fmt::Debug for WorthQueryCorrectionHistory<Schema> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(&self.entries).finish()
    }
}

impl<Schema, Program> WorthQueryApplicationHome<Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema> + 'static,
{
    /// Lists at most `limit` durable authored writes on the active branch that
    /// `principal` may correct, newest first, each with its reversal state.
    ///
    /// Requires a Directory Home. Writes on other branches, writes the
    /// principal cannot disclose, and writes whose retained history expired
    /// are omitted; undoing an entry rechecks all of this afresh and refuses
    /// a write changed since it was recorded.
    pub fn correction_history(
        &self,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
        limit: WorthQueryCorrectionHistoryLimit,
    ) -> Result<WorthQueryCorrectionHistory<Schema>, WorthQueryApplicationCorrectionHistoryDenial>
    {
        let principal = principal.borrow();
        let scope = WorthQueryRequestScope::new(
            None,
            WorthQueryCancellationSource::new().token(),
            principal.clock().clone(),
        );
        let records = self
            .primary_graph()
            .request(principal, &scope)
            .on_branch(self.active_branch())
            .corrections()
            .history(limit.0, DEFAULT_CORRECTION_HISTORY_LIMITS)?;
        Ok(WorthQueryCorrectionHistory {
            entries: records
                .into_iter()
                .map(|record| WorthQueryCorrectionHistoryEntry {
                    entry: WorthQueryCorrectionEntry::from_coordinates(record.handle().clone()),
                    record,
                    _schema: PhantomData,
                })
                .collect(),
        })
    }
}
