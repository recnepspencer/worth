//! Sealed test doors over this module's private members.
use super::*;

impl SealedNativeOutputWitness {
    #[doc(hidden)]
    pub fn same_output_as_for_test(&self, other: &Self) -> Result<bool, CompanionPreflightStop> {
        Self::same_output_as(self, other)
    }
}
