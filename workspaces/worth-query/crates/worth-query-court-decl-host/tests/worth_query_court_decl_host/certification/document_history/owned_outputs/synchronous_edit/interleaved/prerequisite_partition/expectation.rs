//! A real Source-bound action makes excluded-candidate support observable.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Observe;
worth_query_structured_value_binding!(pub Input for Observe { identity: "owned-outputs.partition-observer.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for () { identity: "owned-outputs.partition-observer.result.v1" });
worth_query_operation!(pub CheckPartition for Schema, input Input; access { read_fields: [ModelKey::at("ModelKey")]; });
pub struct Binding;
impl ApplicationMutationAdmissionBinding<Schema> for Binding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<Schema> for Binding {
    type Input = Observe;
    type InputBinding = Input;
    type Result = ();
    type ResultBinding = ResultBinding;
    type IdempotencyKey = String;
    type Operation = CheckPartition;
    type Decision = ();
    type Denial = edit::Denial;
    type DenialBinding = edit::DenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<ModelKey>;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<Query>;
    const IDENTITY: &'static str = "owned-outputs.partition-observer.v1";
    const HANDLER_IDENTITY: &'static str = "owned-outputs.partition-observer.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "owned-outputs.partition-observer.key.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> ModelKey {
        ModelKey::at("ModelKey")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        Actor,
        Mapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        Actor::reference()
    }
    fn expected_source_parameters(
        _: &Observe,
    ) -> Result<Option<ApplicationQueryParameterSet<Query>>, ApplicationValueEncodeDenial> {
        Ok(Some(super::parameters()))
    }
}
impl ApplicationMutationIntent<Schema> for Observe {
    type Binding = Binding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> <Binding as ApplicationMutationBinding<Schema>>::ScopeBinding {
        ApplicationMutationFieldScope::new(ModelKey::at("ModelKey"), "a".to_owned())
    }
}
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        _: &Observe,
        _: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<(), edit::Denial> {
        HandlerResult::Completed(())
    }
    fn build_candidate(
        &self,
        _: &Observe,
        _: (),
        _: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<(), edit::Denial> {
        HandlerResult::Completed(())
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<CheckPartition>(
            CheckPartition::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
