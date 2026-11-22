//! Bind the actual installed forward/inverse provider to one Native Source service.
use super::*;
impl<S, I, P> SourceFactory for TypedIndexProvider<S, I, P>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    fn definition(
        &self,
    ) -> &worth_query_declaration::facade::application_query::ErasedApplicationManagedIndexDefinition
    {
        self.installed.definition()
    }
    fn bind(
        self: Arc<Self>,
        execution: &ExecutionRequest<'_, '_>,
        program: Program,
        cold: Cold,
        authority: super::super::initialization::SourceInitializationAuthority,
    ) -> Result<
        Arc<dyn SourceService>,
        worth_relational::facade::transactions::RelationalExecutionDenialCause,
    > {
        let predicate = self.predicate.as_ref().map(|factory|
            crate::domain_computation::primary_graph::application_query::derived_view::SourcePredicateAxis::new(execution, Arc::clone(factory))).transpose()?;
        Ok(Arc::new(TypedSourceService {
            provider: self,
            predicate,
            program,
            cold,
            initialization: authority,
        }))
    }
}
