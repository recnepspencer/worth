//! Ancestor proof through retained owner parent edges, including retirement splices.
use std::num::NonZeroUsize;
use std::sync::Arc;

use crate::history::retention::CompositeHistoryProtectionObligation;
use crate::identity::CompositeCommitIdentity;

use super::support::{effective_parent, lock_state, validate_owner};
use super::traversal::{protect_installed, HistoryTraversalControl};
use super::{CompositeHistoryCatalog, CompositeHistoryCatalogDenial, CompositeRuntimeWorldCommit};

/// Historical occurrence protection; it grants no mutation authority.
#[derive(Debug)]
pub struct RetainedCompositeAncestor {
    source: Arc<CompositeRuntimeWorldCommit>,
    target: CompositeCommitIdentity,
    parent_links_followed: usize,
    _protection: CompositeHistoryProtectionObligation,
    _catalog: CompositeHistoryCatalog,
}

impl RetainedCompositeAncestor {
    pub fn source(&self) -> &CompositeRuntimeWorldCommit {
        &self.source
    }

    pub fn target(&self) -> &CompositeCommitIdentity {
        &self.target
    }

    /// Retained parent edges traversed from the selected target to this source.
    /// A retirement splice is one edge; its retired intermediate contents are
    /// not selected or disclosed.
    /// The source and target admission lookups are separate from this count.
    pub const fn parent_links_followed(&self) -> usize {
        self.parent_links_followed
    }
}

/// Why a retained source could not be established on the target's ancestry.
/// `NotAncestor` establishes a different lineage; `SelectionLimitReached`
/// means the requested work ended before that question was resolved.
/// Unavailable endpoints or unproven parent gaps refuse. Owner-recorded
/// retirement splices prove ancestry without granting retired contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositeAncestorSelectionDenial {
    SourceUnavailable,
    TargetUnavailable,
    NotAncestor,
    SelectionLimitReached,
    RetiredHistoryGap,
    Allocation(worth_execution::ExecutionAllocationDenial),
    Request(worth_execution::WorkCeilingDenial),
    Catalog(CompositeHistoryCatalogDenial),
}

impl From<CompositeHistoryCatalogDenial> for CompositeAncestorSelectionDenial {
    fn from(denial: CompositeHistoryCatalogDenial) -> Self {
        Self::Catalog(denial)
    }
}

impl CompositeHistoryCatalog {
    /// O(retained parent edges from target to source or root) time, O(1) space.
    /// Owner-recorded splices prove ancestry across retired intermediates. This
    /// does not grant their contents; exact history pages still follow raw parents.
    pub(crate) fn select_retained_ancestor(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &CompositeCommitIdentity,
        source: &CompositeCommitIdentity,
        maximum: Option<NonZeroUsize>,
    ) -> Result<RetainedCompositeAncestor, CompositeAncestorSelectionDenial> {
        let control = HistoryTraversalControl::Request {
            execution: *execution,
            maximum,
        };
        control.check_live()?;
        let state = lock_state(&self.state);
        validate_owner(&state, target.owner_identity())?;
        validate_owner(&state, source.owner_identity())?;
        let source_entry = state
            .entries
            .get(source)
            .and_then(|slot| slot.get())
            .ok_or(CompositeAncestorSelectionDenial::SourceUnavailable)?;
        if state
            .entries
            .get(target)
            .and_then(|slot| slot.get())
            .is_none()
        {
            return Err(CompositeAncestorSelectionDenial::TargetUnavailable);
        }
        let mut current = target.clone();
        let mut visited = 0usize;
        loop {
            control.check_live()?;
            if maximum.is_some_and(|limit| visited >= limit.get()) {
                return Err(CompositeAncestorSelectionDenial::SelectionLimitReached);
            }
            let entry = state
                .entries
                .get(&current)
                .and_then(|slot| slot.get())
                .ok_or(CompositeAncestorSelectionDenial::RetiredHistoryGap)?;
            if &current == source {
                return Ok(RetainedCompositeAncestor {
                    source: Arc::clone(&source_entry.commit),
                    target: target.clone(),
                    parent_links_followed: visited,
                    _protection: protect_installed(&state, source)?,
                    _catalog: self.clone(),
                });
            }
            visited = visited
                .checked_add(1)
                .ok_or(CompositeHistoryCatalogDenial::ArithmeticOverflow)?;
            current = match effective_parent(&state, entry.commit()) {
                Some((parent, _)) => parent,
                None => return Err(CompositeAncestorSelectionDenial::NotAncestor),
            };
        }
    }
}
