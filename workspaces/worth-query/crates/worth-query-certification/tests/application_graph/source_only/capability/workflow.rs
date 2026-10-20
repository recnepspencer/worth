//! Capability-selected completion acceptance is independent of permission to advance.
#[path = "workflow/actor_binding.rs"]
pub(super) mod actor_binding;
#[path = "workflow/actor_program.rs"]
mod actor_program;
#[path = "workflow/bindings.rs"]
pub(super) mod bindings;
#[path = "workflow/captured_actor.rs"]
mod captured_actor;
#[path = "workflow/inspection.rs"]
mod inspection;
#[path = "workflow/program.rs"]
mod program;
use super::{grants::*, host, schema::*};
use crate::document_retention_model::{
    operator_identity::request_scope,
    workflow::{
        definition_limits, install_certification_authentication_with_clock, workflow_resources,
    },
};
use program::*;
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*, application_installation::*,
    declaration::application_program::*, declaration::application_schema::*,
    domain::WorthQueryApplicationWorkflowSpecInstallation,
};
pub struct Spec;
impl ApplicationWorkflowSpec for Spec {
    type Schema = CapabilitySchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("capability-workflow.spec.v1");
    fn observation_policy() -> ApplicationWorkflowObservationPolicy {
        // This completion-only fixture installs no governed workflow-read binding.
        ApplicationWorkflowObservationPolicy::disabled()
    }
    fn actor_policy() -> ApplicationWorkflowActorPolicy {
        // Its real retention/start/advance grants declare no recorded-actor rule.
        ApplicationWorkflowActorPolicy::not_applicable()
    }
}
fn input() -> SetRetentionInput {
    SetRetentionInput {
        identity: DOCUMENT_IDENTITY.into(),
        retention_days: 18,
    }
}
fn advance_input() -> SetRetentionInput {
    input()
}
#[test]
fn completed_workflow_owner_acceptance_requires_fresh_selected_capability() {
    let host = host::install_program::<Program>(true);
    let installed =
        WorthQueryApplicationWorkflowSpecInstallation::<CapabilitySchema, Spec, Program>::begin(
            host.application.installed_schema(),
            host.application.installed_program(),
            workflow_resources(),
        )
        .operation::<bindings::ControlBinding>()
        .unwrap()
        .operation::<bindings::GuardedBinding>()
        .unwrap()
        .authoring_capability::<RetentionCapability, SetRetention, SetRetentionInput>()
        .unwrap()
        .instance_start_capability::<WorkflowInstanceStartCapability, SetRetention, SetRetentionInput>()
        .unwrap()
        .advance_capability::<WorkflowAdvanceCapability, SetRetention, SetRetentionInput>()
        .unwrap()
        .assessment::<super::readback::ReadBinding>()
        .unwrap()
        .approval::<RetentionCapability, SetRetention, SetRetentionInput>()
        .unwrap()
        .finish()
        .unwrap();
    let (authentication, _clock) =
        install_certification_authentication_with_clock(host.application.installed_schema());
    let workflow = host
        .application
        .retain_workflow_spec(installed, authentication.signing_owner())
        .unwrap();
    let host = host::Host {
        application: workflow.program_runtime().clone(),
        contacts: host.contacts,
        projection: host.projection,
    };
    let runtime = workflow.runtime();
    let scope = request_scope();
    let actor = host::authenticate(&host, 1, &scope);
    let (instance, required, owner, competitor, _published, _started) = {
        let request = runtime.request(&actor, &scope);
        let mut builder = ApplicationWorkflowDefinitionBuilder::<Spec>::new(
            "capability-completion",
            definition_limits(),
        )
        .unwrap();
        let propose = builder.operation::<SetRetention>("propose", false).unwrap();
        let effect = builder
            .operation_binding::<bindings::GuardedBinding>("effect")
            .unwrap();
        let done = builder.terminal("done").unwrap();
        builder
            .start(&propose)
            .control(
                &propose,
                ApplicationWorkflowControlOutcome::Completed,
                &effect,
            )
            .control(&effect, ApplicationWorkflowControlOutcome::Completed, &done)
            .operation_input(&propose, &effect);
        let contract = workflow
            .workflow_spec()
            .bind_definition(builder.finish().unwrap().validate().unwrap())
            .unwrap();
        let WorkflowDefinitionPublicationOutcome::Published(published) = request
            .mutate(Control(input()))
            .without_source()
            .idempotency(&0x9176ca00_u64)
            .prepare_workflow_publication(contract, WorkflowDefinitionExpectedPredecessor::Absent)
            .unwrap()
            .execute()
        else {
            panic!("real capability definition publication")
        };
        let WorkflowInstanceStartOutcome::Started(started) = request
            .mutate(Start(input()))
            .without_source()
            .idempotency(&0x9176ca01_u64)
            .prepare_workflow_instance_start(&workflow, published.definition().clone())
            .unwrap()
            .execute()
        else {
            panic!("real capability instance start")
        };
        let instance = started.instance().clone();
        let WorkflowProposalOutcome::Published(_) = request
            .mutate(Control(input()))
            .without_source()
            .idempotency(&0x9176ca02_u64)
            .prepare_workflow_proposal(&workflow, instance.clone())
            .unwrap()
            .execute()
        else {
            panic!("actual original proposal")
        };
        let required = match request
            .mutate(Advance(advance_input()))
            .without_source()
            .idempotency(&0x9176ca03_u64)
            .prepare_workflow_advance(&workflow, instance.clone())
            .unwrap()
            .execute()
        {
            WorkflowProgressOutcome::AwaitingOperation(required) => required,
            _ => panic!("native capability guarded requirement"),
        };
        let owner = request
            .mutate(Effect(input()))
            .without_source()
            .preconditions(TypedMutationPreconditions::new().expect_fact(
                DocumentIdentityField::reference(),
                ApplicationEncodedScalarValue::try_new(DOCUMENT_IDENTITY.to_owned()).unwrap(),
            ))
            .idempotency_owned(0x9176ca04_u64)
            .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
            .for_workflow_operation(&workflow, &required)
            .unwrap_or_else(|_| panic!("original binding"))
            .retain_operation::<Program, Action>(workflow.program_runtime())
            .unwrap_or_else(|_| panic!("actual capability action"));
        let competitor = request
            .mutate(Effect(input()))
            .without_source()
            .preconditions(TypedMutationPreconditions::new().expect_fact(
                DocumentIdentityField::reference(),
                ApplicationEncodedScalarValue::try_new(DOCUMENT_IDENTITY.to_owned()).unwrap(),
            ))
            .idempotency_owned(0x9176ca08_u64)
            .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
            .for_workflow_operation(&workflow, &required)
            .unwrap_or_else(|_| panic!("same original issued requirement"))
            .retain_operation::<Program, Action>(workflow.program_runtime())
            .unwrap_or_else(|_| panic!("preconstructed competing key"));
        (instance, required, owner, competitor, published, started)
    };
    drop(actor);
    drop(scope);
    let authentication_scope = request_scope();
    let actor = host::authenticate(&host, 1, &authentication_scope);
    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    let canceled = WorthQueryRequestScope::new(None, cancellation.token());
    let (owner, canceled_denial) = match owner.advance(
        &runtime
            .request(&actor, &canceled)
            .on_branch(instance.branch()),
    ) {
        WorthQueryWorkflowOperationAdvance::Refused { operation, denial } => (operation, denial),
        _ => panic!("canceled attempt must return original owner without spending slot"),
    };
    assert_eq!(host.contacts.counts(), (0, 0));
    assert_eq!(
        runtime
            .request(&actor, &authentication_scope)
            .on_branch(instance.branch())
            .query(super::readback::ReadRetention {
                identity: DOCUMENT_IDENTITY.to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .retention_days,
        7
    );
    drop(actor);
    drop(authentication_scope);
    let scope = request_scope();
    let actor = host::authenticate(&host, 1, &scope);
    let completed = {
        let request = runtime.request(&actor, &scope).on_branch(instance.branch());
        let inspector = host::authenticate(&host, 2, &scope);
        inspection::assert_unperformed(
            &owner,
            &request,
            &runtime
                .request(&inspector, &scope)
                .on_branch(instance.branch()),
        );
        assert_eq!(host.contacts.counts(), (0, 0));
        let completed = match owner.advance(&request) {
            WorthQueryWorkflowOperationAdvance::Completed(done) => done,
            WorthQueryWorkflowOperationAdvance::Progress(operation) => {
                let _original_owner = operation;
                panic!(
                    "fresh advance returned Progress; canceled={canceled_denial:?}; contacts={:?}",
                    host.contacts.counts()
                );
            }
            WorthQueryWorkflowOperationAdvance::Refused { operation, denial } => {
                let _original_owner = operation;
                panic!("fresh advance refused: {denial:?}; canceled={canceled_denial:?}; contacts={:?}", host.contacts.counts());
            }
            WorthQueryWorkflowOperationAdvance::Terminal(terminal) => {
                panic!("fresh advance terminal: denial={:?}, native={:?}; canceled={canceled_denial:?}; contacts={:?}", terminal.denial(), terminal.native_terminal().and_then(|owner| owner.native_outcome()), host.contacts.counts());
            }
        };
        assert!(
            matches!(completed.evidence(), WorthQuerySourceCompletion::Fresh { result, .. } if result.retention_days == 18)
        );
        assert_eq!(host.contacts.counts(), (1, 1));
        let before_competitor = runtime
            .with_host_advancement(|phase| request.retain_read(&phase).unwrap())
            .unwrap();
        let terminal = match competitor.advance(&request) {
            WorthQueryWorkflowOperationAdvance::Terminal(terminal) => terminal,
            _ => panic!("competing key cannot duplicate original issued authority"),
        };
        assert!(
            matches!(
                terminal
                    .native_terminal()
                    .and_then(|owner| owner.native_outcome()),
                Some(WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift)
            ),
            "competing original client key must retain native guarded intent drift"
        );
        assert!(
            terminal.denial().is_none(),
            "this early keyed stop is not a slot-take denial"
        );
        assert_eq!(host.contacts.counts(), (1, 1));
        let after_competitor = runtime
            .with_host_advancement(|phase| request.retain_read(&phase).unwrap())
            .unwrap();
        assert_eq!(
            before_competitor.selected_commit(),
            after_competitor.selected_commit()
        );
        assert_eq!(
            request
                .query(super::readback::ReadRetention {
                    identity: DOCUMENT_IDENTITY.to_owned()
                })
                .execute()
                .unwrap()
                .rows()[0]
                .retention_days,
            18
        );
        drop(before_competitor);
        drop(after_competitor);
        terminal
            .close(&request)
            .unwrap_or_else(|_| panic!("native unexecuted competitor cleanup"));
        completed
    };
    drop(actor);
    drop(scope);
    let completed = inspection::denied_acceptance(&workflow, &host, &instance, completed);
    assert_eq!(host.contacts.counts(), (1, 1));
    let scope = request_scope();
    let actor = host::authenticate(&host, 1, &scope);
    let request = runtime.request(&actor, &scope).on_branch(instance.branch());
    assert_eq!(
        request
            .query(super::readback::ReadRetention {
                identity: DOCUMENT_IDENTITY.to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .retention_days,
        18,
        "fresh refusal retained real committed business truth"
    );
    let original_effect_receipt = match completed.evidence() {
        WorthQuerySourceCompletion::Fresh { receipt, .. } => receipt.clone(),
        _ => panic!("this journey retains the original fresh receipt"),
    };
    let before_confirmation = runtime
        .with_host_advancement(|phase| request.retain_read(&phase).unwrap())
        .unwrap();
    let accepted = request
        .mutate(Advance(advance_input()))
        .without_source()
        .idempotency(&0x9176ca06_u64)
        .prepare_workflow_advance(&workflow, instance.clone())
        .unwrap()
        .accept_owned_operation(completed)
        .unwrap_or_else(|(_, cause)| {
            panic!("fresh original capability admits same completed owner: {cause:?}")
        });
    let WorkflowProgressOutcome::Completed(confirmed) = &accepted.progress else {
        panic!("native original settlement confirmation must project progress");
    };
    assert!(confirmed.replayed());
    assert_eq!(confirmed.node_path(), required.node_path());
    assert_eq!(confirmed.node_path(), "effect");
    assert!(
        !confirmed.terminal(),
        "confirmation cannot execute the successor"
    );
    // Native custody readback is historical: terminal accounting is Recovered,
    // while the unique fresh source receipt retains Executed/performed custody.
    assert_eq!(
        confirmed.receipt().commit_reference(),
        original_effect_receipt.commit_reference()
    );
    assert_eq!(
        confirmed.receipt().outcome_identity(),
        original_effect_receipt.outcome_identity()
    );
    assert_eq!(
        confirmed.receipt().authority_binding(),
        original_effect_receipt.authority_binding()
    );
    assert_eq!(
        confirmed.receipt().committed_changes(),
        original_effect_receipt.committed_changes()
    );
    assert!(confirmed.operation_receipt_identity().is_some());
    assert!(original_effect_receipt
        .committed_changes()
        .entity_changes()
        .any(|(entity, _)| entity == confirmed.transition()));
    let after_confirmation = runtime
        .with_host_advancement(|phase| request.retain_read(&phase).unwrap())
        .unwrap();
    assert_eq!(
        before_confirmation.selected_commit(),
        after_confirmation.selected_commit()
    );
    assert_eq!(host.contacts.counts(), (1, 1));
    assert_eq!(
        request
            .query(super::readback::ReadRetention {
                identity: DOCUMENT_IDENTITY.to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .retention_days,
        18
    );
    drop(before_confirmation);
    drop(after_confirmation);
    assert!(
        matches!(request.mutate(Advance(advance_input())).without_source().idempotency(&0x9176ca07_u64).prepare_workflow_advance(&workflow,instance).unwrap().execute(),WorkflowProgressOutcome::Completed(ref transition) if transition.terminal()),
        "independent native terminal progression"
    );
    accepted
        .operation
        .close(&request)
        .unwrap_or_else(|_| panic!("same completion actual cleanup"));
}
