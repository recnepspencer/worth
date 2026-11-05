//! Sealed test doors over this module's private members.
use super::*;

impl<'storage, Schema: ApplicationSchema, Scope> InspectionRead<'_, 'storage, Schema, Scope> {
    #[doc(hidden)]
    pub fn terminal_for_test(
        self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<InspectionTerminal<'storage>, InspectionAdmissionDenial> {
        Self::terminal(self, execution)
    }
}

impl InspectionTerminal<'_> {
    #[doc(hidden)]
    pub fn complete_for_test(
        self,
        execution: &ExecutionRequest<'_, '_>,
        release: WorthQueryApplicationBasisReleaseReceipt,
    ) -> Result<CompletedInspection, InspectionAdmissionDenial> {
        Self::complete(self, execution, release)
    }
}
