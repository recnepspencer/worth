//! Retained positions on authoritative parent edges, without retirement splicing.
use std::sync::Arc;

use crate::history::retention::CompositeHistoryProtectionObligation;
use crate::identity::CompositeCommitIdentity;

use super::support::{lock_state, validate_owner};
use super::traversal::{protect_installed, HistoryTraversalControl};
use super::{
    CompositeAncestorSelectionDenial as Denial, CompositeCommitParent, CompositeHistoryCatalog,
    CompositeRuntimeWorldCommit,
};

#[derive(Debug)]
pub(crate) struct RetainedExactHistoryPosition {
    commit: Arc<CompositeRuntimeWorldCommit>,
    _protection: CompositeHistoryProtectionObligation,
    catalog: CompositeHistoryCatalog,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl RetainedExactHistoryPosition {
    pub(crate) fn commit(&self) -> &CompositeRuntimeWorldCommit {
        &self.commit
    }
}

impl CompositeHistoryCatalog {
    pub(crate) fn retain_exact_history_position(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        selected: &CompositeCommitIdentity,
    ) -> Result<Arc<RetainedExactHistoryPosition>, Denial> {
        let memory = reserve_position(execution)?;
        let state = lock_state(&self.state);
        validate_owner(&state, selected.owner_identity())?;
        let entry = state
            .entries
            .get(selected)
            .and_then(|slot| slot.get())
            .ok_or(Denial::SourceUnavailable)?;
        Ok(Arc::new(RetainedExactHistoryPosition {
            commit: Arc::clone(&entry.commit),
            _protection: protect_installed(&state, selected)?,
            catalog: self.clone(),
            _memory: memory,
        }))
    }

    /// Current custody stays held while the next raw parent is found and
    /// protected under the same catalog lock. Missing history is never skipped.
    pub(crate) fn next_exact_history_position(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        current: &RetainedExactHistoryPosition,
    ) -> Result<Option<Arc<RetainedExactHistoryPosition>>, Denial> {
        HistoryTraversalControl::Request {
            execution,
            maximum: None,
        }
        .check_live()?;
        let state = lock_state(&self.state);
        validate_owner(&state, current.commit.identity().owner_identity())?;
        if !Arc::ptr_eq(&self.state, &current.catalog.state) {
            return Err(Denial::SourceUnavailable);
        }
        let CompositeCommitParent::Ordinary(parent) = current.commit.parent() else {
            return Ok(None);
        };
        let entry = state
            .entries
            .get(parent.commit())
            .and_then(|slot| slot.get())
            .ok_or(Denial::RetiredHistoryGap)?;
        let memory = reserve_position(execution)?;
        Ok(Some(Arc::new(RetainedExactHistoryPosition {
            commit: Arc::clone(&entry.commit),
            _protection: protect_installed(&state, parent.commit())?,
            catalog: self.clone(),
            _memory: memory,
        })))
    }
}

fn reserve_position(
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<Option<worth_execution::ExecutionMemoryReservation>, Denial> {
    HistoryTraversalControl::Request {
        execution,
        maximum: None,
    }
    .check_live()?;
    execution
        .allocation_policy()
        .reserve_memory(
            (std::mem::size_of::<RetainedExactHistoryPosition>() + 2 * std::mem::size_of::<usize>())
                as u64,
        )
        .map_err(Denial::Allocation)
}
