//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryConditionalOperationLiveness {
    #[doc(hidden)]
    pub fn new_for_test(
        binding: Weak<()>,
        lease: Weak<super::super::installation::ConditionalClockLease>,
        wakes: Vec<Weak<()>>,
        intents: Vec<Weak<()>>,
        attempts: Vec<Weak<()>>,
    ) -> Self {
        Self {
            binding,
            lease,
            wakes,
            intents,
            attempts,
        }
    }
}
