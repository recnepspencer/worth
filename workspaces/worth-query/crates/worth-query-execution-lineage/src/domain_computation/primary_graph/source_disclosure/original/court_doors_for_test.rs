//! Sealed test doors over this module's private members.
use super::*;

impl OriginalSourceDisclosure {
    #[doc(hidden)]
    pub fn query_for_test(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity {
        Self::query(self)
    }
}
