//! Whole-aspect removal requires its own compile-time marker and installed grant.
use super::CandidateWriter;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationEffectEntity,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_schema::{
    ApplicationAspectMarkerIdentity, ApplicationAspectRef, ApplicationEntityMarkerIdentity,
    OperationRemovesAspects,
};
use worth_query_installation::facade::ApplicationSchema;
impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    CandidateWriter<'_, Schema, Binding>
{
    /// Removes one explicitly Optional parent only after all installed children were read.
    /// Field-write markers alone cannot invoke this operation.
    ///
    /// ```compile_fail
    /// use worth_query_execution::facade::primary_graph::*;
    /// use worth_query_declaration::facade::{application_schema::*, application_operation::*};
    /// fn field_authority_is_insufficient<S, B, E, A>(
    ///     writer: &mut CandidateWriter<'_, S, B>,
    ///     target: &WorthQueryApplicationEffectEntity<S, E>,
    ///     aspect: ApplicationAspectRef<S, E, A>,
    /// ) where S: ApplicationSchema, B: ApplicationMutationBinding<S>,
    ///     E: ApplicationEntityMarkerIdentity<S>,
    ///     A: ApplicationAspectMarkerIdentity<S, E> + OperationWrites<B::Operation>
    /// {
    ///     writer.remove_aspect(target, aspect).unwrap();
    /// }
    /// ```
    pub fn remove_aspect<Entity, Aspect>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        aspect: ApplicationAspectRef<Schema, Entity, Aspect>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>,
        Aspect: ApplicationAspectMarkerIdentity<Schema, Entity>
            + OperationRemovesAspects<Binding::Operation>,
    {
        self.candidate.remove_aspect(target, aspect)
    }
}
