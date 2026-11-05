//! Sealed test doors over this module's private members.
use super::*;

impl PreparedNativeOutputWitness {
    #[doc(hidden)]
    pub fn prepare_republication_for_test(
        correspondence: &WorthQueryApplicationOutputCorrespondence,
        layout: &WorthQueryPrimaryGraphLayout,
    ) -> Result<Option<Self>, CompanionPreflightStop> {
        Self::prepare_republication(correspondence, layout)
    }
}
