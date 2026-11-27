//! `RequestWorkflow`: a one-slot review workflow whose instances are only
//! ever born with their request. The vocabulary binds the actual creator,
//! its Create output and its own capability through `birth_operation`.
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::domain::{
    WorthQueryApplicationWorkflowResourceCeiling, WorthQueryApplicationWorkflowSpecInstallation,
    WorthQueryInstalledApplicationWorkflowSpec,
};
use worth_query_host::facade::primary_graph::WorthQueryFixedRuntimeTimeSource;
use worth_query_host::facade::WorthQueryWorkflowApplicationHome;

use super::control::{AdvanceOp, Author, AuthorBinding, AuthorOp, CompanyControl, StartOp};
use super::creator::{CreatedRequest, OpenRequestBinding};
use super::grants::{AdvanceCapability, AuthorCapability, StartCapability};
use super::program::{self, owner, BirthHome, BirthProgram, Caller};
use super::schema::BirthSchema;
use super::signing::{self, SigningOwner};
use super::COMPANY;

pub struct RequestWorkflow;
impl ApplicationWorkflowSpec for RequestWorkflow {
    type Schema = BirthSchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("workflow-birth-time.dx.request-workflow.v1");
    fn observation_policy() -> ApplicationWorkflowObservationPolicy {
        ApplicationWorkflowObservationPolicy::disabled()
    }
    fn actor_policy() -> ApplicationWorkflowActorPolicy {
        ApplicationWorkflowActorPolicy::not_applicable()
    }
}

pub type WorkflowHome =
    WorthQueryWorkflowApplicationHome<BirthSchema, RequestWorkflow, BirthProgram>;

/// A Home bound to the request vocabulary, with its published definition.
pub struct World {
    pub home: WorkflowHome,
    pub definition: PublishedWorkflowDefinitionRef,
    _signing: SigningOwner,
}

fn components() -> ApplicationWorkflowComponentLimits {
    ApplicationWorkflowComponentLimits::new(8, 2, 32, 64, 64).unwrap()
}

fn vocabulary(
    home: &BirthHome,
) -> WorthQueryInstalledApplicationWorkflowSpec<BirthSchema, RequestWorkflow> {
    let resources = WorthQueryApplicationWorkflowResourceCeiling::new(
        8,
        16,
        2,
        components(),
        16 * 1024,
        8,
        32,
        64 * 1024,
    )
    .expect("the workflow ceilings are nonzero");
    WorthQueryApplicationWorkflowSpecInstallation::<BirthSchema, RequestWorkflow, BirthProgram>::begin(
        home.runtime().installed_schema(),
        home.runtime().installed_program(),
        resources,
    )
    .operation::<AuthorBinding>()
    .expect("the review slot is installed")
    .birth_operation::<OpenRequestBinding, CreatedRequest>()
    .expect("the creator is installed as a birth")
    .authoring_capability::<AuthorCapability, AuthorOp, CompanyControl>()
    .expect("the authoring capability is installed")
    .instance_start_capability::<StartCapability, StartOp, CompanyControl>()
    .expect("the instance-start capability is installed")
    .advance_capability::<AdvanceCapability, AdvanceOp, CompanyControl>()
    .expect("the advance capability is installed")
    .finish()
    .expect("the request vocabulary is valid")
}

/// Born at `review`, a proposal slot that completes to `done`.
fn definition() -> ValidatedWorkflowDefinition<RequestWorkflow> {
    let limits = ApplicationWorkflowDefinitionLimits::new(8, 16, 2, components(), 16 * 1024)
        .expect("the definition limits are nonzero");
    let mut graph =
        ApplicationWorkflowDefinitionBuilder::<RequestWorkflow>::new("purchase-request", limits)
            .unwrap();
    let review = graph.operation_binding::<AuthorBinding>("review").unwrap();
    let done = graph.terminal("done").unwrap();
    graph
        .start(&review)
        .control(&review, ApplicationWorkflowControlOutcome::Completed, &done);
    graph
        .finish()
        .unwrap()
        .validate()
        .expect("the definition validates")
}

fn publish(home: &WorkflowHome, caller: &Caller) -> PublishedWorkflowDefinitionRef {
    let runtime = home.runtime().runtime();
    let installed = home
        .runtime()
        .with_host_advancement(|phase| {
            home.vocabulary()
                .workflow_spec_on(&phase, runtime.current_world())
                .unwrap()
        })
        .unwrap();
    let contract = installed.bind_definition(definition()).unwrap();
    let author = Author(CompanyControl {
        company: COMPANY.to_owned(),
    });
    let outcome = runtime
        .request(&caller.principal, &caller.scope)
        .mutate(author)
        .without_source()
        .idempotency(&1_u64)
        .prepare_workflow_publication(contract, WorkflowDefinitionExpectedPredecessor::Absent)
        .unwrap_or_else(|denial| panic!("publication prepares: {denial:?}"))
        .execute();
    match outcome {
        WorkflowDefinitionPublicationOutcome::Published(done) => done.definition().clone(),
        other => panic!("definition publication: {other:?}"),
    }
}

/// Opens the Home over `clock`, binds the vocabulary and publishes the definition.
pub fn open(clock: WorthQueryFixedRuntimeTimeSource) -> World {
    let home = program::open(clock);
    let installed = vocabulary(&home);
    let signing = signing::install(home.runtime().installed_schema());
    let home = home
        .retain_workflow_spec(installed, signing.signing_owner())
        .unwrap_or_else(|refused| panic!("workflow binding: {:?}", refused.denial()));
    let caller = owner(home.runtime().runtime().installed_schema());
    let definition = publish(&home, &caller);
    World {
        home,
        definition,
        _signing: signing,
    }
}
