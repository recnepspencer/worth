//! Workflow lifecycle calls made through a Home session's own request, with
//! the workflow Home's installed vocabulary.
use super::*;
use document::retention_entry::ReviewedSetRetentionIntent;
use document::schema::{DocumentRetentionSchema, SetRetentionInput};

pub(super) type Original = WorthQueryWorkflowInstanceDiscoveryIntent<DocumentRetentionSchema>;

/// Binds and publishes the reviewed definition on the session's branch.
pub(super) fn publish(
    home: &Home,
    session: &Session<'_>,
    key: u64,
) -> (PublishedWorkflowDefinitionRef, TargetNode) {
    let (definition, target) = definition::reviewed();
    let installed = home
        .runtime()
        .with_host_advancement(|phase| {
            home.workflow_vocabulary::<GovernedDocumentWorkflow>()
                .unwrap()
                .workflow_spec_on(&phase, home.runtime().runtime().current_world())
                .unwrap()
        })
        .unwrap();
    let contract = installed.bind_definition(definition).unwrap();
    let scope = session.fresh_scope();
    let outcome = session
        .request(&scope)
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&key)
        .prepare_workflow_publication(contract, WorkflowDefinitionExpectedPredecessor::Absent)
        .unwrap()
        .execute();
    match outcome {
        WorkflowDefinitionPublicationOutcome::Published(done) => {
            (done.definition().clone(), target)
        }
        other => panic!("session publication: {other:?}"),
    }
}

pub(super) fn start(
    home: &Home,
    session: &Session<'_>,
    definition: PublishedWorkflowDefinitionRef,
    key: u64,
) -> (PublishedWorkflowInstanceRef, Original) {
    let scope = session.fresh_scope();
    let prepared = session
        .request(&scope)
        .mutate(WorkflowInstanceStartIntent {
            input: WorkflowInstanceStartInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&key)
        .prepare_workflow_instance_start(
            home.workflow_vocabulary::<GovernedDocumentWorkflow>()
                .unwrap(),
            definition,
        )
        .unwrap();
    let original = prepared.discovery_intent();
    match prepared.execute() {
        WorkflowInstanceStartOutcome::Started(done) => (done.instance().clone(), original),
        other => panic!("session instance start: {other:?}"),
    }
}

/// Proposes with the guarded operation's original target input.
pub(super) fn propose(
    home: &Home,
    session: &Session<'_>,
    instance: PublishedWorkflowInstanceRef,
    target: &TargetNode,
    key: u64,
) -> PublishedWorkflowProposalRef {
    let scope = session.fresh_scope();
    let request = session.request(&scope).on_branch(instance.branch());
    let guarded = request
        .mutate(ReviewedSetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 8,
            },
        })
        .without_source()
        .idempotency_owned(key + 1)
        .unwrap_or_else(|_| panic!("the guarded target description"));
    let inputs = WorthQueryWorkflowOperationInputs::new(home.runtime())
        .with_at::<document::programs::ReviewedAction, _>(target, guarded)
        .unwrap_or_else(|_| panic!("the installed guarded target action"));
    let outcome = request
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&key)
        .prepare_workflow_proposal_with_inputs(
            home.workflow_vocabulary::<GovernedDocumentWorkflow>()
                .unwrap(),
            instance,
            &inputs,
        )
        .unwrap()
        .execute();
    match outcome {
        WorkflowProposalOutcome::Published(done) => done.proposal().clone(),
        other => panic!("session proposal: {other:?}"),
    }
}

pub(super) fn advance(
    home: &Home,
    session: &Session<'_>,
    instance: PublishedWorkflowInstanceRef,
    key: u64,
) -> WorkflowProgressOutcome {
    let scope = session.fresh_scope();
    session
        .request(&scope)
        .on_branch(instance.branch())
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&key)
        .prepare_workflow_advance(
            home.workflow_vocabulary::<GovernedDocumentWorkflow>()
                .unwrap(),
            instance,
        )
        .unwrap()
        .execute()
}

/// Signs and records the reviewer session's approval.
pub(super) fn approve(
    home: &Home,
    reviewer: &Session<'_>,
    authentication: &CertificationAuthenticationOwner,
    instance: PublishedWorkflowInstanceRef,
    required: &RequiredWorkflowApproval,
    proposal: &PublishedWorkflowProposalRef,
    key: u64,
) -> WorkflowProgressOutcome {
    let scope = reviewer.fresh_scope();
    let signing = reviewer
        .request(&scope)
        .on_branch(instance.branch())
        .mutate(WorkflowApprovalIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&key)
        .prepare_workflow_approval(
            home.workflow_vocabulary::<GovernedDocumentWorkflow>()
                .unwrap(),
            instance,
            required,
            proposal,
            WorkflowApprovalDecision::Approve,
        )
        .unwrap();
    let challenge = signing
        .authentication_intent()
        .cloned()
        .expect("a fresh approval key requires a fresh authentication event");
    let event = block_on(authentication.authenticate((), reviewer.principal(), challenge, &scope))
        .expect("the installed certification factor accepts the exact approval challenge");
    signing.sign(&event).unwrap().execute()
}
