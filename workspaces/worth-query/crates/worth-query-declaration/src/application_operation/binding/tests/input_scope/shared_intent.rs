//! Distinct intents share one input through the context spelling: each intent
//! dispatches to its own binding, lends the shared input by reference and
//! selects its scope from the intent.
use super::*;

struct Context;
impl ApplicationBindingContext<MutationSchema> for Context {
    type PrincipalBinding = PrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Account;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type IdempotencyKey = u64;
    type DenialBinding = RenameDenialBinding;
    const IDEMPOTENCY_IDENTITY: &'static str = "shared-scope.command";
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        MutationSchema,
        PrincipalBinding,
        ExternalMapping,
        Account,
        u64,
        U64ApplicationValueBinding,
    > {
        RenameBinding::principal_binding()
    }
}

struct Open {
    shared: CommonInput,
}
struct Close {
    shared: CommonInput,
}
fn open_input(intent: &Open) -> &CommonInput {
    &intent.shared
}
fn open_scope(intent: &Open) -> u64 {
    intent.shared.initial_account
}
fn close_input(intent: &Close) -> &CommonInput {
    &intent.shared
}
crate::worth_query_mutation_binding!(OpenBinding for CommonInput, schema MutationSchema, context Context,
    intent Open => open_input,
    identity "shared-scope.open", input CommonInputBinding, operation CommonOperation,
    result RenameResultBinding, decision RenameDecision,
    handler identity "shared-scope.open.handler", outputs RenameOutputs,
    scope AccountIdField, field AccountIdField::at("AccountIdField"), value open_scope,
);
crate::worth_query_mutation_binding!(CloseBinding for CommonInput, schema MutationSchema, context Context,
    intent Close => close_input,
    identity "shared-scope.close", input CommonInputBinding, operation CommonOperation,
    result RenameResultBinding, decision RenameDecision,
    handler identity "shared-scope.close.handler", outputs RenameOutputs,
    scope AccountIdField, principal_field AccountIdField::at("AccountIdField"),
);

#[test]
fn intents_sharing_one_input_dispatch_to_their_own_binding_and_scope() {
    let shared = CommonInput {
        initial_account: 17,
        preserved_account: 29,
    };
    let (open, close) = (
        Open {
            shared: shared.clone(),
        },
        Close { shared },
    );
    assert!(std::ptr::eq(open.input(), &open.shared));
    assert!(std::ptr::eq(close.input(), &close.shared));
    assert_eq!(open.scope_binding().into_field_parts(&900).1, 17);
    assert_eq!(close.scope_binding().into_field_parts(&900).1, 900);
    assert_eq!(
        TypeId::of::<<Open as ApplicationMutationIntent<MutationSchema>>::Binding>(),
        TypeId::of::<OpenBinding>()
    );
    assert_eq!(
        TypeId::of::<<Close as ApplicationMutationIntent<MutationSchema>>::Binding>(),
        TypeId::of::<CloseBinding>()
    );
    assert_eq!(
        TypeId::of::<<OpenBinding as ApplicationMutationBinding<MutationSchema>>::Input>(),
        TypeId::of::<CommonInput>()
    );
    assert_eq!(
        <CloseBinding as ApplicationMutationBinding<MutationSchema>>::IDEMPOTENCY_IDENTITY,
        "shared-scope.command"
    );
    assert_eq!(open.operation().name(), "CommonOperation");
}
