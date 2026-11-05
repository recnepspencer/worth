//! Sealed test doors over this module's private members.
use super::*;

impl<Schema: ApplicationSchema, Scope> InspectionRead<'_, '_, Schema, Scope> {
    #[doc(hidden)]
    pub fn materialize_for_test(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<FundedInspectionReport, InspectionAdmissionDenial> {
        Self::materialize(self, execution)
    }
}
