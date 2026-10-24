//! Exact ancestor selection through the retained owner parent chain.
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

    /// Parent links traversed from the selected target to this source.
    /// The source and target admission lookups are separate from this count.
    pub const fn parent_links_followed(&self) -> usize {
        self.parent_links_followed
    }
}

/// Why a retained source could not be established on the target's ancestry.
/// `NotAncestor` establishes a different lineage; `SelectionLimitReached`
/// means the requested work ended before that question was resolved.
/// Unavailable entries and retired gaps require retained history to be present
/// before selection can succeed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositeAncestorSelectionDenial {
    SourceUnavailable,
    TargetUnavailable,
    NotAncestor,
    SelectionLimitReached,
    RetiredHistoryGap,
    Catalog(CompositeHistoryCatalogDenial),
}

impl From<CompositeHistoryCatalogDenial> for CompositeAncestorSelectionDenial {
    fn from(denial: CompositeHistoryCatalogDenial) -> Self {
        Self::Catalog(denial)
    }
}

impl CompositeHistoryCatalog {
    /// O(parent links from target to source or retained root) time, O(1) space.
    /// Exact catalog lookups follow only this ancestry, including fork parents.
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
            current = effective_parent(&state, entry.commit())
                .map(|(parent, _)| parent)
                .ok_or(CompositeAncestorSelectionDenial::NotAncestor)?;
        }
    }
}
