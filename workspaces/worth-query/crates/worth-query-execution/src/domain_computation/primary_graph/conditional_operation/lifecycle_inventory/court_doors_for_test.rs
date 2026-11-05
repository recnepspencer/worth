//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryConditionalOperationLiveness {
    #[doc(hidden)]
    pub fn new_for_test(
        binding: Weak<()>,
        lease: Weak<crate::domain_computation::primary_graph::vocabulary::conditional_operation::clock_handle::ConditionalClockLease>,
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
