//! Coherence of authored selection under an already-selected binding.
use super::*;
use crate::application_operation::ApplicationMutationInputScope;
use crate::application_schema::*;

#[derive(Clone, serde::Serialize)]
struct CommonInput {
    initial_account: u64,
    preserved_account: u64,
}
crate::worth_query_structured_value_binding!(CommonInputBinding for CommonInput {
    identity: "worth.query.test.common-scope-input.v1"
});
crate::worth_query_operation!(CommonOperation for MutationSchema, input CommonInputBinding);

macro_rules! binding {
    ($name:ident, $input:ty, $input_binding:ty, $operation:ty, $id:literal) => {
        struct $name;
        impl ApplicationMutationBinding<MutationSchema> for $name {
            type Input = $input;
            type InputBinding = $input_binding;
            type Operation = $operation;
            type Result = RenameResult;
            type ResultBinding = RenameResultBinding;
            type IdempotencyKey = u64;
            type Decision = RenameDecision;
            type Denial = RenameDenial;
            type DenialBinding = RenameDenialBinding;
            type Output = RenameOutputs;
            type ScopeBinding =
                <RenameBinding as ApplicationMutationBinding<MutationSchema>>::ScopeBinding;
            type PrincipalBinding = PrincipalBinding;
            type Mapping = ExternalMapping;
            type Principal = Account;
            type PrincipalIdentity = u64;
            type PrincipalIdentityBinding = U64ApplicationValueBinding;
            type SourceExpectation =
                <RenameBinding as ApplicationMutationBinding<MutationSchema>>::SourceExpectation;
            const IDENTITY: &'static str = $id;
            const HANDLER_IDENTITY: &'static str = concat!($id, ".handler");
            const IDEMPOTENCY_IDENTITY: &'static str = concat!($id, ".command");
            const CANDIDATES: crate::application_operation::ApplicationCandidateRequirements =
                RenameBinding::CANDIDATES;
            fn workflow_execution_requirements(
            ) -> crate::application_operation::ApplicationWorkflowExecutionRequirements {
                RenameBinding::workflow_execution_requirements()
            }
            fn scope_field() -> ApplicationFieldRef<
                MutationSchema,
                Account,
                AccountIdentity,
                AccountIdField,
                u64,
                ReadOnly,
                EqualityPredicate,
                NoApplicationUnit,
            > {
                RenameBinding::scope_field()
            }
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
    };
}
binding!(
    InitialBinding,
    CommonInput,
    CommonInputBinding,
    CommonOperation,
    "initial-scope"
);
binding!(
    PreserveBinding,
    CommonInput,
    CommonInputBinding,
    CommonOperation,
    "preserve-scope"
);
binding!(
    OtherBinding,
    RenameInput,
    RenameInputBinding,
    RenameOperation,
    "other-scope"
);
type Scope = <RenameBinding as ApplicationMutationBinding<MutationSchema>>::ScopeBinding;
impl ApplicationMutationInputScope<MutationSchema, InitialBinding> for CommonInput {
    fn scope_for_binding(&self) -> Scope {
        Scope::new(AccountIdField::reference(), self.initial_account)
    }
}
impl ApplicationMutationInputScope<MutationSchema, PreserveBinding> for CommonInput {
    fn scope_for_binding(&self) -> Scope {
        Scope::new(AccountIdField::reference(), self.preserved_account)
    }
}
// The input already has an ordinary intent for RenameBinding. This separate
// explicit association must coexist without replacing that ordinary dispatch.
impl ApplicationMutationInputScope<MutationSchema, OtherBinding> for RenameInput {
    fn scope_for_binding(&self) -> Scope {
        Scope::new(AccountIdField::reference(), self.account_id + 1)
    }
}
#[test]
fn existing_intent_and_two_selected_bindings_keep_distinct_scope_selection() {
    assert_eq!(CommonOperation::reference().name(), "CommonOperation");
    let existing = RenameInput { account_id: 41 };
    let ordinary = existing.scope_binding().into_field_parts(&900).1;
    let delegated = <RenameInput as ApplicationMutationInputScope<MutationSchema, RenameBinding>>::scope_for_binding(&existing).into_field_parts(&900).1;
    let other = <RenameInput as ApplicationMutationInputScope<MutationSchema, OtherBinding>>::scope_for_binding(&existing).into_field_parts(&900).1;
    assert_eq!((ordinary, delegated, other), (41, 41, 42));
    assert_eq!(existing.operation().name(), "RenameOperation");
    let common = CommonInput {
        initial_account: 17,
        preserved_account: 29,
    };
    let initial = <CommonInput as ApplicationMutationInputScope<MutationSchema, InitialBinding>>::scope_for_binding(&common).into_field_parts(&900).1;
    let preserved = <CommonInput as ApplicationMutationInputScope<
        MutationSchema,
        PreserveBinding,
    >>::scope_for_binding(&common)
    .into_field_parts(&900)
    .1;
    assert_eq!((initial, preserved), (17, 29));
}
