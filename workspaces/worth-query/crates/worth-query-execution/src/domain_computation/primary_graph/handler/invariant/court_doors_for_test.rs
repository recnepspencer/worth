//! Sealed test doors over this module's private members.
use super::*;

impl<'borrow, 'reader, 'runtime, Schema, Binding>
    DecisionReader<'borrow, 'reader, 'runtime, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    #[doc(hidden)]
    pub fn new_for_test(
        reader: &'borrow mut WorthQueryApplicationOperationInvariantProjectionReader<
            'reader,
            'runtime,
            Schema,
            Binding::Operation,
        >,
        scope: &'borrow WorthQueryInvariantEntityIdentity<
            Schema,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        principal_identity: &'borrow Binding::PrincipalIdentity,
        operation_scope_binding: &'borrow WorthQueryOperationScopeBinding,
        identities: &'borrow ApplicationMutationIdentities<'borrow, Schema, Binding>,
        execution: &'borrow QueryRequestExecution<'borrow>,
        context_use: &'borrow Cell<DecisionContextUse>,
        interruption: &'borrow Cell<Option<HandlerInterruption>>,
        conditional_definitions: super::super::ConditionalDefinitionAdmission<'borrow, Schema>,
    ) -> Self {
        Self::new(
            reader,
            scope,
            principal_identity,
            operation_scope_binding,
            identities,
            execution,
            context_use,
            interruption,
            conditional_definitions,
        )
    }
}
