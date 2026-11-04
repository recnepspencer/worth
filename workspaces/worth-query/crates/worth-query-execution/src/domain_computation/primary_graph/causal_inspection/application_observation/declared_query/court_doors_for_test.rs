//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationQueryObservation {
    #[doc(hidden)]
    pub fn source_for_test(&self) -> &WorthQueryObservedSource<()> {
        &self.source
    }
}
