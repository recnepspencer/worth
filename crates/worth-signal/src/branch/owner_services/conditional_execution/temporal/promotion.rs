use crate::data::retained_storage::SignalConditionalRetentionReservation;
use crate::data::temporal::{
    BoundedTemporalReadyPromotionSummary, ReadyTemporalWake, TemporalFrontierSnapshot,
};

/// Move-only custody for a bounded promotion result. Its representation remains
/// charged to the originating Signal owner even after the partition is retired.
#[derive(Debug)]
pub struct SignalConditionalTemporalPromotion {
    pub(super) summary: BoundedTemporalReadyPromotionSummary,
    pub(super) _custody: SignalConditionalRetentionReservation,
}

/// Read-only projection; it cannot extract or clone the retained result buffer.
pub struct SignalConditionalTemporalPromotionView<'a> {
    result: &'a SignalConditionalTemporalPromotion,
}

impl SignalConditionalTemporalPromotion {
    pub fn promotion(&self) -> SignalConditionalTemporalPromotionView<'_> {
        SignalConditionalTemporalPromotionView { result: self }
    }

    pub fn selection_limit(&self) -> u64 {
        self.summary.selection_limit()
    }

    pub fn due_work_remaining(&self) -> bool {
        self.summary.due_work_remaining()
    }
}

impl SignalConditionalTemporalPromotionView<'_> {
    pub fn frontier_before(&self) -> TemporalFrontierSnapshot {
        self.result.summary.promotion().frontier_before()
    }

    pub fn frontier_after(&self) -> TemporalFrontierSnapshot {
        self.result.summary.promotion().frontier_after()
    }

    pub fn ready_wakes(&self) -> &[ReadyTemporalWake] {
        self.result.summary.promotion().ready_wakes()
    }

    pub fn promoted_wake_count(&self) -> u64 {
        self.result.summary.promotion().promoted_wake_count()
    }

    pub fn broad_scan_denial_count_delta(&self) -> u64 {
        self.result
            .summary
            .promotion()
            .broad_scan_denial_count_delta()
    }
}
