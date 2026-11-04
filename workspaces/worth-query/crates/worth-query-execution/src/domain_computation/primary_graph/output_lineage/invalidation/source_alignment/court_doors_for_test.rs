//! Sealed test doors over this module's private members.
use super::*;

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
