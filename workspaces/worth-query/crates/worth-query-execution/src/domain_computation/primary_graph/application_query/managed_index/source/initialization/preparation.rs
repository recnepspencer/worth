//! Bootstrap admission constructs only Native-proven new available scopes.
use super::*;
use crate::domain_computation::primary_graph::application_query::managed_index::{
    source::service::{Payload, TypedSourceService},
    WorthQueryManagedIndexProvider,
};
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_relational::facade::runtime::{
    ManagedIndexSourceAvailability as Availability, ManagedIndexSourcePreparationDenial as Denial,
    ManagedIndexSourceScope, PreparedManagedIndexSourceRoot,
};
impl<S, I, P> TypedSourceService<S, I, P>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    pub(in crate::domain_computation::primary_graph::application_query::managed_index::source) fn initial_root(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        proof: &Grant,
        scope: ManagedIndexSourceScope<'_>,
        retained: Option<&Payload>,
    ) -> Result<PreparedManagedIndexSourceRoot, Denial> {
        if !self.initialization.admits(proof) || scope.program() != &self.program {
            return Err(Denial::ForeignSource);
        }
        // Propagate unavailable-before: it is never evidence of a new scope.
        if scope.committed_scope_exists()? || !scope.scope_exists()? || retained.is_some() {
            return Err(Denial::ForeignSource);
        }
        if scope.physical_availability() != Availability::Available {
            return Err(Denial::RootUnavailable);
        }
        let root = self.cold(execution, &scope, &scope.proposed())?;
        execution.consult().map_err(|cause| {
            Denial::Source(
                worth_relational::facade::runtime::DerivedRelationReadDenial::Request(cause.into()),
            )
        })?;
        Ok(scope.seal(None, Some(root)))
    }
}
