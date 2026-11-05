//! Sealed test doors over this module's private members.
use super::*;

impl PreparedOutputLineageSlot {
    #[doc(hidden)]
    pub fn into_recovery_metadata_for_test(self) -> PreparedLineageRecoveryMetadata {
        Self::into_recovery_metadata(self)
    }
}

impl PreparedOutputLineageSlot {
    #[doc(hidden)]
    pub fn retain_recovery_metadata_for_test(&mut self, metadata: PreparedLineageRecoveryMetadata) {
        Self::retain_recovery_metadata(self, metadata)
    }
}
