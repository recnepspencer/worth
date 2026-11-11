//! A refused constructor returns the actual unconsumed publication owner.
use super::*;
pub(in crate::domain_computation::primary_graph) struct RefusedConditionalRuntimeInstallation<
    Schema,
> {
    pub(super) publication: ApplicationRuntimePublication<Schema>,
    pub(super) cause: WorthQueryConditionalRuntimeInstallationDenial,
}
impl<Schema> RefusedConditionalRuntimeInstallation<Schema> {
    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (
        ApplicationRuntimePublication<Schema>,
        WorthQueryConditionalRuntimeInstallationDenial,
    ) {
        (self.publication, self.cause)
    }
}
