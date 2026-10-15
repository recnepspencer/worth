//! A selected branch names the definition it holds current under a workflow
//! identity. A discovered reference grants nothing: a start that names a
//! definition its branch has since superseded or retired is refused with a
//! typed outcome before any effect, and a retry of a recorded start still
//! replays.

use worth_query_host::facade::{
    application_discovery::{
        WorthQueryWorkflowDefinitionDiscovery, WorthQueryWorkflowDefinitionDiscoveryDenial,
    },
    declaration::application_program::{
        ApplicationWorkflowDefinitionIdentity, ApplicationWorkflowSpec,
        ApplicationWorkflowSpecIdentity,
    },
    product::WorthQueryProductBranch,
};

use super::super::document_retention_model::schema::DocumentRetentionSchema;
use super::fork_definitions::fork_of;
use super::*;

fn discover(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &DocumentWorkflowRuntime,
    branch: WorthQueryProductBranch,
    identity: &ApplicationWorkflowDefinitionIdentity,
) -> Result<WorthQueryWorkflowDefinitionDiscovery, WorthQueryWorkflowDefinitionDiscoveryDenial> {
    application
        .with_host_advancement(|host_phase| {
            application
                .runtime()
                .on_branch(branch)
                .select(&host_phase)
                .expect("the branch selects its exact occurrence")
        })
        .expect("installed host call opens")
        .discover_workflow_definition::<ReviewedDocumentWorkflow>(request_execution, identity)
}

#[test]
fn workflow_actor_contract_disabled_discovery_preserves_native_start_currentness() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let application = publish_workflow_on_first_program();
    let main = application.runtime().current_world();
    let draft = terminal_definition("completed");
    let identity = draft.identity().clone();
    assert_eq!(
        discover(request_execution, &application, main, &identity),
        Err(WorthQueryWorkflowDefinitionDiscoveryDenial::ObservationDisabled),
    );

    let first = expect_published(
        "first revision",
        publish_definition(
            &application,
            draft,
            WorkflowDefinitionExpectedPredecessor::Absent,
            601,
        ),
    );
    assert_eq!(
        discover(request_execution, &application, main, &identity),
        Err(WorthQueryWorkflowDefinitionDiscoveryDenial::ObservationDisabled)
    );
    let recorded = expect_started(start_instance(&application, first.clone(), 602));
    let second = expect_published(
        "second revision",
        publish_definition(
            &application,
            terminal_definition("settled"),
            WorkflowDefinitionExpectedPredecessor::Published(first.clone()),
            603,
        ),
    );
    let current = second.clone();
    assert_eq!(current, second);

    let named = expect_superseded_start(start_instance(&application, first.clone(), 604));
    assert_eq!(named, current, "a refused start names what discovery named");
    match start_instance(&application, first.clone(), 602)
        .expect("a recorded start's retry prepares")
    {
        WorkflowInstanceStartOutcome::Started(replay) => {
            assert!(
                replay.replayed(),
                "a superseded definition's recorded start replays"
            );
            assert_eq!(replay.instance(), &recorded);
        }
        other => panic!("a recorded start must replay, got {other:?}"),
    }
    let started = expect_started(start_instance(&application, named, 604));
    assert_ne!(started, recorded, "the refused key recorded nothing");

    expect_retired(retire_definition(&application, second.clone(), 605));
    assert_eq!(
        discover(request_execution, &application, main, &identity),
        Err(WorthQueryWorkflowDefinitionDiscoveryDenial::ObservationDisabled),
    );
    expect_retired_start(start_instance(&application, second, 606));
    let reopened = expect_published(
        "reopened lineage",
        publish_definition(
            &application,
            terminal_definition("reopened"),
            WorkflowDefinitionExpectedPredecessor::Absent,
            607,
        ),
    );
    assert_eq!(
        discover(request_execution, &application, main, &identity),
        Err(WorthQueryWorkflowDefinitionDiscoveryDenial::ObservationDisabled)
    );
}

#[test]
fn workflow_actor_contract_disabled_discovery_preserves_fork_start_control() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let application = publish_workflow_on_first_program();
    let main = application.runtime().current_world();
    let draft = terminal_definition("completed");
    let identity = draft.identity().clone();
    let first = expect_published(
        "main revision",
        publish_definition(
            &application,
            draft,
            WorkflowDefinitionExpectedPredecessor::Absent,
            611,
        ),
    );
    let fork = fork_of(&application, main);
    let copied = first.held_on(fork);
    assert_eq!(copied, first.held_on(fork));
    let successor = expect_published(
        "fork successor",
        publish_definition(
            &application,
            terminal_definition("settled"),
            WorkflowDefinitionExpectedPredecessor::Published(copied),
            612,
        ),
    );
    assert_eq!(
        discover(request_execution, &application, fork, &identity),
        Err(WorthQueryWorkflowDefinitionDiscoveryDenial::ObservationDisabled)
    );
    assert_eq!(
        discover(request_execution, &application, main, &identity),
        Err(WorthQueryWorkflowDefinitionDiscoveryDenial::ObservationDisabled)
    );
    expect_started(start_instance(&application, first, 613));
}

/// A second workflow family over the same schema, which never published the
/// lineage the reviewed-document spec owns.
struct ForeignWorkflow;

impl ApplicationWorkflowSpec for ForeignWorkflow {
    fn observation_policy(
    ) -> worth_query_decl::facade::application_program::ApplicationWorkflowObservationPolicy {
        worth_query_decl::facade::application_program::ApplicationWorkflowObservationPolicy::disabled()
    }
    fn actor_policy(
    ) -> worth_query_decl::facade::application_program::ApplicationWorkflowActorPolicy {
        worth_query_decl::facade::application_program::ApplicationWorkflowActorPolicy::not_applicable()
    }

    type Schema = DocumentRetentionSchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("worth.query.certification.foreign-workflow.v1");
}

#[test]
fn workflow_actor_contract_disabled_discovery_withholds_foreign_lineage() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let application = publish_workflow_on_first_program();
    let main = application.runtime().current_world();
    let draft = terminal_definition("completed");
    let identity = draft.identity().clone();
    let first = expect_published(
        "owned revision",
        publish_definition(
            &application,
            draft,
            WorkflowDefinitionExpectedPredecessor::Absent,
            621,
        ),
    );
    let foreign = application
        .with_host_advancement(|host_phase| {
            application
                .runtime()
                .on_branch(main)
                .select(&host_phase)
                .expect("the branch selects its exact occurrence")
        })
        .expect("installed host call opens")
        .discover_workflow_definition::<ForeignWorkflow>(request_execution, &identity);
    assert_eq!(
        foreign,
        Err(WorthQueryWorkflowDefinitionDiscoveryDenial::ObservationDisabled),
        "a disabled route discloses neither current nor foreign lineage",
    );
    assert_eq!(
        discover(request_execution, &application, main, &identity),
        Err(WorthQueryWorkflowDefinitionDiscoveryDenial::ObservationDisabled)
    );
}
