//! Sealed test doors over this module's private members.
use super::*;

impl ValidatedCheckpointSize {
    #[doc(hidden)]
    pub fn new_for_test(
        native_len: usize,
        accepted_outputs: &[WorthQueryAcceptedOutputCheckpointIdentity],
        policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
    ) -> Result<Self, WorthQueryCheckpointCaptureDenial> {
        Self::new(native_len, accepted_outputs, policy)
    }
}

impl ValidatedCheckpointSize {
    #[doc(hidden)]
    pub fn total_for_test(&self) -> usize {
        Self::total(self)
    }
}
