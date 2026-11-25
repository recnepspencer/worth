//! OperationHandler bodies use `?` across resolve_entity, relations_from, and
//! writer methods with zero IIFE closures and default durable capabilities.
use super::numeric_home::*;
use super::*;
use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::{
    application_contribution::*, application_entry::WorthQueryApplicationRequestExt,
    application_installation::*, primary_graph::*, WorthQueryApplicationHomeLocation,
    WorthQueryApplicationHomeRequest, WorthQuerySourceCustody, WorthQuerySourceCustodyStep,
};

struct QuestionMarkEditHandler;

impl QuestionMarkEditHandler {
    fn decide_target(
        &self,
        input: &model::edit::Edit,
        reader: &mut DecisionReader<
            '_,
            '_,
            '_,
            model::schema::NumericSchema,
            model::edit::EditBinding,
        >,
    ) -> Result<
        WorthQueryInvariantMutationTarget<model::schema::NumericSchema, model::schema::Number>,
        HandlerExecutionDenial,
    > {
        let entity = reader.resolve_entity(model::schema::Key::at("Key"), input.key.clone())?;
        let _previous = reader.field(&entity, model::schema::Primary::at("Primary"))?;
        let _outgoing = reader.relations_from(
            model::authored_identity_model::AuthoredReference::reference(),
            &entity,
        )?;
        reader.mutation_target(&entity)
    }

    fn apply_write(
        &self,
        input: &model::edit::Edit,
        target: WorthQueryInvariantMutationTarget<
            model::schema::NumericSchema,
            model::schema::Number,
        >,
        writer: &mut CandidateWriter<'_, model::schema::NumericSchema, model::edit::EditBinding>,
    ) -> Result<model::edit::Edited, HandlerExecutionDenial> {
        let entity = writer.projected_entity(&target)?;
        let model::edit::Value::Primary(next) = input.value else {
            return Ok(model::edit::Edited);
        };
        writer.write_field(
            &entity,
            model::schema::Primary::at("Primary"),
            model::schema::Counter(next),
        )?;
        Ok(model::edit::Edited)
    }
}

// Intentionally omits `declared_durable_capabilities` to prove the default
// (`Some(&[])`) admits ordinary relational handlers on a Directory Home.
impl OperationHandler<model::schema::NumericSchema, model::edit::EditBinding>
    for QuestionMarkEditHandler
{
    fn decide(
        &self,
        input: &model::edit::Edit,
        reader: &mut DecisionReader<
            '_,
            '_,
            '_,
            model::schema::NumericSchema,
            model::edit::EditBinding,
        >,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<model::schema::NumericSchema, model::schema::Number>,
        model::edit::MissingValue,
    > {
        self.decide_target(input, reader).into()
    }

    fn build_candidate(
        &self,
        input: &model::edit::Edit,
        decision: WorthQueryInvariantMutationTarget<
            model::schema::NumericSchema,
            model::schema::Number,
        >,
        writer: &mut CandidateWriter<'_, model::schema::NumericSchema, model::edit::EditBinding>,
    ) -> HandlerResult<model::edit::Edited, model::edit::MissingValue> {
        self.apply_write(input, decision, writer).into()
    }
}

struct QuestionMarkContribution;
impl ApplicationSchemaContribution<model::schema::NumericSchema> for QuestionMarkContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <model::schema::NumericContribution as ApplicationSchemaContribution<
            model::schema::NumericSchema,
        >>::IDENTITY;
    type Members = <model::schema::NumericContribution as ApplicationSchemaContribution<
        model::schema::NumericSchema,
    >>::Members;
}
impl WorthQueryApplicationContribution<model::schema::NumericSchema> for QuestionMarkContribution {
    type Configuration = ();
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<model::schema::NumericSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        <contribution::DirectoryNumericContribution as WorthQueryApplicationContribution<
            model::schema::NumericSchema,
        >>::contracts(contracts)
    }
    fn configure(
        (): (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, model::schema::NumericSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        contribution::configure_with_edit_handler(setup, QuestionMarkEditHandler)
    }
}

struct QuestionMarkProgram;
impl ApplicationProgramDefinition<model::schema::NumericSchema> for QuestionMarkProgram {
    type Contributions = (QuestionMarkContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("numeric-correction.question-mark-handler.v1");
    fn adoption_authorization<
        V: ApplicationProgramAdoptionAuthorizationVisitor<model::schema::NumericSchema>,
    >(
        visitor: V,
    ) -> V::Output {
        visitor.capability::<model::adoption::AdoptBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        <model::program::Program as ApplicationProgramDefinition<model::schema::NumericSchema>>::feature_specs()
    }
}

#[test]
fn handler_uses_question_mark_across_reads_relations_and_writes_with_default_capabilities() {
    use model::{
        authentication::{authenticate_operator, request_scope},
        edit::{Edit, Value},
        program::Action,
        schema::NumericSchema,
    };
    let path = fixture::location();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let declaration = NumericSchema::declaration().unwrap();
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(path),
        ApplicationProgramAuthoring::<NumericSchema, QuestionMarkProgram>::begin()
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new().support(
            ApplicationProgramAuthoring::<NumericSchema, DirectoryOutputProgram>::begin()
                .validated_program()
                .unwrap(),
        ),
        declaration,
        ((),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        app_limits(),
        |_, graph, installed| seed_numeric(graph, installed),
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("QuestionMarkProgram opens: {refused:?}"));

    let branch = home.primary_graph().current_world();
    {
        let scope = request_scope();
        let principal = authenticate_operator(home.primary_graph().installed_schema(), &scope);
        let request = home.runtime().request(&principal, &scope).on_branch(branch);
        let described = request
            .mutate(Edit {
                key: "numeric".to_owned(),
                value: Value::Primary(42),
            })
            .without_source()
            .idempotency_owned("question-mark-edit-1".to_owned())
            .unwrap_or_else(|(_, _, denial)| panic!("capture edit: {denial:?}"));
        let step =
            WorthQuerySourceCustody::<_, _, QuestionMarkProgram, Action>::Described(described)
                .resume(home.runtime(), &request);
        match step {
            WorthQuerySourceCustodyStep::Committed { .. } => {}
            WorthQuerySourceCustodyStep::Declined { evidence, .. } => {
                panic!("default capabilities and ? conversions declined: {evidence:?}")
            }
            WorthQuerySourceCustodyStep::Pending { cause, .. } => {
                panic!("default capabilities and ? conversions stayed pending: {cause:?}")
            }
        }
        assert_eq!(read_rows(home.runtime(), branch)[0].primary, 42);
    }

    let closed = home
        .close_with_policy(ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(256 << 20), None),
        ))
        .unwrap_or_else(|refused| panic!("policy close: {refused:?}"));
    assert!(closed.is_closed());
    closed.reclaim();
}
