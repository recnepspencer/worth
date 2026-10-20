//! Foreign-Spec non-disclosure and authentic native locator provenance.
use super::super::super::super::{host, programs::RetentionProgramP1, workflow};
use super::*;
use worth_query_host::facade::application_entry::{
    PerformedWorkflowInstanceStart, PublishedWorkflowInstanceRef, WorkflowProgressOutcome,
};
use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowControlOutcome, ApplicationWorkflowDefinitionBuilder,
};

#[test]
fn foreign_spec_present_and_unperformed_intents_are_both_undisclosed() {
    let foreign = host::publish_workflow_on_first_program();
    let definition = match workflow::publish_definition(
        &foreign,
        workflow::terminal_definition("foreign"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        91_600,
    )
    .unwrap()
    {
        WorkflowDefinitionPublicationOutcome::Published(done) => done.definition().clone(),
        other => panic!("foreign definition must really publish: {other:?}"),
    };
    let scope = request_scope();
    let principal = authenticate_operator(foreign.runtime().installed_schema(), &scope);
    let prepared = foreign
        .runtime()
        .request(&principal, &scope)
        .mutate(WorkflowInstanceStartIntent {
            input: start_input(),
        })
        .without_source()
        .idempotency(&91_601_u64)
        .prepare_workflow_instance_start(&foreign, definition.clone())
        .unwrap();
    let present = prepared.discovery_intent();
    let performed = started(prepared.execute());
    let absent = foreign
        .runtime()
        .request(&principal, &scope)
        .mutate(WorkflowInstanceStartIntent {
            input: start_input(),
        })
        .without_source()
        .idempotency(&91_602_u64)
        .prepare_workflow_instance_start(&foreign, definition)
        .unwrap();
    let missing = absent.discovery_intent();
    drop(absent);
    assert!(!performed.replayed());
    drop(performed);
    // Retain a genuine different Spec on the SAME runtime/schema/branch, not a fabricated vocabulary.
    let application = super::super::runtime::retain_with_signing_owner(
        foreign.workflow.into_program_runtime(),
        foreign.authentication,
    )
    .0;
    let operator = authenticate_operator(application.runtime().installed_schema(), &scope);
    let basis = application
        .runtime()
        .request(&operator, &scope)
        .retain_current_read()
        .unwrap();
    for name in [
        "reviewed-document-terminal",
        "foreign-definition-never-published",
    ] {
        let target=worth_query_host::facade::declaration::application_program::ApplicationWorkflowDefinitionReadTarget::<GovernedDocumentWorkflow>::new(
            worth_query_host::facade::declaration::application_program::ApplicationWorkflowDefinitionIdentity::new(name).unwrap());
        let found = application
            .runtime()
            .request(&operator, &scope)
            .at(&basis)
            .discover_workflow_definition::<GovernedDocumentWorkflow, MetadataRead>(
                application.vocabulary(),
                &target,
                input(),
            )
            .unwrap();
        assert_eq!(
            found.definition(),
            &WorthQueryWorkflowDefinitionDiscovery::Unpublished
        );
        assert_eq!(found.basis().selected_commit(), basis.selected_commit());
    }
    for original in [&present, &missing] {
        match read(&application, &operator, &scope, &basis, original).unwrap() {
            WorthQueryWorkflowInstanceRead::Missing { basis: found } => {
                assert_eq!(found.selected_commit(), basis.selected_commit())
            }
            _ => panic!("a foreign Spec never discloses whether its original intent exists"),
        }
    }
}

#[test]
fn migrated_and_fork_successors_reacquire_the_actual_native_resume_path() {
    for forked in [false, true] {
        let application = install();
        let first = publish(&application);
        let source = started(prepare(&application, first.clone(), 91_620).execute());
        let target = if forked {
            first
        } else {
            let mut builder =
                ApplicationWorkflowDefinitionBuilder::<GovernedDocumentWorkflow>::new(
                    "governed-original-intent",
                    workflow::definition_limits(),
                )
                .unwrap();
            let proposal = builder
                .operation::<workflow::WorkflowDefinitionAuthoringOperation>("proposal", false)
                .unwrap();
            let resumed = builder.terminal("resumed").unwrap();
            builder.start(&proposal).control(
                &proposal,
                ApplicationWorkflowControlOutcome::Completed,
                &resumed,
            );
            publication::publish_definition(
                &application,
                builder.finish().unwrap().validate().unwrap(),
                WorkflowDefinitionExpectedPredecessor::Published(first),
                91_621,
            )
        };
        let runtime = application.runtime();
        let scope = request_scope();
        let operator = authenticate_operator(runtime.installed_schema(), &scope);
        let branch = if forked {
            application
                .with_host_advancement(|phase| {
                    runtime
                        .branches()
                        .fork(runtime.current_world())
                        .components(|components| {
                            components.fork_relational().reuse_exact_signal_basis()
                        })
                        .create(&phase)
                        .unwrap()
                })
                .unwrap()
        } else {
            runtime.current_world()
        };
        let mutation = runtime
            .request(&operator, &scope)
            .on_branch(branch)
            .mutate(WorkflowInstanceStartIntent {
                input: start_input(),
            })
            .without_source()
            .idempotency(&91_622_u64);
        let successor = if forked {
            mutation.prepare_workflow_fork_continuation(
                application.vocabulary(),
                source.instance().clone(),
                target,
                "done",
            )
        } else {
            mutation.prepare_workflow_instance_migration(
                application.vocabulary(),
                source.instance().clone(),
                target,
                "resumed",
            )
        }
        .unwrap();
        let original = successor.discovery_intent();
        let performed = started(successor.execute());
        let retained = runtime
            .request(&operator, &scope)
            .on_branch(branch)
            .retain_current_read()
            .unwrap();
        let view = runtime
            .request(&operator, &scope)
            .on_branch(branch)
            .at(&retained)
            .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
                application.vocabulary(),
                &original,
                input(),
                input(),
            )
            .unwrap();
        let found = live(view);
        assert_eq!(found, *performed.instance());
        assert_eq!(
            found.start_node_path(),
            if forked { "done" } else { "resumed" }
        );
        let outcome = advance(&application, found, 91_623);
        assert!(
            matches!(outcome, WorkflowProgressOutcome::Completed(_)),
            "fresh resumed native advance must succeed: {outcome:?}"
        );
    }
}

#[test]
fn carry_keeps_original_start_revision_while_fresh_advance_uses_current_program() {
    let mut application = install();
    let supported = workflow::runtime::install_workflow_spec_for::<
        RetentionProgramP1,
        GovernedDocumentWorkflow,
    >(
        application.runtime().installed_schema(),
        application
            .supported_program::<RetentionProgramP1>()
            .unwrap()
            .installed_program(),
        workflow::runtime::workflow_resources(),
    );
    application
        .support_workflow_spec(supported)
        .unwrap_or_else(|(_supported, denial)| {
            panic!("the vocabulary belongs to a rostered program: {denial:?}")
        });
    let definition = publish(&application);
    let prepared = prepare(&application, definition, 91_640);
    let performed = started(prepared.execute());
    let started_under = *performed.instance().program_revision();
    let scope = request_scope();
    let operator = authenticate_operator(application.runtime().installed_schema(), &scope);
    let target = *application
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .installed_program()
        .revision();
    assert_ne!(target, started_under);
    let programs = application.runtime().request(&operator, &scope).programs();
    let requirements = programs.compare(&target).unwrap();
    let request = programs.adopt(&requirements).unwrap();
    let prepared = request
        .workflow(worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions::carry_compatible())
        .prepare(1_024)
        .unwrap();
    assert!(matches!(prepared.publish(application.runtime(), &operator, &scope),
        worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPublicationOutcome::Performed(_)));
    let retained = application
        .runtime()
        .request(&operator, &scope)
        .retain_current_read()
        .unwrap();
    let target=worth_query_host::facade::declaration::application_program::ApplicationWorkflowDefinitionReadTarget::<GovernedDocumentWorkflow>::new(
        worth_query_host::facade::declaration::application_program::ApplicationWorkflowDefinitionIdentity::new("governed-original-intent").unwrap());
    let page = application
        .runtime()
        .request(&operator, &scope)
        .at(&retained)
        .read_workflow_instances::<GovernedDocumentWorkflow, DefinitionRead>(
            application.vocabulary(),
            &target,
            input(),
            WorthQueryWorkflowReadExtent::Definition,
            8,
            WorthQueryWorkflowIndexRead::PublishedOnly,
            None,
        )
        .unwrap();
    assert_eq!(page.entries().len(), 1);
    let found = match &page.entries()[0] {
        WorthQueryWorkflowInstanceRead::Live { instance, .. } => instance.clone(),
        _ => panic!("token-free carried enumeration must find a live native occurrence"),
    };
    assert_eq!(
        found.program_revision(),
        performed.instance().program_revision()
    );
    assert_eq!(found.program_revision(), &started_under);
    assert_eq!(
        found.start_node_path(),
        performed.instance().start_node_path()
    );
    // The receiving locator retains P0 provenance; the existing native advance reads P1 instance truth.
    let outcome = advance(&application, found, 91_641);
    assert!(
        matches!(outcome, WorkflowProgressOutcome::Completed(_)),
        "fresh carried advance must admit P1: {outcome:?}"
    );
}

fn started(outcome: WorkflowInstanceStartOutcome) -> PerformedWorkflowInstanceStart {
    match outcome {
        WorkflowInstanceStartOutcome::Started(done) => done,
        other => panic!("genuine native start required: {other:?}"),
    }
}
fn live(view: WorthQueryWorkflowInstanceRead) -> PublishedWorkflowInstanceRef {
    match view {
        WorthQueryWorkflowInstanceRead::Live { instance, .. } => instance,
        _ => panic!("authentic current successor must be live"),
    }
}
fn start_input() -> WorkflowInstanceStartInput {
    WorkflowInstanceStartInput {
        document_identity: DOCUMENT_IDENTITY.to_owned(),
    }
}
fn advance(
    application: &GovernedRuntime,
    instance: PublishedWorkflowInstanceRef,
    key: u64,
) -> WorkflowProgressOutcome {
    let scope = request_scope();
    let principal = authenticate_operator(application.runtime().installed_schema(), &scope);
    application
        .runtime()
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(workflow::WorkflowAdvanceIntent {
            input: workflow::WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&key)
        .prepare_workflow_advance(application.vocabulary(), instance)
        .unwrap()
        .execute()
}
