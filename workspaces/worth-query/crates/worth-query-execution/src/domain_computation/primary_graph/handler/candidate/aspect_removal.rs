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
    ///
    /// ```compile_fail
    /// use worth_query_execution::facade::primary_graph::*;
    /// use worth_query_declaration::facade::{application_schema::*, application_operation::*};
    /// use worth_query_installation::facade::ApplicationSchema;
    /// fn field_authority_is_insufficient<S, B, F, E, A>(
    ///     writer: &mut CandidateWriter<'_, S, B>,
    ///     target: &WorthQueryApplicationEffectEntity<S, E>,
    ///     aspect: fragment::AspectSelection<F, E, A>,
    /// ) where S: ApplicationSchema, B: ApplicationMutationBinding<S>,
    ///     F: fragment::Fragment, E: fragment::Entity,
    ///     A: fragment::Aspect + OperationWrites<B::Operation>
    /// {
    ///     writer.remove_aspect(target, aspect).unwrap();
    /// }
    /// ```
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
