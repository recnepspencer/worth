//! A body stamping admission time reads the operation's one trusted instant
//! through the same request-context read a handler makes, so the decision is
//! never reused and an unavailable time source refuses the operation.
use super::*;
use worth_foundational::facade::CanonicalTimestamp;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyCreatedField, ApplicationOperationBodyProgram,
};
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> DecisionReader<'_, '_, '_, S, B> {
    /// The advancement's settled admission instant when any creation site
    /// stamps it; `None` (and no request-context mark) otherwise.
    pub(crate) fn body_admitted_at(
        &self,
        body: &ApplicationOperationBodyProgram,
    ) -> Result<Option<CanonicalTimestamp>, HandlerExecutionDenial> {
        let stamps = body
            .creation()
            .into_iter()
            .flat_map(|root| root.initialization_sites())
            .flat_map(|site| site.fields())
            .any(ApplicationOperationBodyCreatedField::is_admission_time);
        if !stamps {
            return Ok(None);
        }
        self.admitted_at()
            .map(Some)
            .map_err(HandlerExecutionDenial::new)
    }
}
