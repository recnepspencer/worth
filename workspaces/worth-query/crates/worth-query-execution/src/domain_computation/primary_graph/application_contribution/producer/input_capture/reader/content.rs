//! Content remains an admitted recorded read of the original capture request.
use super::*;
use crate::domain_computation::primary_graph::AdmittedContent;
use worth_query_declaration::facade::application_schema::{
    fragment::RecordSelection, ApplicationContentBinding, ApplicationContentFieldRef,
    ApplicationContentReferenceBinding,
};

impl<S, B, Capture> WorthQueryProducerInputReader<'_, '_, '_, S, B, Capture>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    Capture: WorthQueryProducerInputCaptureDefinition<S>,
{
    pub fn content<E, F, C>(
        &mut self,
        entity: &WorthQueryInvariantEntityIdentity<S, E>,
        field: ApplicationContentFieldRef<F, C>,
    ) -> Result<Option<AdmittedContent<C>>, HandlerExecutionDenial>
    where
        F: RecordSelection<Entity = E>,
        F::Binding: ApplicationContentReferenceBinding,
        C: ApplicationContentBinding,
    {
        let result = self.reader.content(entity, field);
        self.checked(result)
    }
}
