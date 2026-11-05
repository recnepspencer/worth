//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::conditional_operation::clock_handle::ConditionalClockLease;

impl<Schema> WorthQueryConditionalOperationRegistry<Schema> {
    #[doc(hidden)]
    pub fn admit_clock_for_test(
        &self,
        identity: &str,
        lease: &Arc<ConditionalClockLease>,
    ) -> Option<WorthQueryConditionalOperationCell<Schema>> {
        Self::admit_clock(self, identity, lease)
    }
}

impl<Schema> WorthQueryConditionalOperationRegistry<Schema> {
    #[doc(hidden)]
    pub fn install_for_test(
        &mut self,
        operation: Box<dyn WorthQueryInstalledConditionalOperation<Schema>>,
    ) -> Result<(), ()> {
        Self::install(self, operation)
    }
}
