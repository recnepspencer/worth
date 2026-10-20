/// Optional host quotas for Bridge-owned conditional storage. These govern Bridge
/// representations; Signal and source owners retain their independent budgets.
/// None removes only the policy ceiling. Native checked reservations, backing
/// custody and final-drop refunds remain active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgeConditionalRetentionBudget {
    pub maximum_managed_clocks: Option<usize>,
    pub maximum_reserved_temporal_intents: Option<usize>,
    pub maximum_retained_definition_candidates: Option<usize>,
    pub maximum_retained_bytes: Option<u64>,
    pub maximum_preparation_visits: Option<usize>,
}

impl BridgeConditionalRetentionBudget {
    pub const fn development() -> Self {
        Self {
            maximum_managed_clocks: None,
            maximum_reserved_temporal_intents: None,
            maximum_retained_definition_candidates: None,
            maximum_retained_bytes: None,
            maximum_preparation_visits: None,
        }
    }

    pub(crate) const fn is_valid(self) -> bool {
        !matches!(self.maximum_managed_clocks, Some(0))
            && !matches!(self.maximum_reserved_temporal_intents, Some(0))
            && !matches!(self.maximum_retained_definition_candidates, Some(0))
            && !matches!(self.maximum_retained_bytes, Some(0))
            && !matches!(self.maximum_preparation_visits, Some(0))
    }
}
