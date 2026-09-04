#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiMountedAppearanceMembershipWork {
    key_probes: usize,
    copied_avl_nodes: usize,
    traversed_entries: usize,
}

impl UiMountedAppearanceMembershipWork {
    pub(crate) fn add_lookup(&mut self, probes: usize) {
        self.key_probes = self.key_probes.saturating_add(probes);
    }

    pub(crate) fn add_mutation(
        &mut self,
        work: crate::runtime::persistent_index::UiPersistentIndexMutationWork,
    ) {
        self.key_probes = self.key_probes.saturating_add(work.key_probes());
        self.copied_avl_nodes = self.copied_avl_nodes.saturating_add(work.node_copies());
    }

    pub(crate) fn add_traversal(&mut self, entries: usize) {
        self.traversed_entries = self.traversed_entries.saturating_add(entries);
    }

    pub(crate) fn merge(&mut self, other: Self) {
        self.key_probes = self.key_probes.saturating_add(other.key_probes);
        self.copied_avl_nodes = self.copied_avl_nodes.saturating_add(other.copied_avl_nodes);
        self.traversed_entries = self
            .traversed_entries
            .saturating_add(other.traversed_entries);
    }

    pub(crate) const fn key_probes(self) -> usize {
        self.key_probes
    }

    pub(crate) const fn copied_avl_nodes(self) -> usize {
        self.copied_avl_nodes
    }

    pub(crate) const fn traversed_entries(self) -> usize {
        self.traversed_entries
    }
}
