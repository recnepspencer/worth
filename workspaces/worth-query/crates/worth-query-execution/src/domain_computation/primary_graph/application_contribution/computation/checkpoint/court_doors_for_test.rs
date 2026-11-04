//! Sealed test doors over this module's private members.
use super::*;

impl<'request> WorthQueryManagedComputationCheckpoint<'request> {
    #[doc(hidden)]
    pub fn charge_for_test(
        &self,
    ) -> &Option<&'request mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>> {
        &self.charge
    }
}
