//! A routed input forks only its resolved owner chain. Catalogs, pending layout,
//! and unrelated owners remain with the installed state, never the candidate.
use super::UiScrollRuntimeState;
use crate::runtime::scroll::{UiScrollChainEntry, UiScrollRouteDenial};

pub(crate) struct UiScrollRouteCandidate {
    state: UiScrollRuntimeState,
    predecessor_revision: u64,
}

impl UiScrollRouteCandidate {
    pub(crate) fn state(&self) -> &UiScrollRuntimeState {
        &self.state
    }

    pub(crate) fn state_mut(&mut self) -> &mut UiScrollRuntimeState {
        &mut self.state
    }
}

impl UiScrollRuntimeState {
    pub(crate) fn route_candidate(
        &self,
        chain: &[UiScrollChainEntry],
        direct: bool,
    ) -> Result<UiScrollRouteCandidate, UiScrollRouteDenial> {
        let mut state = Self::new_session_restore_candidate_with_policy(self.policy);
        state.counters = self.counters;
        state.revision = self.revision;
        state.last_owner = self.last_owner;
        for entry in chain {
            let accepted = *self.exact_owner(entry.owner(), entry.incarnation())?;
            let record = if direct {
                self.pending_direct
                    .get(&entry.owner())
                    .filter(|prepared| prepared.record.incarnation == entry.incarnation())
                    .map_or(accepted, |prepared| prepared.record)
            } else {
                accepted
            };
            state.owners.insert(entry.owner(), record);
            state
                .transition_targets
                .copy_owner_from(&self.transition_targets, entry.owner());
        }
        Ok(UiScrollRouteCandidate {
            state,
            predecessor_revision: self.revision,
        })
    }

    /// The synchronous settle publication accepted intent, not sampled pixels.
    /// Merge the exact fork only; a small candidate may never replace the session.
    pub(crate) fn commit_routed_candidate(&mut self, candidate: UiScrollRouteCandidate) {
        assert_eq!(self.revision, candidate.predecessor_revision);
        let candidate = candidate.state;
        for (owner, record) in candidate.owners {
            self.owners.insert(owner, record);
            self.transition_targets
                .copy_owner_from(&candidate.transition_targets, owner);
        }
        self.counters = candidate.counters;
        self.revision = candidate.revision;
        self.last_owner = candidate.last_owner;
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
std::thread_local! {
    static STATE_CLONES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Observe the derived full-state Clone itself, so a future clone-and-filter
/// implementation cannot pass a locality test merely by returning few rows.
#[cfg(test)]
pub(super) struct UiScrollStateCloneObservation;

#[cfg(test)]
impl Clone for UiScrollStateCloneObservation {
    fn clone(&self) -> Self {
        STATE_CLONES.with(|count| count.set(count.get() + 1));
        Self
    }
}
