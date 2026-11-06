//! Sealed test doors over this module's private members.
use super::*;

impl<Schema: ApplicationSchema, Scope> InspectionRead<'_, '_, Schema, Scope> {
    #[doc(hidden)]
    pub fn materialize_rich_for_test<Q, P, R, S>(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        query: &WorthQueryInstalledApplicationQuery<Schema, Q, P, R, S>,
    ) -> Result<FundedRichInspectionReport, InspectionAdmissionDenial> {
        Self::materialize_rich::<Q, P, R, S>(self, execution, query)
    }
}

impl PreparedRichMaterialization {
    #[doc(hidden)]
    pub fn artifact_for_test(&self) -> &WorthQueryApplicationRichMaterialization {
        &self.artifact
    }
}

impl PreparedRichMaterialization {
    #[doc(hidden)]
    pub fn artifact_mut_for_test(&mut self) -> &mut WorthQueryApplicationRichMaterialization {
        &mut self.artifact
    }
}
