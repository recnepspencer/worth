//! Bounded exact-delta history for mounted allocation projections.

use std::collections::VecDeque;

const MOUNTED_PROJECTION_DELTA_HISTORY_LIMIT: usize = 32;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct UiMountedAllocationProjectionJournal {
    entries: VecDeque<UiMountedAllocationProjectionJournalEntry>,
}

#[derive(Clone, Debug, PartialEq)]
struct UiMountedAllocationProjectionJournalEntry {
    predecessor_revision: u64,
    successor_revision: u64,
    changed_projection_graph_keys: Box<[crate::graph::UiGraphNodeIdentity]>,
}

#[derive(Clone, Debug)]
pub(crate) struct UiMountedAllocationProjectionSource {
    catalog: super::UiMountedAllocationProjectionCatalog,
    delta: UiMountedAllocationProjectionDelta,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum UiMountedAllocationProjectionDelta {
    Exact(UiMountedAllocationExactDelta),
    ReconstructionRequired,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiMountedAllocationExactDelta {
    changed_projection_graph_keys: Box<[crate::graph::UiGraphNodeIdentity]>,
    journal_entries_touched: usize,
}

impl UiMountedAllocationProjectionJournal {
    pub(super) fn record(
        &mut self,
        predecessor_revision: u64,
        successor_revision: u64,
        mut changed_projection_graph_keys: Vec<crate::graph::UiGraphNodeIdentity>,
    ) {
        debug_assert!(successor_revision >= predecessor_revision);
        if successor_revision <= predecessor_revision {
            return;
        }
        changed_projection_graph_keys.sort();
        changed_projection_graph_keys.dedup();
        self.entries
            .push_back(UiMountedAllocationProjectionJournalEntry {
                predecessor_revision,
                successor_revision,
                changed_projection_graph_keys: changed_projection_graph_keys.into_boxed_slice(),
            });
        while self.entries.len() > MOUNTED_PROJECTION_DELTA_HISTORY_LIMIT {
            self.entries.pop_front();
        }
    }

    pub(super) fn source(
        &self,
        catalog: super::UiMountedAllocationProjectionCatalog,
        predecessor_revision: Option<u64>,
        current_revision: u64,
    ) -> UiMountedAllocationProjectionSource {
        let Some(predecessor_revision) = predecessor_revision else {
            return UiMountedAllocationProjectionSource::reconstructive(catalog);
        };
        if predecessor_revision == current_revision {
            return UiMountedAllocationProjectionSource::exact(catalog, Vec::new(), 0);
        }
        let mut cursor = predecessor_revision;
        let mut changed = Vec::new();
        let mut touched = 0usize;
        for entry in &self.entries {
            if entry.predecessor_revision != cursor {
                continue;
            }
            touched += 1;
            changed.extend_from_slice(&entry.changed_projection_graph_keys);
            cursor = entry.successor_revision;
            if cursor == current_revision {
                changed.sort();
                changed.dedup();
                return UiMountedAllocationProjectionSource::exact(catalog, changed, touched);
            }
        }
        UiMountedAllocationProjectionSource::reconstructive(catalog)
    }
}

impl Default for UiMountedAllocationProjectionJournal {
    fn default() -> Self {
        Self {
            entries: VecDeque::with_capacity(MOUNTED_PROJECTION_DELTA_HISTORY_LIMIT),
        }
    }
}

impl UiMountedAllocationProjectionSource {
    pub(crate) fn for_replacement(catalog: super::UiMountedAllocationProjectionCatalog) -> Self {
        Self::exact(catalog, Vec::new(), 0)
    }

    pub(crate) fn preview_only() -> Self {
        Self::exact(Default::default(), Vec::new(), 0)
    }

    pub(crate) fn projection(
        &self,
        graph_node: crate::graph::UiGraphNodeIdentity,
    ) -> Result<
        Option<worth_ui_host_contract::UiMountedAllocationProjection>,
        super::UiMountedAllocationProjectionDenial,
    > {
        self.catalog.projection(graph_node)
    }

    pub(crate) fn viewport_bounds(
        &self,
        graph_node: crate::graph::UiGraphNodeIdentity,
    ) -> Result<
        Option<super::UiCommittedViewportGeometry>,
        super::UiMountedAllocationProjectionDenial,
    > {
        self.catalog.viewport_bounds(graph_node)
    }

    pub(crate) fn delta(&self) -> &UiMountedAllocationProjectionDelta {
        &self.delta
    }

    fn exact(
        catalog: super::UiMountedAllocationProjectionCatalog,
        changed_projection_graph_keys: Vec<crate::graph::UiGraphNodeIdentity>,
        journal_entries_touched: usize,
    ) -> Self {
        Self {
            catalog,
            delta: UiMountedAllocationProjectionDelta::Exact(UiMountedAllocationExactDelta {
                changed_projection_graph_keys: changed_projection_graph_keys.into_boxed_slice(),
                journal_entries_touched,
            }),
        }
    }

    fn reconstructive(catalog: super::UiMountedAllocationProjectionCatalog) -> Self {
        Self {
            catalog,
            delta: UiMountedAllocationProjectionDelta::ReconstructionRequired,
        }
    }
}

impl UiMountedAllocationExactDelta {
    pub(crate) fn changed_projection_graph_keys(&self) -> &[crate::graph::UiGraphNodeIdentity] {
        &self.changed_projection_graph_keys
    }

    pub(crate) fn journal_entries_touched(&self) -> usize {
        self.journal_entries_touched
    }
}

#[cfg(test)]
mod tests {
    use super::{
        UiMountedAllocationProjectionDelta, UiMountedAllocationProjectionJournal,
        MOUNTED_PROJECTION_DELTA_HISTORY_LIMIT,
    };

    #[test]
    fn exact_source_composes_retained_revision_chain_once_per_graph_node() {
        let mut journal = UiMountedAllocationProjectionJournal::default();
        journal.record(7, 8, vec![crate::graph::UiGraphNodeIdentity::new(10)]);
        journal.record(
            8,
            9,
            vec![
                crate::graph::UiGraphNodeIdentity::new(20),
                crate::graph::UiGraphNodeIdentity::new(10),
            ],
        );

        let source = journal.source(Default::default(), Some(7), 9);
        let UiMountedAllocationProjectionDelta::Exact(delta) = source.delta() else {
            panic!("retained contiguous revisions must produce an exact delta");
        };
        assert_eq!(delta.journal_entries_touched(), 2);
        assert_eq!(
            delta.changed_projection_graph_keys(),
            &[
                crate::graph::UiGraphNodeIdentity::new(10),
                crate::graph::UiGraphNodeIdentity::new(20),
            ]
        );
    }

    #[test]
    fn expired_predecessor_requires_reconstruction_instead_of_empty_delta() {
        let mut journal = UiMountedAllocationProjectionJournal::default();
        for predecessor in 0..=MOUNTED_PROJECTION_DELTA_HISTORY_LIMIT as u64 {
            journal.record(predecessor, predecessor + 1, Vec::new());
        }

        let source = journal.source(
            Default::default(),
            Some(0),
            MOUNTED_PROJECTION_DELTA_HISTORY_LIMIT as u64 + 1,
        );
        assert!(matches!(
            source.delta(),
            UiMountedAllocationProjectionDelta::ReconstructionRequired
        ));
    }
}
