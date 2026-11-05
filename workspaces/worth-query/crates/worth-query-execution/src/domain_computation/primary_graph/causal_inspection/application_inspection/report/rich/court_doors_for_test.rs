//! Sealed test doors over this module's private members.
use super::*;

impl FundedRichInspectionReport {
    #[doc(hidden)]
    pub fn body_for_test(&self) -> &PreparedRichMaterialization {
        &self.body
    }
}

impl FundedRichInspectionReport {
    #[doc(hidden)]
    pub fn reference_for_test(&self) -> &FundedInspectionReport {
        &self.reference
    }
}

impl FundedRichInspectionReport {
    #[doc(hidden)]
    pub fn body_mut_for_test(&mut self) -> &mut PreparedRichMaterialization {
        &mut self.body
    }
}

impl PreparedRichMaterialization {
    #[doc(hidden)]
    pub fn matches_for_test(&self, reference: &FundedInspectionReport) -> bool {
        Self::matches(self, reference)
    }
}

impl FundedRichInspectionReport {
    #[doc(hidden)]
    pub fn body_into_for_test(self) -> PreparedRichMaterialization {
        self.body
    }
}
