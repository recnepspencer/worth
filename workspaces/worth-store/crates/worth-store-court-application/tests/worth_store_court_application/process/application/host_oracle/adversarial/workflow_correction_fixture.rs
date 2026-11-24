//! Shared Governed Workflow (`CorrectableRetentionBinding`) and Authored
//! Correction (`RestoreReviewedRetention`) fixture operations for C4b courts.
use super::{build_id, BUILD_PRIMARY};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_query_decl::facade::application_program::{
    ApplicationWorkflowControlOutcome as Ctrl, ApplicationWorkflowDefinitionBuilder,
    ApplicationWorkflowDefinitionIdentity, ApplicationWorkflowDefinitionReadTarget,
    ApplicationWorkflowEvidenceJoinPolicy, ValidatedWorkflowDefinition,
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_contribution::with_bootstrap_advancement_from_request,
    application_discovery::{
        WorthQueryWorkflowDefinitionDiscovery, WorthQueryWorkflowInstanceRead,
    },
    application_entry::{
        PublishedWorkflowInstanceRef, WorkflowApprovalDecision,
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
        WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationRequestExt,
        WorthQueryWorkflowAssessmentDemandProgress, WorthQueryWorkflowInstanceDiscoveryIntent,
    },
    application_installation::WorthQueryApplicationProgramRoster,
    runtime::ExecutionAllocationPolicy,
    WorthQueryApplicationHomeLocation as Location, WorthQueryApplicationHomeRequest,
    WorthQueryWorkflowApplicationHome,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    actor_generation::validated_actor_program,
    assessment_output::RetentionAssessmentDemand,
    correctable_retention::{CorrectableRetentionBinding, CorrectableRetentionIntent},
    host::{host_limits, seed_host},
    model_lifecycle::validated_model_lifecycle_program,
    operator_identity::{authenticate_operator, authenticate_reviewer, request_scope},
    programs::{
        validated_first_program, validated_removed_operation_program, validated_second_program,
        RetentionProgramP0,
    },
    retention_entry::{DocumentRetentionWritten, SetRetentionDenial, DOCUMENT_IDENTITY},
    schema::{DocumentRetentionQuery, DocumentRetentionSchema as Schema, SetRetentionInput},
    workflow::{
        authoring_intent, definition_limits,
        governed_read::{
            AuditRead, GovernedDocumentWorkflow, MetadataRead, SubjectRead, WorkflowReadInput,
        },
        install_certification_authentication,
        runtime::{install_workflow_spec_for, workflow_resources},
        CertificationAuthenticationOwner, WorkflowAdvanceInput, WorkflowAdvanceIntent,
        WorkflowApprovalCapability, WorkflowApprovalIntent, WorkflowDefinitionAuthoringBinding,
        WorkflowInstanceStartInput, WorkflowInstanceStartIntent,
    },
};
use worth_store_testkit_application_oracle::facade::{
    host_interruption::opening::AdversarialPlacement, host_observation,
};

pub(super) type WorkflowHome =
    WorthQueryWorkflowApplicationHome<Schema, GovernedDocumentWorkflow, RetentionProgramP0>;

const DEF_NAME: &str = "reviewed-document";
const KEY_PUB: u64 = 5001;
const KEY_START: u64 = 5002;
const KEY_PROPOSE: u64 = 5003;
const KEY_JOIN: u64 = 5008;
const KEY_ADV_APPROVAL: u64 = 5009;
const KEY_APPROVE: u64 = 5010;
const KEY_ADV_APPLY: u64 = 5011;
const KEY_APPLY: u64 = 5012;
pub(super) const UPDATED_RETENTION: u64 = 8;

pub(super) fn wf_input() -> WorkflowReadInput {
    WorkflowReadInput { document_identity: DOCUMENT_IDENTITY.to_owned() }
}
fn adv_intent() -> WorkflowAdvanceIntent {
    WorkflowAdvanceIntent {
        input: WorkflowAdvanceInput { document_identity: DOCUMENT_IDENTITY.to_owned() },
    }
}

pub(super) fn open_workflow_home(
    location: &Location,
    placement: Option<AdversarialPlacement>,
    execution: ExecutionRequest<'_, '_>,
) -> (WorkflowHome, CertificationAuthenticationOwner) {
    let owned_location = match location {
        Location::MemoryOnly => Location::MemoryOnly,
        Location::Directory(path) => Location::Directory(path.clone()),
    };
    let mut req = WorthQueryApplicationHomeRequest::new(
        owned_location,
        validated_first_program(),
        WorthQueryApplicationProgramRoster::new()
            .support(validated_second_program())
            .support(validated_removed_operation_program())
            .support(validated_actor_program())
            .support(validated_model_lifecycle_program()),
        Schema::declaration().unwrap(),
        ((),),
        build_id(BUILD_PRIMARY),
        host_limits(WorthQueryAdmissionClock::system()),
        seed_host,
    );
    if let Some(p) = placement {
        req = host_observation::opening::attach_placement(req, p).0;
    }
    let home = *with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(req.open_in_advancement(&phase).unwrap())
    })
    .unwrap();
    let spec = install_workflow_spec_for::<RetentionProgramP0, GovernedDocumentWorkflow>(
        home.runtime().installed_schema(),
        home.runtime().installed_program(),
        workflow_resources(),
    );
    let auth = install_certification_authentication(home.runtime().installed_schema());
    (home.retain_workflow_spec(spec, auth.signing_owner()).unwrap(), auth)
}

fn correctable_workflow_def() -> ValidatedWorkflowDefinition<GovernedDocumentWorkflow> {
    let mut b = ApplicationWorkflowDefinitionBuilder::<GovernedDocumentWorkflow>::new(
        DEF_NAME, definition_limits(),
    ).unwrap();
    let propose = b.operation_binding::<WorkflowDefinitionAuthoringBinding>("propose").unwrap();
    let s1 = b.assessment::<DocumentRetentionQuery>("checks/structural").unwrap();
    let s2 = b.assessment::<DocumentRetentionQuery>("checks/compliance").unwrap();
    let ev = b
        .evidence_join("checks/evidence", ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing)
        .unwrap();
    let approval = b.approval::<WorkflowApprovalCapability>("approval").unwrap();
    let apply = b.operation_binding::<CorrectableRetentionBinding>("apply").unwrap();
    let (done, rej) = (b.terminal("applied").unwrap(), b.terminal("rejected").unwrap());
    b.start(&propose)
        .control(&propose, Ctrl::Completed, &s1)
        .control(&s1, Ctrl::Completed, &s2)
        .control(&s2, Ctrl::Completed, &ev)
        .control(&ev, Ctrl::EvidenceSatisfied, &approval)
        .control(&ev, Ctrl::EvidenceFailed, &rej)
        .control(&approval, Ctrl::Approved, &apply)
        .control(&approval, Ctrl::Rejected, &rej)
        .control(&apply, Ctrl::Completed, &done)
        .proposal_for_assessment(&propose, &s1)
        .proposal_for_assessment(&propose, &s2)
        .assessment_evidence(&s1, &ev)
        .assessment_evidence(&s2, &ev)
        .proposal_for_approval(&propose, &approval)
        .joined_evidence(&ev, &approval)
        .approval_authority(&approval, &apply)
        .operation_input(&propose, &apply);
    b.finish().unwrap().validate().unwrap()
}

pub(super) fn advance(
    home: &WorkflowHome,
    instance: PublishedWorkflowInstanceRef,
    key: u64,
) -> WorkflowProgressOutcome {
    let runtime = home.runtime().runtime();
    let scope = request_scope();
    let operator = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&operator, &scope)
        .on_branch(instance.branch())
        .mutate(adv_intent())
        .without_source()
        .idempotency(&key)
        .prepare_workflow_advance(home.vocabulary(), instance)
        .unwrap()
        .execute()
}

fn run_assessment(
    home: &WorkflowHome,
    instance: PublishedWorkflowInstanceRef,
    assess_key: u64,
    accept_key: u64,
) {
    let runtime = home.runtime().runtime();
    let mut owner = {
        let scope = request_scope();
        let operator = authenticate_operator(runtime.installed_schema(), &scope);
        runtime
            .request(&operator, &scope)
            .on_branch(instance.branch())
            .mutate(adv_intent())
            .without_source()
            .idempotency(&assess_key)
            .prepare_workflow_advance(home.vocabulary(), instance.clone())
            .unwrap()
            .into_assessment_demand(RetentionAssessmentDemand::new(DOCUMENT_IDENTITY))
            .unwrap()
            .retain_operation(home.runtime())
            .unwrap()
    };
    let settlement = loop {
        let scope = request_scope();
        let operator = authenticate_operator(runtime.installed_schema(), &scope);
        let req = runtime.request(&operator, &scope).on_branch(instance.branch());
        match owner.advance(&req).unwrap() {
            WorthQueryWorkflowAssessmentDemandProgress::Settled(settled) => break settled,
            WorthQueryWorkflowAssessmentDemandProgress::Progressed => continue,
            WorthQueryWorkflowAssessmentDemandProgress::Pending => panic!("assessment blocked"),
        }
    };
    let scope = request_scope();
    let operator = authenticate_operator(runtime.installed_schema(), &scope);
    let outcome = runtime
        .request(&operator, &scope)
        .on_branch(instance.branch())
        .mutate(adv_intent())
        .without_source()
        .idempotency(&accept_key)
        .prepare_workflow_advance(home.vocabulary(), instance)
        .unwrap()
        .accept_assessment(&settlement)
        .unwrap();
    assert!(matches!(outcome, WorkflowProgressOutcome::Completed(_)));
}

pub(super) fn seed_approved_workflow(
    home: &WorkflowHome,
    auth: &CertificationAuthenticationOwner,
) -> WorthQueryWorkflowInstanceDiscoveryIntent<Schema> {
    let (instance, original) = publish_and_start_workflow(home);
    approve_workflow(home, auth, instance);
    original
}

fn publish_and_start_workflow(
    home: &WorkflowHome,
) -> (PublishedWorkflowInstanceRef, WorthQueryWorkflowInstanceDiscoveryIntent<Schema>) {
    let runtime = home.runtime().runtime();
    let scope = request_scope();
    let operator = authenticate_operator(runtime.installed_schema(), &scope);
    let installed = home
        .runtime()
        .with_host_advancement(|phase| {
            home.vocabulary().workflow_spec_on(&phase, runtime.current_world()).unwrap()
        })
        .unwrap();
    let contract = installed.bind_definition(correctable_workflow_def()).unwrap();
    let WorkflowDefinitionPublicationOutcome::Published(published) = runtime
        .request(&operator, &scope)
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&KEY_PUB)
        .prepare_workflow_publication(contract, WorkflowDefinitionExpectedPredecessor::Absent)
        .unwrap()
        .execute()
    else {
        panic!("correctable workflow definition must publish");
    };
    let prepared_start = runtime
        .request(&operator, &scope)
        .mutate(WorkflowInstanceStartIntent {
            input: WorkflowInstanceStartInput { document_identity: DOCUMENT_IDENTITY.to_owned() },
        })
        .without_source()
        .idempotency(&KEY_START)
        .prepare_workflow_instance_start(home.vocabulary(), published.definition().clone())
        .unwrap();
    let original = prepared_start.discovery_intent();
    let WorkflowInstanceStartOutcome::Started(started) = prepared_start.execute() else {
        panic!("workflow instance must start");
    };
    (started.instance().clone(), original)
}

fn approve_workflow(
    home: &WorkflowHome,
    auth: &CertificationAuthenticationOwner,
    instance: PublishedWorkflowInstanceRef,
) {
    let runtime = home.runtime().runtime();
    let scope = request_scope();
    let operator = authenticate_operator(runtime.installed_schema(), &scope);
    let WorkflowProposalOutcome::Published(proposed) = runtime
        .request(&operator, &scope)
        .on_branch(instance.branch())
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&KEY_PROPOSE)
        .prepare_workflow_proposal(home.vocabulary(), instance.clone())
        .unwrap()
        .execute()
    else {
        panic!("proposal must publish");
    };
    run_assessment(home, instance.clone(), 5004, 5005);
    run_assessment(home, instance.clone(), 5006, 5007);
    assert!(matches!(advance(home, instance.clone(), KEY_JOIN), WorkflowProgressOutcome::Completed(_)));
    let WorkflowProgressOutcome::AwaitingApproval(required) =
        advance(home, instance.clone(), KEY_ADV_APPROVAL)
    else {
        panic!("workflow must await approval");
    };
    let reviewer = authenticate_reviewer(runtime.installed_schema(), &scope);
    let signing = runtime
        .request(&reviewer, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowApprovalIntent {
            input: WorkflowAdvanceInput { document_identity: DOCUMENT_IDENTITY.to_owned() },
        })
        .without_source()
        .idempotency(&KEY_APPROVE)
        .prepare_workflow_approval(
            home.vocabulary(), instance, &required, proposed.proposal(), WorkflowApprovalDecision::Approve,
        )
        .unwrap();
    let intent = signing.authentication_intent().cloned().unwrap();
    let event = poll_ready(auth.authenticate((), &reviewer, intent, &scope)).unwrap();
    assert!(matches!(signing.sign(&event).unwrap().execute(), WorkflowProgressOutcome::Completed(_)));
}

fn poll_ready<F: std::future::Future>(future: F) -> F::Output {
    struct NoopWake;
    impl std::task::Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }
    let waker = std::task::Waker::from(Arc::new(NoopWake));
    let mut cx = std::task::Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        if let std::task::Poll::Ready(out) = future.as_mut().poll(&mut cx) {
            return out;
        }
        std::thread::yield_now();
    }
}

pub(super) fn discover_original_start(home: &WorkflowHome) -> WorthQueryWorkflowInstanceDiscoveryIntent<Schema> {
    let runtime = home.runtime().runtime();
    let scope = request_scope();
    let operator = authenticate_operator(runtime.installed_schema(), &scope);
    let req = runtime.request(&operator, &scope);
    let basis = req.retain_current_read().unwrap();
    let target = ApplicationWorkflowDefinitionReadTarget::<GovernedDocumentWorkflow>::new(
        ApplicationWorkflowDefinitionIdentity::new(DEF_NAME).unwrap(),
    );
    let found = req
        .at(&basis)
        .discover_workflow_definition::<GovernedDocumentWorkflow, MetadataRead>(
            home.vocabulary(), &target, wf_input(),
        )
        .unwrap();
    let WorthQueryWorkflowDefinitionDiscovery::Current(def) = found.definition() else {
        panic!("published definition must be current");
    };
    runtime
        .request(&operator, &scope)
        .mutate(WorkflowInstanceStartIntent {
            input: WorkflowInstanceStartInput { document_identity: DOCUMENT_IDENTITY.to_owned() },
        })
        .without_source()
        .idempotency(&KEY_START)
        .prepare_workflow_instance_start(home.vocabulary(), def.clone())
        .unwrap()
        .discovery_intent()
}

pub(super) fn run_correctable_step(
    home: &WorkflowHome,
    original: &WorthQueryWorkflowInstanceDiscoveryIntent<Schema>,
) -> Outcome<SetRetentionDenial, DocumentRetentionWritten> {
    let runtime = home.runtime().runtime();
    let scope = request_scope();
    let reviewer = authenticate_reviewer(runtime.installed_schema(), &scope);
    let operator = authenticate_operator(runtime.installed_schema(), &scope);
    let basis = runtime.request(&reviewer, &scope).retain_current_read().unwrap();
    let WorthQueryWorkflowInstanceRead::Live { instance, .. } = runtime
        .request(&reviewer, &scope)
        .at(&basis)
        .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
            home.vocabulary(), original, wf_input(), wf_input(),
        )
        .unwrap()
    else {
        panic!("workflow instance must remain live before completion");
    };
    drop(basis);
    let WorkflowProgressOutcome::AwaitingOperation(required) =
        advance(home, instance.clone(), KEY_ADV_APPLY)
    else { panic!("approved workflow must await CorrectableRetention") };
    runtime
        .request(&operator, &scope)
        .on_branch(instance.branch())
        .mutate(CorrectableRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: UPDATED_RETENTION,
            },
        })
        .without_source()
        .idempotency(&KEY_APPLY)
        .for_workflow_operation(home.vocabulary(), &required).unwrap()
        .advanced()
        .execute_in_program(home.runtime(), ExecutionAllocationPolicy::SystemAllocation).unwrap()
}
