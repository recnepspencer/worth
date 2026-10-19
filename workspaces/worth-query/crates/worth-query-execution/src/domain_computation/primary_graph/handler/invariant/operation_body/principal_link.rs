//! Retain admission's exact observed principal target before constructing any effects.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryInvariantMutationTarget;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyCreationLinkSource as Source, ApplicationOperationBodyProgram,
};
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> DecisionReader<'_, '_, '_, S, B> {
    pub(in crate::domain_computation::primary_graph) fn body_principal_link(
        &mut self,
        body: &ApplicationOperationBodyProgram,
    ) -> Result<Option<WorthQueryInvariantMutationTarget<S, ()>>, HandlerExecutionDenial> {
        let source = body
            .creation()
            .into_iter()
            .flat_map(|c| c.creation_roots())
            .flat_map(|c| c.links())
            .find_map(|l| match &l.source {
                Source::Principal { identity, .. } => Some(identity),
                Source::Scope => None,
            });
        let Some(field) = source else { return Ok(None) };
        let _ = self.principal_identity();
        self.checkpoint().map_err(HandlerExecutionDenial::new)?;
        let identity = self.reader.body_principal_identity(field)?;
        if !self
            .reader
            .body_field(&identity, field, BodyFieldExpectation::Present)?
        {
            return Err(HandlerExecutionDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(
                crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact, field.entity())));
        }
        self.reader
            .mutation_target(&identity)
            .map(Some)
            .map_err(HandlerExecutionDenial::new)
    }
}
