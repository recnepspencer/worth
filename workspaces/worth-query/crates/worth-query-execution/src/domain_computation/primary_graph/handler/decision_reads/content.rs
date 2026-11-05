//! The ordinary typed decision door; content is field-bound, never a free lookup.
use super::super::invariant::DecisionReader;
use {crate::domain_computation::primary_graph::AdmittedContent, crate::domain_computation::primary_graph::ContentReadDenial, crate::domain_computation::primary_graph::vocabulary::invariant_projection::WorthQueryInvariantEntityIdentity};
use worth_foundational::facade::ContentRefId;
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_schema::{
        ApplicationContentBinding, ApplicationContentFieldRef, ContentRefApplicationValueBinding,
    },
};
use worth_query_installation::facade::{
    ApplicationFieldUnit, ApplicationSchema, DeclaredApplicationFieldValue, OperationReads,
    WritePosture,
};
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> DecisionReader<'_, '_, '_, S, B> {
    /// Content does not confer operation read permission.
    /// ```compile_fail,E0277
    /// use worth_query_execution::facade::primary_graph::{DecisionReader,WorthQueryInvariantEntityIdentity};
    /// use worth_query_declaration::facade::{application_schema::*,application_operation::ApplicationMutationBinding};
    /// use worth_foundational::facade::ContentRefId;
    /// fn missing_read_permission<S:ApplicationSchema,B:ApplicationMutationBinding<S>,E,A,F,W:WritePosture,Q,U:ApplicationFieldUnit,C:ApplicationContentBinding>(
    ///     reader:&mut DecisionReader<'_, '_, '_, S,B>,identity:&WorthQueryInvariantEntityIdentity<S,E>,
    ///     field:ApplicationContentFieldRef<S,E,A,F,W,Q,U,C>,
    /// ) where F:DeclaredApplicationFieldValue<Value=ContentRefId,Binding=ContentRefApplicationValueBinding> {
    ///     let _=reader.content(identity,field);
    /// }
    /// ```
    pub fn content<E, A, F, W, Q, U, C>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<S, E>,
        field: ApplicationContentFieldRef<S, E, A, F, W, Q, U, C>,
    ) -> Result<Option<AdmittedContent<C>>, ContentReadDenial>
    where
        F: OperationReads<B::Operation>
            + DeclaredApplicationFieldValue<
                Value = ContentRefId,
                Binding = ContentRefApplicationValueBinding,
            >,
        W: WritePosture,
        U: ApplicationFieldUnit,
        C: ApplicationContentBinding,
    {
        self.reader.decision_content(identity, field)
    }
}
