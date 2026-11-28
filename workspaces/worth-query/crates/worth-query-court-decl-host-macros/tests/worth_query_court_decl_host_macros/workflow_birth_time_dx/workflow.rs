//! `RequestWorkflow`: a one-slot review workflow whose instances are only
//! ever born with their request. The vocabulary binds the actual creator,
//! its Create output and its own capability through `birth_operation`;
//! instances are disclosed only through the governed subject read.
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::domain::{
    WorthQueryApplicationWorkflowResourceCeiling, WorthQueryApplicationWorkflowSpecInstallation,
    WorthQueryInstalledApplicationWorkflowSpec,
};
use worth_query_host::facade::primary_graph::WorthQueryFixedRuntimeTimeSource;
use worth_query_host::facade::WorthQueryApplicationHome;

use super::control::{Author, AuthorOp, CompanyControl};
use super::creator::{CreatedRequest, OpenRequestBinding};
use super::disclosure::RequestSubject;
use super::disclosure::{DefinitionRead, SubjectRead};
use super::grants::AuthorCapability;
use super::program::{self, owner, BirthHome, BirthProgram, Caller};
use super::schema::BirthSchema;
use super::signing::{self, SigningOwner};
use super::subject_control::{
    AdvanceCapability, AdvanceOp, ReviewBinding, StartCapability, StartOp,
};
use super::COMPANY;

pub struct RequestWorkflow;
impl ApplicationWorkflowSpec for RequestWorkflow {
    type Schema = BirthSchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("workflow-birth-time.dx.request-workflow.v1");
    fn observation_policy() -> ApplicationWorkflowObservationPolicy {
        ApplicationWorkflowObservationPolicy::governed(
            ApplicationWorkflowReadRule::subject::<Self, SubjectRead>(),
            ApplicationWorkflowReadRule::definition::<Self, DefinitionRead>(),
            None,
            ApplicationWorkflowAuditPolicy::NotApplicable,
        )
    }
    fn actor_policy() -> ApplicationWorkflowActorPolicy {
        ApplicationWorkflowActorPolicy::not_applicable()
    }
}

pub type WorkflowHome = WorthQueryApplicationHome<BirthSchema, BirthProgram>;

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
    .operation::<ReviewBinding>()
    .expect("the review slot is installed")
    .birth_operation::<OpenRequestBinding, CreatedRequest>()
    .expect("the creator is installed as a birth")
    .authoring_capability::<AuthorCapability, AuthorOp, CompanyControl>()
    .expect("the authoring capability is installed")
    .instance_start_capability::<StartCapability, StartOp, RequestSubject>()
    .expect("the instance-start capability is installed")
    .advance_capability::<AdvanceCapability, AdvanceOp, RequestSubject>()
    .expect("the advance capability is installed")
    .finish()
    .expect("the request vocabulary is valid")
}

/// The lineage every revision of the request workflow is published under.
pub const LINEAGE: &str = "purchase-request";

/// Born at `review`, a proposal slot that completes to `done`; the second
/// revision completes to `archived` instead.
fn definition(revision: u8) -> ValidatedWorkflowDefinition<RequestWorkflow> {
    let limits = ApplicationWorkflowDefinitionLimits::new(8, 16, 2, components(), 16 * 1024)
        .expect("the definition limits are nonzero");
    let mut graph =
        ApplicationWorkflowDefinitionBuilder::<RequestWorkflow>::new(LINEAGE, limits).unwrap();
    let review = graph.operation_binding::<ReviewBinding>("review").unwrap();
    let done = graph
        .terminal(if revision == 1 { "done" } else { "archived" })
        .unwrap();
    graph
        .start(&review)
        .control(&review, ApplicationWorkflowControlOutcome::Completed, &done);
    graph
        .finish()
        .unwrap()
        .validate()
        .expect("the definition validates")
}

/// Publishes `revision` of the definition, superseding `predecessor`.
pub fn publish(
    home: &WorkflowHome,
    caller: &Caller,
    revision: u8,
    predecessor: Option<PublishedWorkflowDefinitionRef>,
) -> PublishedWorkflowDefinitionRef {
    let runtime = home.runtime().runtime();
    let installed = home
        .runtime()
        .with_host_advancement(|phase| {
            home.workflow_vocabulary::<RequestWorkflow>()
                .unwrap()
                .workflow_spec_on(&phase, runtime.current_world())
                .unwrap()
        })
        .unwrap();
    let contract = installed.bind_definition(definition(revision)).unwrap();
    let expected = predecessor.map_or(
        WorkflowDefinitionExpectedPredecessor::Absent,
        WorkflowDefinitionExpectedPredecessor::Published,
    );
    let author = Author(CompanyControl {
        company: COMPANY.to_owned(),
    });
    let outcome = runtime
        .request(&caller.principal, &caller.scope)
        .mutate(author)
        .without_source()
        .idempotency(&u64::from(revision))
        .prepare_workflow_publication(contract, expected)
        .unwrap_or_else(|denial| panic!("publication prepares: {denial:?}"))
        .execute();
    match outcome {
        WorkflowDefinitionPublicationOutcome::Published(done) => done.definition().clone(),
        other => panic!("definition publication: {other:?}"),
    }
}

/// Opens the in-memory Home over `clock`, binds the vocabulary and
/// publishes the first definition.
pub fn open(clock: WorthQueryFixedRuntimeTimeSource) -> World {
    let (home, signing) = open_home(clock, None);
    let caller = owner(home.runtime().runtime().installed_schema());
    let definition = publish(&home, &caller, 1, None);
    World {
        home,
        definition,
        _signing: signing,
    }
}

/// Opens (or reopens the Directory Home at `directory`) and binds a freshly
/// installed vocabulary. Nothing is published.
pub fn open_home(
    clock: WorthQueryFixedRuntimeTimeSource,
    directory: Option<&std::path::Path>,
) -> (WorkflowHome, SigningOwner) {
    let home = program::open(clock, directory);
    let installed = vocabulary(&home);
    let signing = signing::install(home.runtime().installed_schema());
    let home = home
        .retain_workflow_spec(installed, signing.signing_owner())
        .unwrap_or_else(|refused| panic!("workflow binding: {:?}", refused.denial()));
    (home, signing)
}
