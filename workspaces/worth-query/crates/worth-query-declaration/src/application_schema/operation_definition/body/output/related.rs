//! A related output names the native selected preimage, never a caller's identifier.
use super::{
    fixed, ApplicationOperationBodyBuilder, ApplicationOperationBodyOutputTarget,
    ApplicationSchema, Denial,
};
use crate::application_operation::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding, WorthQueryApplicationOutputRole,
    WorthQueryExactlyOneOutput, WorthQueryPreserveOutput,
};
use crate::application_schema::{ApplicationRelationRef, OperationReads};

impl<Schema: ApplicationSchema, Operation, Input>
    ApplicationOperationBodyBuilder<Schema, Operation, Input>
{
    /// Preserve the single related entity captured at the body's decision basis.
    /// Missing or ambiguous selection refuses before candidate effects.
    ///
    /// ```compile_fail,E0271
    /// use worth_query_declaration::facade::{application_operation::*, application_schema::*};
    /// fn wrong_action<S, B, R, Rel>(body: ApplicationOperationBodyBuilder<S,B::Operation,B::Input>, relation: ApplicationRelationRef<S,Rel,<B::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope,R::Entity>)
    /// where S: ApplicationSchema, B: ApplicationMutationBinding<S>,
    /// <B::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope: OperationReads<B::Operation>,
    /// R: WorthQueryApplicationOutputRole<Schema=S,Contract=B::Output,Action=WorthQueryCreateOutput,Cardinality=WorthQueryExactlyOneOutput>,
    /// R::Entity: OperationReads<B::Operation>, Rel: OperationReads<B::Operation> {
    ///     let _ = body.preserve_related_output::<B,R>(relation);
    /// }
    /// ```
    pub fn preserve_related_output<Binding, Role>(
        mut self,
        relation: ApplicationRelationRef<
            Schema,
            impl OperationReads<Operation>,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
            Role::Entity,
        >,
    ) -> Result<Self, Denial>
    where
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
        <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope:
            OperationReads<Operation>,
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = WorthQueryPreserveOutput,
            Cardinality = WorthQueryExactlyOneOutput,
        >,
        Role::Entity: OperationReads<Operation>,
    {
        self.add_output(fixed::<Schema, Binding, Role>(
            ApplicationOperationBodyOutputTarget::Related {
                relation: relation.name().into(),
                from: relation.from().into(),
                to: relation.to().into(),
            },
        ))?;
        Ok(self)
    }
}
