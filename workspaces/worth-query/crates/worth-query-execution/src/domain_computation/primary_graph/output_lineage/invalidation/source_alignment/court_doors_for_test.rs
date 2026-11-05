//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::mark_state::FullVerificationReason;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity;

impl<'selected> SnapshotAlignedMarkState<'selected> {
    #[doc(hidden)]
    pub fn currentness_for_test(
        &self,
        settlement: &RecordedSettlementIdentity,
    ) -> SettlementCurrentness<'_> {
        Self::currentness(self, settlement)
    }
}

impl<'selected> SnapshotAlignedMarkState<'selected> {
    #[doc(hidden)]
    pub fn observe_image_for_test(
        image: &CompanionBranchImage<BranchMarkRoot>,
        selected: &'selected PositionedRelationalSnapshot,
    ) -> Result<Self, FullVerificationReason> {
        Self::observe_image(image, selected)
    }
}

impl BranchMarkRoot {
    #[doc(hidden)]
    pub fn current_for_test(&self) -> &Arc<MarkState> {
        &self.current
    }
}

impl BranchMarkRoot {
    #[doc(hidden)]
    pub fn past_for_test(&self) -> &OrdMap<Option<PatchStreamPosition>, HistoricalMarkState> {
        &self.past
    }
}

impl HistoricalMarkState {
    #[doc(hidden)]
    pub fn commit_id_for_test(&self) -> &Option<CommitId> {
        &self.commit_id
    }
}

impl HistoricalMarkState {
    #[doc(hidden)]
    pub fn root_id_for_test(&self) -> &u64 {
        &self.root_id
    }
}
