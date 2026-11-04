//! Sealed test doors over this module's private members.
use super::*;

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn native_marking_report_for_test(
        &self,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<Option<NativeMarkingReport>, CompanionPreflightStop> {
        Self::native_marking_report(self, selected)
    }
}
