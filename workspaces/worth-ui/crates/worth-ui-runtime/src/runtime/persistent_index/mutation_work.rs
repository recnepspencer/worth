/// Exact structural work performed by one persistent ordered-index mutation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiPersistentIndexMutationWork {
    key_probes: usize,
    /// Logical path copies: every node a persistent update rebuilds, which is
    /// the locality bound. Physically most are rewritten in place.
    node_copies: usize,
    /// Nodes actually allocated: new leaves, plus path nodes copied because a
    /// fork still shares them.
    node_allocations: usize,
}

impl UiPersistentIndexMutationWork {
    pub(crate) fn key_probes(self) -> usize {
        self.key_probes
    }

    pub(crate) fn node_copies(self) -> usize {
        self.node_copies
    }

    #[cfg(test)]
    pub(crate) fn node_allocations(self) -> usize {
        self.node_allocations
    }

    pub(super) fn record_key_probe(&mut self) {
        self.key_probes += 1;
    }

    pub(super) fn record_node_copy(&mut self, allocated: bool) {
        self.node_copies += 1;
        self.node_allocations += usize::from(allocated);
    }

    pub(crate) fn merge(&mut self, other: Self) -> Result<(), ()> {
        self.key_probes = self.key_probes.checked_add(other.key_probes).ok_or(())?;
        self.node_copies = self.node_copies.checked_add(other.node_copies).ok_or(())?;
        self.node_allocations = self
            .node_allocations
            .checked_add(other.node_allocations)
            .ok_or(())?;
        Ok(())
    }

    pub(crate) fn with_key_probes(key_probes: usize) -> Self {
        Self {
            key_probes,
            node_copies: 0,
            node_allocations: 0,
        }
    }
}
