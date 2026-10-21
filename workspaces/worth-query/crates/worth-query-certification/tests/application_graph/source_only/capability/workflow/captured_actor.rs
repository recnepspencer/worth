//! Captured target input is distinct from the proposal used by native actor policy.
use super::super::{grants::*, host, readback::ReadRetention, schema::*};
use super::{
    actor_binding::ActorBinding,
    actor_program::{Action, Effect, Program, Spec},
    bindings::ControlBinding,
    program::{Advance, Control, Start},
};
use crate::document_retention_model::{
    operator_identity::request_scope,
    workflow::{
        definition_limits, install_certification_authentication_with_clock, workflow_resources,
    },
};
use worth_query_host::facade::{
    application_entry::*, application_installation::*, declaration::application_program::*,
    domain::WorthQueryApplicationWorkflowSpecInstallation, primary_graph::*,
};
type Workflow = WorthQueryWorkflowApplicationRuntime<CapabilitySchema, Spec, Program>;
type Roster =
    WorthQueryWorkflowInputNode<CapabilitySchema, Effect, Action, WorthQueryWorkflowInputEnd>;
type Inputs = WorthQueryWorkflowOperationInputs<CapabilitySchema, Program, Roster>;
type Selected = WorthQuerySelectedWorkflowOperation<CapabilitySchema, Program, Roster>;
fn input(retention_days: u64) -> SetRetentionInput {
    SetRetentionInput {
        identity: DOCUMENT_IDENTITY.into(),
        retention_days,
    }
}

#[test]
fn captured_target_actor_rule_uses_original_proposal_not_target_input() {
    let installed_host = host::install_program::<Program>(true);
    let installed = WorthQueryApplicationWorkflowSpecInstallation::<CapabilitySchema, Spec, Program>::begin(
        installed_host.application.installed_schema(),
        installed_host.application.installed_program(),
        workflow_resources(),
    )
    .operation::<ControlBinding>().unwrap()
    .operation::<ActorBinding>().unwrap()
    .authoring_capability::<RetentionCapability, SetRetention, SetRetentionInput>().unwrap()
    .instance_start_capability::<WorkflowInstanceStartCapability, SetRetention, SetRetentionInput>().unwrap()
    .advance_capability::<WorkflowAdvanceCapability, SetRetention, SetRetentionInput>().unwrap()
    .assessment::<super::super::readback::ReadBinding>().unwrap()
    .approval::<RetentionCapability, SetRetention, SetRetentionInput>().unwrap()
    .finish().unwrap();
    let (authentication, _clock) = install_certification_authentication_with_clock(
        installed_host.application.installed_schema(),
    );
    let workflow = installed_host
        .application
        .retain_workflow_spec(installed, authentication.signing_owner())
        .unwrap();
    let host = host::Host {
        application: workflow.program_runtime().clone(),
        contacts: installed_host.contacts,
        projection: installed_host.projection,
    };
    let (instance, inputs, required) = propose(&workflow, &host);
    assert_actor_execution(&workflow, &host, &instance, inputs, &required);
}

fn publish(
    workflow: &Workflow,
    host: &host::Host<Program>,
) -> (
    PublishedWorkflowInstanceRef,
    ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>,
) {
    let scope = request_scope();
    let author = host::authenticate(host, 1, &scope);
    let request = workflow.runtime().request(&author, &scope);
    let mut builder = ApplicationWorkflowDefinitionBuilder::<Spec>::new(
        "captured-operation-actor",
        definition_limits(),
    )
    .unwrap();
    let propose = builder
        .operation_binding::<ControlBinding>("propose")
        .unwrap();
    let effect = builder.operation_binding::<ActorBinding>("effect").unwrap();
    let done = builder.terminal("done").unwrap();
    builder
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &effect,
        )
        .control(&effect, ApplicationWorkflowControlOutcome::Completed, &done)
        .captured_operation_input(&propose, &effect);
    let contract = workflow
        .workflow_spec()
        .bind_definition(builder.finish().unwrap().validate().unwrap())
        .unwrap();
    let WorkflowDefinitionPublicationOutcome::Published(published) = request
        .mutate(Control(input(18)))
        .without_source()
        .idempotency(&0x9176cb00_u64)
        .prepare_workflow_publication(contract, WorkflowDefinitionExpectedPredecessor::Absent)
        .unwrap()
        .execute()
    else {
        panic!("native captured-actor definition publication")
    };
    let WorkflowInstanceStartOutcome::Started(started) = request
        .mutate(Start(input(18)))
        .without_source()
        .idempotency(&0x9176cb01_u64)
        .prepare_workflow_instance_start(workflow, published.definition().clone())
        .unwrap()
        .execute()
    else {
        panic!("native captured-actor instance")
    };
    (started.instance().clone(), effect)
}

fn propose(
    workflow: &Workflow,
    host: &host::Host<Program>,
) -> (
    PublishedWorkflowInstanceRef,
    Inputs,
    RequiredWorkflowOperation,
) {
    let (instance, effect) = publish(workflow, host);
    let scope = request_scope();
    let author = host::authenticate(host, 1, &scope);
    let request = workflow
        .runtime()
        .request(&author, &scope)
        .on_branch(instance.branch());
    let original = request
        .mutate(Effect(input(23)))
        .without_source()
        .idempotency_owned(0x9176cb04_u64)
        .unwrap_or_else(|_| panic!("original target description"));
    let inputs = WorthQueryWorkflowOperationInputs::new(workflow.program_runtime())
        .with_at::<Action, _>(&effect, original)
        .unwrap_or_else(|_| panic!("qualified original target"));
    assert!(matches!(
        request
            .mutate(Control(input(18)))
            .without_source()
            .idempotency(&0x9176cb02_u64)
            .prepare_workflow_proposal_with_inputs(workflow, instance.clone(), &inputs)
            .unwrap()
            .execute(),
        WorkflowProposalOutcome::Published(_)
    ));
    let WorkflowProgressOutcome::AwaitingOperation(required) = request
        .mutate(Advance(input(18)))
        .without_source()
        .idempotency(&0x9176cb03_u64)
        .prepare_workflow_advance(workflow, instance.clone())
        .unwrap()
        .execute()
    else {
        panic!("captured actor target requirement")
    };
    (instance, inputs, required)
}

fn assert_actor_execution(
    workflow: &Workflow,
    host: &host::Host<Program>,
    instance: &PublishedWorkflowInstanceRef,
    mut inputs: Inputs,
    required: &RequiredWorkflowOperation,
) {
    let scope = request_scope();
    let proposer = host::authenticate(host, 1, &scope);
    let request = workflow
        .runtime()
        .request(&proposer, &scope)
        .on_branch(instance.branch());
    let owner = inputs
        .select(workflow, required)
        .unwrap_or_else(|cause| panic!("original target selection: {cause:?}"));
    let owner =
        match owner.advance(&request) {
            WorthQueryWorkflowOperationAdvance::Refused {
                operation,
                denial:
                    WorthQueryWorkflowOperationDenial::Source(WorthQuerySourceOnlyDenial::Mutation(
                        WorthQueryApplicationRequestMutationDenial::Authorization(cause),
                    )),
            } => {
                assert_eq!(cause.kind(), WorthQueryOperationAuthorizationDenialKind::Refused(
                WorthQueryOperationAuthorizationRefusalKind::WorkflowActorComparisonMismatch),
                "native actor source: {:?}", cause.workflow_actor_source_denial());
                operation
            }
            WorthQueryWorkflowOperationAdvance::Refused { denial, .. } => {
                panic!("wrong recorded-actor refusal: {denial:?}")
            }
            _ => panic!("otherwise authorized proposer must hit exact recorded-actor comparison"),
        };
    assert_eq!(host.contacts.counts(), (0, 0));
    assert_eq!(
        request
            .query(ReadRetention {
                identity: DOCUMENT_IDENTITY.into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .retention_days,
        7
    );
    let executor = host::authenticate(host, 2, &scope);
    let request = workflow
        .runtime()
        .request(&executor, &scope)
        .on_branch(instance.branch());
    recover_executor(workflow, host, &request, owner);
    assert_eq!(host.contacts.counts(), (1, 1));
    assert_eq!(
        request
            .query(ReadRetention {
                identity: DOCUMENT_IDENTITY.into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .retention_days,
        23
    );
}

fn recover_executor(
    workflow: &Workflow,
    host: &host::Host<Program>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, CapabilitySchema>,
    owner: Selected,
) {
    let branch = owner.required().branch();
    let held = workflow
        .runtime()
        .hold_post_commit_snapshot_failure_for_test()
        .unwrap();
    let WorthQueryWorkflowOperationAdvance::Progress(owner) = owner.advance(request) else {
        panic!("different authorized actor must perform, retaining uncertain custody")
    };
    let owner = match owner.advance(request) {
        WorthQueryWorkflowOperationAdvance::Refused {
            operation,
            denial:
                WorthQueryWorkflowOperationDenial::Source(WorthQuerySourceOnlyDenial::Recovery(
                    WorthQueryApplicationRecoveryRequestDenial::Idempotency(cause),
                )),
        } => {
            assert_eq!(cause.kind(), WorthQueryApplicationIdempotencyResolutionDenialKind::ActiveSnapshotCapacityExhausted { maximum_active_snapshots: 0 });
            operation
        }
        WorthQueryWorkflowOperationAdvance::Refused { denial, .. } => {
            panic!("wrong held recovery refusal: {denial:?}")
        }
        _ => panic!("held recovery must return the original owner"),
    };
    assert_eq!(host.contacts.counts(), (1, 1));
    drop(held);
    let scope = request_scope();
    let executor = host::authenticate(host, 2, &scope);
    let request = workflow
        .runtime()
        .request(&executor, &scope)
        .on_branch(branch);
    let mut owner = owner;
    let completed = loop {
        match owner.advance(&request) {
            WorthQueryWorkflowOperationAdvance::Progress(retained) => owner = retained,
            WorthQueryWorkflowOperationAdvance::Completed(completed) => break completed,
            WorthQueryWorkflowOperationAdvance::Refused { denial, .. } => {
                panic!("different actor recovery: {denial:?}")
            }
            WorthQueryWorkflowOperationAdvance::Terminal(_) => {
                panic!("performed actor operation cannot become an unexecuted terminal")
            }
        }
    };
    let WorthQueryWorkflowOperationChoice::Here(completed) = completed.into_operation();
    assert!(matches!(
        completed.evidence(),
        WorthQuerySourceCompletion::Recovered {
            initial_posture: Some(WorthQueryApplicationUncommitted::Indeterminate(_)),
            ..
        }
    ));
    completed
        .close(&request)
        .unwrap_or_else(|_| panic!("real recovered actor operation cleanup"));
}
