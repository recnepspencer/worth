//! Content is an ordinary recorded decision field of this owner call.
use super::*;
use crate::domain_computation::primary_graph::{AdmittedContent, ContentReadDenial};
use worth_query_declaration::facade::application_schema::{
    fragment::RecordSelection, ApplicationContentBinding, ApplicationContentFieldRef,
    ApplicationContentReferenceBinding,
};
impl<S: ApplicationSchema, O> WorthQueryComputationReader<'_, '_, '_, S, O> {
    /// Reads one installed content field with the call's dependency evidence.
    pub fn content<E, F, C>(
        &mut self,
        entity: &WorthQueryInvariantEntityIdentity<S, E>,
        field: ApplicationContentFieldRef<F, C>,
    ) -> Result<Option<AdmittedContent<C>>, ContentReadDenial>
    where
        F: RecordSelection<Entity = E>,
        F::Binding: ApplicationContentReferenceBinding,
        C: ApplicationContentBinding,
    {
        self.reader.content(entity, field)
    }
}
