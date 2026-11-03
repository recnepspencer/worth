//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryObservedSourceMeaningRegistry {
    #[doc(hidden)]
    pub fn intern_for_test(
        &self, query: &[u8; 32], parameters: &[u8; 32], footprint: WorthQueryObservedSourceFootprint, selection: &WorthQueryApplicationBasisSelectionIdentity) -> Option<Arc<WorthQueryObservedSourceMeaning>> {
        Self::intern(self, query, parameters, footprint, selection)
    }
}
