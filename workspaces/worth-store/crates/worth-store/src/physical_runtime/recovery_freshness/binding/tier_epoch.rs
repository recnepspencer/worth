//! At most one tier-namespace activation can exist for a Store. The fixed
//! observation retains exact WAL intervals without a growing recovery buffer.

use worth_store_physical_format::{TierEpochActivationPhaseV1, TierEpochActivationV1};
use worth_store_wal::WalLsnRange;

use super::StoreRecoveryBindingSampleDenial;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreTierEpochActivationObservation {
    intent_range: WalLsnRange,
    intent: TierEpochActivationV1,
    completion_range: Option<WalLsnRange>,
}

impl StoreTierEpochActivationObservation {
    pub const fn intent_range(self) -> WalLsnRange {
        self.intent_range
    }
    pub const fn intent(self) -> TierEpochActivationV1 {
        self.intent
    }
    pub const fn completion_range(self) -> Option<WalLsnRange> {
        self.completion_range
    }
}

#[derive(Default)]
pub(super) struct TierEpochObservations {
    activation: Option<StoreTierEpochActivationObservation>,
}

impl TierEpochObservations {
    pub(super) fn observe(
        &mut self,
        range: WalLsnRange,
        record: TierEpochActivationV1,
        store: [u8; 16],
    ) -> Result<(), StoreRecoveryBindingSampleDenial> {
        if record.store() != store {
            return Err(StoreRecoveryBindingSampleDenial::InvalidWalMember);
        }
        match (record.phase(), self.activation.as_mut()) {
            (TierEpochActivationPhaseV1::Intent, None) => {
                self.activation = Some(StoreTierEpochActivationObservation {
                    intent_range: range,
                    intent: record,
                    completion_range: None,
                });
                Ok(())
            }
            (TierEpochActivationPhaseV1::Completed, Some(existing))
                if existing.completion_range.is_none()
                    && existing.intent.completed() == record
                    && existing.intent_range.end_exclusive().get() <= range.start().get() =>
            {
                existing.completion_range = Some(range);
                Ok(())
            }
            _ => Err(StoreRecoveryBindingSampleDenial::InvalidWalMember),
        }
    }

    pub(super) fn finish(self) -> Option<StoreTierEpochActivationObservation> {
        self.activation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_wal::LogSequenceNumber;

    fn range(start: u64) -> WalLsnRange {
        WalLsnRange::new(
            LogSequenceNumber::new(start),
            LogSequenceNumber::new(start + 1),
        )
        .unwrap()
    }

    fn intent() -> TierEpochActivationV1 {
        TierEpochActivationV1::intent(
            [1; 16], [2; 16], 3, [4; 32], [5; 32], 7, 4, [6; 32], 4096, 8,
        )
        .unwrap()
    }

    #[test]
    fn exact_intent_completion_pair_is_retained_with_both_lsn_ranges() {
        let mut observations = TierEpochObservations::default();
        observations.observe(range(10), intent(), [1; 16]).unwrap();
        observations
            .observe(range(20), intent().completed(), [1; 16])
            .unwrap();
        let selected = observations.finish().unwrap();
        assert_eq!(selected.intent_range(), range(10));
        assert_eq!(selected.completion_range(), Some(range(20)));
        assert_eq!(selected.intent(), intent());
    }

    #[test]
    fn orphan_or_duplicate_completion_is_rejected() {
        let mut observations = TierEpochObservations::default();
        assert!(observations
            .observe(range(10), intent().completed(), [1; 16])
            .is_err());
        observations.observe(range(10), intent(), [1; 16]).unwrap();
        assert!(observations.observe(range(20), intent(), [1; 16]).is_err());
        observations
            .observe(range(20), intent().completed(), [1; 16])
            .unwrap();
        assert!(observations
            .observe(range(30), intent().completed(), [1; 16])
            .is_err());
    }
}
