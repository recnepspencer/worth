use super::RootProtectionRegistry;
use crate::physical_runtime::durability::{ProtectedReachSet, ReachWindow};

impl RootProtectionRegistry {
    /// Projects every in-process registration and durable root pin that
    /// reaches `window` into `reach`, through the same counted indexes the
    /// boolean protection queries use, so projection stays bounded by the
    /// retained-root limit.
    pub(in crate::physical_runtime) fn project_reach(
        &self,
        window: ReachWindow,
        reach: &mut ProtectedReachSet,
    ) {
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            match window {
                ReachWindow::Root {
                    generation,
                    inline_tail,
                } => {
                    state.index_probes = state.index_probes.saturating_add(2);
                    let (visited, protected) = state.roots.contains(&generation);
                    let (tail_visited, tail) = state.inline_tails.consult(&inline_tail);
                    let tail = tail.is_some_and(|roots| *roots > 0);
                    state.examined_entries = state
                        .examined_entries
                        .saturating_add(visited)
                        .saturating_add(tail_visited);
                    if protected {
                        reach.protect_root(generation);
                    }
                    if tail {
                        reach.protect_inline_tail(inline_tail.0, inline_tail.1);
                    }
                }
                ReachWindow::RootsAtOrBelow(generation) => {
                    state.index_probes = state.index_probes.saturating_add(1);
                    let (visited, witness) = state.roots.find_key(|root| *root <= generation);
                    state.examined_entries = state.examined_entries.saturating_add(visited);
                    if let Some(root) = witness {
                        reach.protect_root(root);
                    }
                }
            }
        }
        // Pins are a leaf lock, consulted after the registration lock.
        self.pins.project_reach(window, reach);
    }
}
