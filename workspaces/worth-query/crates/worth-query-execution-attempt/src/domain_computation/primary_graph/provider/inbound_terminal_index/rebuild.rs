//! Disposable compact-index reconstruction under World frontier protection.

use std::collections::BTreeMap;

use super::source::InboundTerminalSourcePage;

use super::{
    TerminalRebuildState, WorthQueryCanonicalInboundCompletion, WorthQueryInboundTerminalIndex,
    WorthQueryInboundTerminalIndexDenial,
};

impl WorthQueryInboundTerminalIndex {
    pub(super) fn reconstruction_page(
        &self,
        maximum: std::num::NonZeroUsize,
    ) -> Result<InboundTerminalSourcePage, WorthQueryInboundTerminalIndexDenial> {
        use std::ops::Bound::{Excluded, Unbounded};
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        if !state.pending.is_empty() {
            return Err(WorthQueryInboundTerminalIndexDenial::IndexUnavailable);
        }
        if state.rebuilding.is_none() {
            state.rebuilding = Some(TerminalRebuildState {
                generation: state.generation,
                cursor: None,
                entries: BTreeMap::new(),
            });
        }
        let rebuilding = state.rebuilding.as_ref().expect("rebuild initialized");
        let lower = rebuilding.cursor.map_or(Unbounded, Excluded);
        let mut remaining = state.protected.range((lower, Unbounded));
        let sources = remaining
            .by_ref()
            .take(maximum.get())
            .map(|(key, source)| (*key, std::sync::Arc::clone(source)))
            .collect::<Vec<_>>();
        let next = if remaining.next().is_some() {
            sources.last().map(|(key, _)| *key)
        } else {
            None
        };
        Ok(InboundTerminalSourcePage {
            generation: rebuilding.generation,
            sources,
            next,
        })
    }

    pub(super) fn advance_reconstruction(
        &self,
        generation: u64,
        next: Option<
            crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity,
        >,
        rows: impl IntoIterator<Item = WorthQueryCanonicalInboundCompletion>,
    ) -> Result<bool, WorthQueryInboundTerminalIndexDenial> {
        use WorthQueryInboundTerminalIndexDenial as Denial;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.generation != generation || !state.pending.is_empty() {
            state.rebuilding = None;
            return Err(Denial::IndexUnavailable);
        }
        let rebuilding = state.rebuilding.as_mut().ok_or(Denial::IndexUnavailable)?;
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
