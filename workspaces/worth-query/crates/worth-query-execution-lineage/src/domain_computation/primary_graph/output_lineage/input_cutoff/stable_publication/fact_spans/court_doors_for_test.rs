//! Sealed test doors over this module's private members.
use super::*;

impl StableFactSpans {
    #[doc(hidden)]
    pub fn new_for_test(
        body: FactPayload,
        handler_count: usize,
        witness_count: usize,
    ) -> Option<Self> {
        Self::new(body, handler_count, witness_count)
    }
}
