//! Installed operation/Parent correspondence with a single scope-mode hostile twin.
use super::{model::*, queries::*};
use std::marker::PhantomData;
use worth_query_declaration::facade::{application_operation::*, application_schema::*};

pub struct DecisionOperation;
impl<S: ModelSchema> ApplicationOperationMarkerIdentity<S> for DecisionOperation {
    type InputBinding = ParametersBinding;
    const IDENTIFIER: &'static str = "DecisionOperation";
}
impl OperationReads<DecisionOperation> for PrincipalIdentity {}

macro_rules! decision {
    ($name:ident, $scope:ident, $identity:literal) => {
        pub struct $name<S>(PhantomData<fn() -> S>);
        impl<S: ModelSchema> ApplicationMutationAdmissionBinding<S> for $name<S> {
            type Admission = ApplicationConventionalAdmission;
        }
        impl<S: ModelSchema> ApplicationMutationBinding<S> for $name<S> {
            type Input = ();
            type InputBinding = ParametersBinding;
            type Operation = DecisionOperation;
            type Result = ();
            type ResultBinding = ParametersBinding;
            type IdempotencyKey = u64;
            type Decision = ();
            type Denial = ();
            type DenialBinding = ParametersBinding;
            type Output = NoApplicationMutationOutputs;
            type ScopeBinding = $scope<
                S,
                Principal,
                PrincipalFacts,
                PrincipalIdentity,
                u64,
                ReadOnly,
                NoApplicationUnit,
            >;
            type PrincipalBinding = PrincipalBinding;
            type Mapping = Mapping;
            type Principal = Principal;
            type PrincipalIdentity = u64;
            type PrincipalIdentityBinding = U64ApplicationValueBinding;
            type SourceExpectation = NoApplicationMutationSource;
            const IDENTITY: &'static str = $identity;
            const HANDLER_IDENTITY: &'static str =
                "worth.query.tests.managed-parent-decision-handler.v1";
            const IDEMPOTENCY_IDENTITY: &'static str =
                "worth.query.tests.managed-parent-decision-key.v1";
            fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
                ApplicationWorkflowExecutionRequirements::not_applicable()
            }
            fn scope_field() -> ApplicationFieldRef<
                S,
                Principal,
                PrincipalFacts,
                PrincipalIdentity,
                u64,
                ReadOnly,
                EqualityPredicate,
                NoApplicationUnit,
            > {
                PrincipalIdentity::reference::<S>()
            }
            fn principal_binding() -> ApplicationPrincipalBindingRef<
                S,
                PrincipalBinding,
                Mapping,
                Principal,
                u64,
                U64ApplicationValueBinding,
            > {
                principal_binding::<S>()
            }
        }
    };
}
decision!(
    PrincipalDecision,
    ApplicationMutationPrincipalScope,
    "worth.query.tests.managed-parent-principal-decision.v1"
);
decision!(
    FieldDecision,
    ApplicationMutationFieldScope,
    "worth.query.tests.managed-parent-field-decision.v1"
);

pub(super) fn declare<S: ModelSchema>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let operation = ApplicationOperationRef::<S, DecisionOperation, ()>::from_declaration();
    let schema = schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(operation, PrincipalIdentity::reference::<S>())
        .operation_managed_index_read::<ParentBinding<S>, _, _>(operation);
    if S::MODE == 4 {
        schema.application_mutation_binding::<FieldDecision<S>>()
    } else {
        schema.application_mutation_binding::<PrincipalDecision<S>>()
    }
}
