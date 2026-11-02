//! An explicitly Source-bound observer preserves authored dependency history.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Observe;
worth_query_structured_value_binding!(pub Input for Observe {identity:"content-source.observer-input.v1"});
worth_query_operation!(pub ReadSource for Schema,input Input;access{read_fields:[Key];});
pub struct Binding;
impl ApplicationMutationAdmissionBinding<Schema> for Binding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<Schema> for Binding {
    type Input = Observe;
    type InputBinding = Input;
    type Result = revision::Revised;
    type ResultBinding = revision::ResultBinding;
    type IdempotencyKey = String;
    type Operation = ReadSource;
    type Decision = ();
    type Denial = operation::Denial;
    type DenialBinding = operation::DenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<
        Schema,
        Record,
        Identity,
        Key,
        String,
        ReadOnly,
        NoApplicationUnit,
    >;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<query::SourceQuery>;
    const IDENTITY: &'static str = "content-source.observer.v1";
    const HANDLER_IDENTITY: &'static str = "content-source.observer-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "content-source.observer-key.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> ApplicationFieldRef<
        Schema,
        Record,
        Identity,
        Key,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        Key::reference()
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
    ) -> Result<
        Option<ApplicationQueryParameterSet<query::SourceQuery>>,
        ApplicationValueEncodeDenial,
    > {
        Ok(Some(ApplicationQueryParameterSet::new()))
    }
}
impl ApplicationMutationIntent<Schema> for Observe {
    type Binding = Binding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> <Binding as ApplicationMutationBinding<Schema>>::ScopeBinding {
        ApplicationMutationFieldScope::new(Key::reference(), "record".to_owned())
    }
}
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        _: &Observe,
        _: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<(), operation::Denial> {
        HandlerResult::Completed(())
    }
    fn build_candidate(
        &self,
        _: &Observe,
        _: (),
        _: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<revision::Revised, operation::Denial> {
        HandlerResult::Completed(revision::Revised)
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<ReadSource>(
            ReadSource::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
