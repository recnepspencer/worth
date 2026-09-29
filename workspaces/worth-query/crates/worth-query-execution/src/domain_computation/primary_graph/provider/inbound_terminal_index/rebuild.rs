//! Disposable compact-index reconstruction under World frontier protection.

use std::collections::BTreeMap;

use worth_runtime_world::facade::{RuntimeWorldPublicationCursor, RuntimeWorldPublicationFrontier};

use super::{
    TerminalRebuildState, WorthQueryCanonicalInboundCompletion, WorthQueryInboundTerminalIndex,
    WorthQueryInboundTerminalIndexDenial,
};

impl WorthQueryInboundTerminalIndex {
    pub(super) fn reconstruction_cursor(&self) -> (Option<RuntimeWorldPublicationCursor>, u64) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.rebuilding.is_none() {
            state.rebuilding = Some(TerminalRebuildState {
                generation: state.generation,
                frontier: None,
                cursor: None,
                entries: BTreeMap::new(),
            });
        }
        let rebuilding = state.rebuilding.as_ref().expect("rebuild initialized");
        (rebuilding.cursor.clone(), rebuilding.generation)
    }

    pub(super) fn advance_reconstruction(
        &self,
        generation: u64,
        frontier: &RuntimeWorldPublicationFrontier,
        next: Option<RuntimeWorldPublicationCursor>,
        rows: impl IntoIterator<Item = WorthQueryCanonicalInboundCompletion>,
    ) -> Result<bool, WorthQueryInboundTerminalIndexDenial> {
        use WorthQueryInboundTerminalIndexDenial as Denial;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.generation != generation {
            state.rebuilding = None;
            return Err(Denial::IndexUnavailable);
        }
        let rebuilding = state.rebuilding.as_mut().ok_or(Denial::IndexUnavailable)?;
        if rebuilding
            .frontier
            .as_ref()
            .is_some_and(|prior| prior != frontier)
        {
            state.rebuilding = None;
            return Err(Denial::IndexUnavailable);
        }
        rebuilding.frontier = Some(frontier.clone());
        for row in rows {
            if rebuilding.entries.insert(*row.correlation(), row).is_some() {
                state.rebuilding = None;
                return Err(Denial::WorldPairMismatch);
            }
        }
        rebuilding.cursor = next;
        if rebuilding.cursor.is_some() {
            return Ok(false);
        }
        let rebuilt = state
            .rebuilding
            .take()
            .expect("rebuild initialized")
            .entries;
        if rebuilt.len() != state.protected.len()
            || rebuilt.iter().any(|(correlation, row)| {
                state.protected.get(correlation).is_none_or(|protection| {
                    protection.commit_identity() != row.completion_world_commit()
                })
            })
        {
            return Err(Denial::IndexUnavailable);
        }
        state.derived = Some(rebuilt);
        Ok(true)
    }

    pub(super) fn abandon_reconstruction(&self) {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .rebuilding = None;
    }
}
