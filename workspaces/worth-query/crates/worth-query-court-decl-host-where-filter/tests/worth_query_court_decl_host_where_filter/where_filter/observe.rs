//! A no-op mutation bound to an earlier search's exact source: Query
//! refuses it once anything that search consulted has changed.
use super::queries::{search_parameters, Search, SearchQuery};
use super::schema::*;
use worth_query_decl::facade::{
    application_operation::*, application_query::*, application_schema::*, *,
};
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Debug, WorthValue)]
pub struct Observe {
    pub project: u64,
    pub q: String,
}
#[derive(Clone, Debug)]
pub struct Observed;
#[derive(Clone, Debug)]
pub enum Denied {}
worth_query_structured_value_binding!(pub Input for Observe {identity:"where-filter.observe.input.v1"});
worth_query_structured_value_binding!(pub ResultBinding for Observed {identity:"where-filter.observe.result.v1"});
worth_query_structured_value_binding!(pub DenialBinding for Denied {identity:"where-filter.observe.denial.v1"});
worth_query_operation!(pub ObserveSearch for Schema, input Input;
    access { read_fields: [ProjectKey::at("ProjectKey")]; });
pub struct Binding;
impl ApplicationMutationAdmissionBinding<Schema> for Binding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<Schema> for Binding {
    type Input = Observe;
    type InputBinding = Input;
    type Result = Observed;
    type ResultBinding = ResultBinding;
    type IdempotencyKey = String;
    type Operation = ObserveSearch;
    type Decision = ();
    type Denial = Denied;
    type DenialBinding = DenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<ProjectKey>;
    type PrincipalBinding = PrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<SearchQuery>;
    const IDENTITY: &'static str = "where-filter.observe.v1";
    const HANDLER_IDENTITY: &'static str = "where-filter.observe-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "where-filter.observe-key.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> ProjectKey {
        ProjectKey::at("ProjectKey")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        PrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        PrincipalBinding::reference()
    }
    fn expected_source_parameters(
        input: &Observe,
    ) -> Result<Option<ApplicationQueryParameterSet<SearchQuery>>, ApplicationValueEncodeDenial>
    {
        Ok(Some(search_parameters(&Search {
            project: input.project,
            q: input.q.clone(),
        })))
    }
}
impl ApplicationMutationIntent<Schema> for Observe {
    type Binding = Binding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> <Binding as ApplicationMutationBinding<Schema>>::ScopeBinding {
        ApplicationMutationFieldScope::new(ProjectKey::at("ProjectKey"), self.project)
    }
}
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        _: &Observe,
        _: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<(), Denied> {
        HandlerResult::Completed(())
    }
    fn build_candidate(
        &self,
        _: &Observe,
        _: (),
        _: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Observed, Denied> {
        HandlerResult::Completed(Observed)
    }
}
impl worth_query_decl::facade::application_schema::fragment::Registration<Schema>
    for ObserveSearch
{
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder
            .declared_operation::<ObserveSearch>(
                ObserveSearch::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<Binding>())
    }
}
