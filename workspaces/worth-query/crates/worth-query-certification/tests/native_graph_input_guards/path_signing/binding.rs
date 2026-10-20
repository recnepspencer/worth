//! All signing variants retain the original row-affine source and logical key.
use super::{
    super::{
        actions::{DenialBinding, ResultBinding},
        revision_query,
        schema::*,
    },
    declaration::*,
};
use worth_query_decl::facade::{application_operation::*, application_schema::*};
use worth_query_host::facade::primary_graph::{
    WorthQueryOperationBodyDecision, WorthQueryOperationBodyDenial,
};
macro_rules! binding {
    ($name:ident,$input:ty,$input_binding:ty,$operation:ty,$writes:literal,$creates:literal,$links:literal) => {
        pub struct $name;
        impl ApplicationMutationAdmissionBinding<Schema> for $name {
            type Admission = ApplicationConventionalAdmission;
        }
        impl ApplicationMutationBinding<Schema> for $name {
            type Input = $input;
            type InputBinding = $input_binding;
            type Result = ();
            type ResultBinding = ResultBinding;
            type IdempotencyKey = u64;
            type Operation = $operation;
            type Decision = WorthQueryOperationBodyDecision<Schema, Row>;
            type Denial = WorthQueryOperationBodyDenial;
            type DenialBinding = DenialBinding;
            type Output = NoApplicationMutationOutputs;
            type ScopeBinding = ApplicationMutationFieldScope<
                Schema,
                Row,
                RowFacts,
                RowIdentity,
                String,
                ReadOnly,
                NoApplicationUnit,
            >;
            type PrincipalBinding = PrincipalBinding;
            type Mapping = Mapping;
            type Principal = Principal;
            type PrincipalIdentity = u64;
            type PrincipalIdentityBinding = U64ApplicationValueBinding;
            type SourceExpectation = ApplicationQueryMutationSource<revision_query::Query>;
            const IDENTITY: &'static str = concat!("worth.query.signed-path.", stringify!($name));
            const HANDLER_IDENTITY: &'static str =
                concat!("worth.query.signed-path.handler.", stringify!($name));
            const IDEMPOTENCY_IDENTITY: &'static str =
                concat!("worth.query.signed-path.request.", stringify!($name));
            const REQUIRES_APPLICATION_PROGRAM: bool = true;
            fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
                ApplicationWorkflowExecutionRequirements::not_applicable()
            }
            fn scope_field() -> ApplicationFieldRef<
                Schema,
                Row,
                RowFacts,
                RowIdentity,
                String,
                ReadOnly,
                EqualityPredicate,
                NoApplicationUnit,
            > {
                RowIdentity::reference()
            }
            fn principal_binding() -> ApplicationPrincipalBindingRef<
                Schema,
                PrincipalBinding,
                Mapping,
                Principal,
                u64,
                U64ApplicationValueBinding,
            > {
                PrincipalBinding::reference()
            }
        }
        impl ApplicationMutationIntent<Schema> for $input {
            type Binding = $name;
            fn input(&self) -> &Self {
                self
            }
            fn scope_binding(&self) -> <$name as ApplicationMutationBinding<Schema>>::ScopeBinding {
                ApplicationMutationFieldScope::new(RowIdentity::reference(), self.0.clone())
            }
        }
    };
}
binding!(GraphBinding, SignGraph, GraphInput, GraphSign, 9, 2, 2);
binding!(
    GraphAliasBinding,
    SignGraphAlias,
    GraphAliasInput,
    GraphAliasSign,
    6,
    1,
    1
);
binding!(
    ScopeAliasBinding,
    SignScopeAlias,
    ScopeAliasInput,
    ScopeAliasSign,
    4,
    1,
    1
);
