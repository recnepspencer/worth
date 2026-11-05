//! Sealed test doors over this module's private members.
use super::*;

impl<Schema: ApplicationSchema, Scope> InspectionRead<'_, '_, Schema, Scope> {
    #[doc(hidden)]
    pub fn check_for_test(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<WorthQuerySessionInspectionReadProof, InspectionAdmissionDenial> {
        Self::check(self, execution)
    }
}
