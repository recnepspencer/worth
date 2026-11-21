//! Whole-aspect removal requires its own compile-time marker and installed grant.
use super::CandidateWriter;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationEffectEntity,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_schema::{
    fragment::{Aspect, AspectSelection, Entity, Fragment},
    OperationRemovesAspects,
};
use worth_query_installation::facade::ApplicationSchema;
impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    CandidateWriter<'_, Schema, Binding>
{
    /// Removes one explicitly Optional parent only after all installed children were read.
    /// Field-write markers alone cannot invoke this operation.
    pub fn remove_aspect<F, E, A>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, E>,
        aspect: AspectSelection<F, E, A>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        F: Fragment,
        E: Entity,
        A: Aspect + OperationRemovesAspects<Binding::Operation>,
    {
        self.candidate.remove_aspect(target, aspect)
    }
}
