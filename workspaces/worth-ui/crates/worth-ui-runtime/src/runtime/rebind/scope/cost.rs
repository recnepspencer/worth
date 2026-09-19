#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAffectedScopeCost {
    observations: usize,
    changed_facts: usize,
    affected_aspects: usize,
    indexed_consumers: usize,
    lookup_receipts: usize,
    index_probes: usize,
    contract_checks: usize,
    graph_and_mounted_entries: usize,
    theme_slots_compared: usize,
}

pub(crate) struct UiAffectedScopeCostInput {
    pub(crate) observations: usize,
    pub(crate) changed_facts: usize,
    pub(crate) affected_aspects: usize,
    pub(crate) indexed_consumers: usize,
    pub(crate) lookup_receipts: usize,
    pub(crate) index_probes: usize,
    pub(crate) contract_checks: usize,
    pub(crate) graph_and_mounted_entries: usize,
    pub(crate) theme_slots_compared: usize,
}

impl UiAffectedScopeCost {
    pub(crate) const fn exact(input: UiAffectedScopeCostInput) -> Self {
        Self {
            observations: input.observations,
            changed_facts: input.changed_facts,
            affected_aspects: input.affected_aspects,
            indexed_consumers: input.indexed_consumers,
            lookup_receipts: input.lookup_receipts,
            index_probes: input.index_probes,
            contract_checks: input.contract_checks,
            graph_and_mounted_entries: input.graph_and_mounted_entries,
            theme_slots_compared: input.theme_slots_compared,
        }
    }

    pub const fn observations(self) -> usize {
        self.observations
    }

    pub const fn theme_slots_compared(self) -> usize {
        self.theme_slots_compared
    }

    pub const fn changed_facts(self) -> usize {
        self.changed_facts
    }

    pub const fn affected_aspects(self) -> usize {
        self.affected_aspects
    }

    pub const fn indexed_consumers(self) -> usize {
        self.indexed_consumers
    }

    pub const fn lookup_receipts(self) -> usize {
        self.lookup_receipts
    }

    pub const fn index_probes(self) -> usize {
        self.index_probes
    }

    pub const fn contract_checks(self) -> usize {
        self.contract_checks
    }

    pub const fn graph_and_mounted_entries(self) -> usize {
        self.graph_and_mounted_entries
    }
}
