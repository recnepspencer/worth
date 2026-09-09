/// Installed Bridge-owned conditional storage limits. These govern Bridge
/// representations; Signal and source owners retain their independent budgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgeConditionalRetentionBudget {
    pub maximum_managed_clocks: usize,
    pub maximum_reserved_temporal_intents: usize,
    pub maximum_retained_definition_candidates: usize,
    pub maximum_retained_bytes: u64,
    pub maximum_preparation_visits: usize,
}

impl BridgeConditionalRetentionBudget {
    pub const fn development() -> Self {
        Self {
            maximum_managed_clocks: 4_096,
            maximum_reserved_temporal_intents: 65_536,
            maximum_retained_definition_candidates: 4_096,
            maximum_retained_bytes: 512 * 1024 * 1024,
            maximum_preparation_visits: 8_000_000,
        }
    }

    pub(crate) const fn is_valid(self) -> bool {
        self.maximum_managed_clocks != 0
            && self.maximum_reserved_temporal_intents != 0
            && self.maximum_retained_definition_candidates != 0
            && self.maximum_retained_bytes != 0
            && self.maximum_preparation_visits != 0
    }
}
