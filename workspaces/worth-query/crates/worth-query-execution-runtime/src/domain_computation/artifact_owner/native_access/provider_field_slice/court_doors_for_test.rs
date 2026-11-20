//! Sealed test doors over this module's private members.
use super::*;

impl<'a> WorthQueryArtifactProviderFieldSlice<'a> {
    #[doc(hidden)]
    pub fn physical_bytes_for_test(self) -> usize {
        Self::physical_bytes(self)
    }
}
