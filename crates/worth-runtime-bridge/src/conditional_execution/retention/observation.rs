/// Point-in-time Bridge-owned conditional retention usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgeConditionalRetentionObservation {
    managed_clocks: usize,
    reserved_temporal_intents: usize,
    retained_definition_candidates: usize,
    retained_bytes: u64,
}

impl BridgeConditionalRetentionObservation {
    pub(super) const fn new(
        managed_clocks: usize,
        reserved_temporal_intents: usize,
        retained_definition_candidates: usize,
        retained_bytes: u64,
    ) -> Self {
        Self {
            managed_clocks,
            reserved_temporal_intents,
            retained_definition_candidates,
            retained_bytes,
        }
    }

    pub(in crate::conditional_execution) fn checked_sum(items: &[Self]) -> Option<Self> {
        let mut total = Self::new(0, 0, 0, 0);
        for item in items {
            total.managed_clocks = total.managed_clocks.checked_add(item.managed_clocks)?;
            total.reserved_temporal_intents = total
                .reserved_temporal_intents
                .checked_add(item.reserved_temporal_intents)?;
            total.retained_definition_candidates = total
                .retained_definition_candidates
                .checked_add(item.retained_definition_candidates)?;
            total.retained_bytes = total.retained_bytes.checked_add(item.retained_bytes)?;
        }
        Some(total)
    }
    pub const fn managed_clocks(self) -> usize {
        self.managed_clocks
    }

    pub const fn reserved_temporal_intents(self) -> usize {
        self.reserved_temporal_intents
    }

    pub const fn retained_definition_candidates(self) -> usize {
        self.retained_definition_candidates
    }

    pub const fn retained_bytes(self) -> u64 {
        self.retained_bytes
    }
}
