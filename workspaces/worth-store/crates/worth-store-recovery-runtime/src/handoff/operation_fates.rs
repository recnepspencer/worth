use worth_store_recovery_physics::{
    HistoricalConsumedOperationSet, ReconciledOperationFate, ReconciledOperationFates,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryOperationFateSet {
    reconciled: ReconciledOperationFates,
    historical_consumed: HistoricalConsumedOperationSet,
}

impl RecoveryOperationFateSet {
    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        self.reconciled
            .owned_heap_bytes()?
            .checked_add(self.historical_consumed.owned_heap_bytes()?)
    }

    pub(crate) const fn new(
        reconciled: ReconciledOperationFates,
        historical_consumed: HistoricalConsumedOperationSet,
    ) -> Self {
        Self {
            reconciled,
            historical_consumed,
        }
    }

    pub fn operations(&self) -> &[ReconciledOperationFate] {
        self.reconciled.operations()
    }

    pub const fn acknowledged_durable(&self) -> u64 {
        self.reconciled.acknowledged_durable()
    }

    pub const fn durable_unacknowledged(&self) -> u64 {
        self.reconciled.durable_unacknowledged()
    }

    pub const fn proven_no_effect(&self) -> u64 {
        self.reconciled.proven_no_effect()
    }

    pub const fn indeterminate(&self) -> u64 {
        self.reconciled.indeterminate() - self.historical_consumed.len() as u64
    }

    pub fn historical_consumed(&self) -> u64 {
        self.historical_consumed.len() as u64
    }

    pub fn historical_consuming_descriptor(&self, operation: [u8; 32]) -> Option<[u8; 32]> {
        self.historical_consumed.descriptor_for(operation)
    }
}
