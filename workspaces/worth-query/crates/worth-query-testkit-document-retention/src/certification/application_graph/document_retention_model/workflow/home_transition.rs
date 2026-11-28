//! A Directory Home holding governed workflow records, transitioned (D16) to
//! a Relational-only successor under explicit workflow dispositions (Phase 7
//! "Workflow migration"). Shared by the Query graceful court and the Store
//! court that kills the transition at its durable barriers.
use super::super::{
    actor_generation, host, model_lifecycle, programs,
    retention_entry::{ReviewedSetRetentionIntent, DOCUMENT_IDENTITY},
    schema::{DocumentRetentionField, DocumentRetentionSchema as Schema, SetRetentionInput},
};
use super::governed_read::GovernedDocumentWorkflow;
use super::{authoring_intent, WorkflowInstanceStartInput, WorkflowInstanceStartIntent};
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::{
    application_entry::*,
    application_installation::ApplicationBuildIdentity,
    home_program_transition::{
        AspectFieldPatch, AspectValue, DerivedIndexMaintenanceBudget, EntityMutationIntent,
        MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
        WorthQueryHomeProgramTransitionReadSource, WorthQueryHomeProgramTransitionTarget,
    },
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryApplicationHomeLocation as Location, WorthQueryApplicationHomeRequest,
    WorthQueryHomeProgramTransitionPlan as Plan, WorthQueryHomeProgramTransitionStep as Step,
    WorthQueryApplicationHome,
};
#[path = "home_transition/reviewed.rs"]
mod reviewed;
pub use reviewed::reviewed;

/// A Home serving workflows under one of the programs below.
pub type WorkflowHome<Program> =
    WorthQueryApplicationHome<Schema, Program>;
/// The guarded operation node a proposal captures its target input for.
pub type TargetNode = ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>;

/// The workflow-holding predecessor; it has no authored Signal programme.
pub struct SourceProgram;
impl ApplicationProgramDefinition<Schema> for SourceProgram {
    type Contributions = (host::RelationalRetentionContribution,);
    type Outputs =
        <programs::WorkflowOwnerProgram as ApplicationProgramDefinition<Schema>>::Outputs;
    type Rules = <programs::WorkflowOwnerProgram as ApplicationProgramDefinition<Schema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("durable-workflow.relational-source.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        programs::WorkflowOwnerProgram::feature_specs()
    }
}

/// The transition target installs only Relational execution, as D16 requires.
pub struct TargetProgram;
impl ApplicationProgramDefinition<Schema> for TargetProgram {
    type Contributions = (host::RelationalRetentionContribution,);
    type Outputs = <programs::RetentionProgramP1 as ApplicationProgramDefinition<Schema>>::Outputs;
    type Rules = <programs::RetentionProgramP1 as ApplicationProgramDefinition<Schema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("durable-workflow.relational-target.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        programs::RetentionProgramP1::feature_specs()
    }
}

pub fn source_request(
    location: &Location,
) -> WorthQueryApplicationHomeRequest<'static, Schema, SourceProgram> {
    WorthQueryApplicationHomeRequest::new(
        same_location(location),
        programs::validate::<SourceProgram>(),
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new()
            .support(programs::validate::<TargetProgram>())
            .support(actor_generation::validated_actor_program())
            .support(model_lifecycle::validated_model_lifecycle_program()),
        Schema::declaration().unwrap(),
        ((),),
        build(),
        host::host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        host::seed_host,
    )
}

/// The successor alone; it never initializes an existing Home.
pub fn successor_request(
    location: &Location,
) -> WorthQueryApplicationHomeRequest<'static, Schema, TargetProgram> {
    WorthQueryApplicationHomeRequest::new(
        same_location(location),
        programs::validate::<TargetProgram>(),
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new()
            .support(programs::validate::<SourceProgram>())
            .support(actor_generation::validated_actor_program())
            .support(model_lifecycle::validated_model_lifecycle_program()),
        Schema::declaration().unwrap(),
        ((),),
        build(),
        host::host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        |_, _, _| panic!("the successor cannot initialize an existing Home"),
    )
}

pub fn resources() -> DerivedIndexMaintenanceBudget {
    DerivedIndexMaintenanceBudget {
        maximum_work_units: Some(1_000_000),
        maximum_cold_record_slots: Some(100_000),
        maximum_derived_rows: Some(100_000),
    }
}

/// One authored step sets every document's retention to 8 days.
pub fn plan(
    source: &WorthQueryHomeProgramTransitionReadSource<'_, '_>,
    target: &WorthQueryHomeProgramTransitionTarget<'_>,
) -> Result<Plan, WorthQueryPrimaryGraphInstallationDenial> {
    let rows = source.entities("Document", 100_000)?;
    let field = target
        .planned_field(
            "Document",
            DocumentRetentionField::at("DocumentRetentionField").aspect(),
            "DocumentRetentionField",
        )
        .unwrap();
    let mut batch = WorkerIntentBatch::new("transition-document-retention");
    for row in rows.records() {
        batch = batch.push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
            UpdateEntityFieldsIntent {
                entity_id: row.entity_id,
                fields: AspectFieldPatch::from_locator(field.clone(), AspectValue::UInt64(8)),
            },
        )));
    }
    Ok(Plan {
        target: 0,
        schema_transition: None,
        steps: vec![Step {
            batches: vec![batch],
        }],
    })
}

/// Publish the reviewed definition, start two instances and propose the
/// second, so the transition retires one definition and cancels two
/// unperformed instances. Returns the instances' original start intents.
pub fn seed_workflows(
    home: &WorkflowHome<SourceProgram>,
) -> [WorthQueryWorkflowInstanceDiscoveryIntent<Schema>; 2] {
    let (definition, target) = reviewed();
    let definition = publish_with(home, definition);
    let (_, unproposed) = start(home, definition.clone(), 1002);
    let (instance, proposed) = start(home, definition, 1003);
    assert!(matches!(
        propose(home, instance, 1004, &target),
        WorkflowProposalOutcome::Published(_)
    ));
    [unproposed, proposed]
}

pub fn publish_with<Program: ApplicationProgramDefinition<Schema>>(
    application: &WorkflowHome<Program>,
    definition: ValidatedWorkflowDefinition<GovernedDocumentWorkflow>,
) -> PublishedWorkflowDefinitionRef {
    let runtime = application.runtime().runtime();
    let scope = super::super::operator_identity::request_scope();
    let principal =
        super::super::operator_identity::authenticate_operator(runtime.installed_schema(), &scope);
    let installed = application
        .runtime()
        .with_host_advancement(|phase| {
            application
                .workflow_vocabulary::<GovernedDocumentWorkflow>().unwrap()
                .workflow_spec_on(&phase, runtime.current_world())
                .unwrap()
        })
        .unwrap();
    let contract = installed.bind_definition(definition).unwrap();
    match runtime
        .request(&principal, &scope)
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&1001u64)
        .prepare_workflow_publication(contract, WorkflowDefinitionExpectedPredecessor::Absent)
        .unwrap()
        .execute()
    {
        WorkflowDefinitionPublicationOutcome::Published(done) => done.definition().clone(),
        other => panic!("definition publication: {other:?}"),
    }
}

pub fn start<Program: ApplicationProgramDefinition<Schema>>(
    application: &WorkflowHome<Program>,
    definition: PublishedWorkflowDefinitionRef,
    idempotency: u64,
) -> (
    PublishedWorkflowInstanceRef,
    WorthQueryWorkflowInstanceDiscoveryIntent<Schema>,
) {
    let runtime = application.runtime().runtime();
    let scope = super::super::operator_identity::request_scope();
    let principal =
        super::super::operator_identity::authenticate_operator(runtime.installed_schema(), &scope);
    let prepared = runtime
        .request(&principal, &scope)
        .mutate(WorkflowInstanceStartIntent {
            input: WorkflowInstanceStartInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_instance_start(application.workflow_vocabulary::<GovernedDocumentWorkflow>().unwrap(), definition)
        .unwrap();
    let original = prepared.discovery_intent();
    match prepared.execute() {
        WorkflowInstanceStartOutcome::Started(done) => (done.instance().clone(), original),
        other => panic!("instance start: {other:?}"),
    }
}

/// Propose with the guarded operation's original target input (key 1012).
pub fn propose<Program: ApplicationProgramDefinition<Schema>>(
    application: &WorkflowHome<Program>,
    instance: PublishedWorkflowInstanceRef,
    idempotency: u64,
    target: &TargetNode,
) -> WorkflowProposalOutcome {
    let runtime = application.runtime().runtime();
    let scope = super::super::operator_identity::request_scope();
    let principal =
        super::super::operator_identity::authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch());
    let original = request
        .mutate(ReviewedSetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 8,
            },
        })
        .without_source()
        .idempotency_owned(1012u64)
        .unwrap_or_else(|_| panic!("original guarded target description"));
    let inputs = WorthQueryWorkflowOperationInputs::new(application.runtime())
        .with_at::<programs::ReviewedAction, _>(target, original)
        .unwrap_or_else(|_| panic!("actual installed target action"));
    let outcome = request
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_proposal_with_inputs(application.workflow_vocabulary::<GovernedDocumentWorkflow>().unwrap(), instance, &inputs)
        .unwrap()
        .execute();
    assert!(
        matches!(
            &outcome,
            WorkflowProposalOutcome::Published(_) | WorkflowProposalOutcome::RecoveredCommitted(_)
        ),
        "proposal: {outcome:?}"
    );
    outcome
}

fn same_location(location: &Location) -> Location {
    match location {
        Location::MemoryOnly => Location::MemoryOnly,
        Location::Directory(path) => Location::Directory(path.clone()),
    }
}

fn build() -> ApplicationBuildIdentity {
    ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap()
}
