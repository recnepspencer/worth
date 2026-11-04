//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryArtifactProjectionSink {
    #[doc(hidden)]
    pub fn pending_row_for_test(&self) -> &Vec<AspectValue> {
        &self.pending_row
    }
}

impl WorthQueryArtifactProjectionSink {
    #[doc(hidden)]
    pub fn values_for_test(&self) -> &Vec<AspectValue> {
        &self.values
    }
}

impl WorthQueryArtifactProjectionSink {
    #[doc(hidden)]
    pub fn new_for_test(
        fields: Vec<AspectContract>,
        max_rows: usize,
        required_alignment: WorthQueryArtifactNativeAlignment,
    ) -> Result<Self, WorthQueryArtifactProviderAccessDenial> {
        Self::new(fields, max_rows, required_alignment)
    }
}
