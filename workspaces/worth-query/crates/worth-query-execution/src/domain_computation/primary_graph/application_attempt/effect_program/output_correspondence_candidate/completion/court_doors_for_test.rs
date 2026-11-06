//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputCorrespondenceCandidate {
    #[doc(hidden)]
    pub fn validate_effects_for_test(
        &self,
        effects: &[WorthQueryApplicationRealizedEffect],
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Self::validate_effects(self, effects)
    }
}
