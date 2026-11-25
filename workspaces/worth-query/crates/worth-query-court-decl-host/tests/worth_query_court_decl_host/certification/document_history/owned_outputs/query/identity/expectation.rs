//! A real Source-bound action observes exact member selection support.
use super::*;
#[derive(Clone, Debug, serde::Serialize)]
pub struct Observe(pub worth_foundational::facade::EntityId);
worth_query_structured_value_binding!(pub Input for Observe { identity: "owned-outputs.identity-observer.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for () { identity: "owned-outputs.identity-observer.result.v1" });
worth_query_operation!(pub CheckIdentity for Schema, input Input; access { read_fields: [ModelKey::at("ModelKey")]; });
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
    type Operation = CheckIdentity;
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
    const IDENTITY: &'static str = "owned-outputs.identity-observer.v1";
    const HANDLER_IDENTITY: &'static str = "owned-outputs.identity-observer.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "owned-outputs.identity-observer.key.v1";
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
        input: &Observe,
    ) -> Result<Option<ApplicationQueryParameterSet<Query>>, ApplicationValueEncodeDenial> {
        Ok(Some(
            ApplicationQueryParameterSet::new().bind(super::selected(), input.0)?,
        ))
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
    fn declared_durable_capabilities(
    ) -> Option<&'static [worth_query_host::facade::primary_graph::WorthQueryOperationDurableCapability]> {
        Some(&[worth_query_host::facade::primary_graph::WorthQueryOperationDurableCapability::RelationalMutation])
    }

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
        .declared_operation::<CheckIdentity>(
            CheckIdentity::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
