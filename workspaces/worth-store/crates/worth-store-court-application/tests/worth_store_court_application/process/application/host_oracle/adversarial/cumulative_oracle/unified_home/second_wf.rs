//! Starts a second approved `GovernedDocumentWorkflow` instance on the already-published
//! `"reviewed-document"` definition after Instance #1 is `Historical` (and post-D14),
//! executes `CorrectableRetentionBinding`, and advances Instance #2 to `Historical`.
use super::super::super::workflow_correction_fixture::{
    advance, wf_input, WorkflowHome, UPDATED_RETENTION,
};
use std::sync::Arc;
use worth_query_decl::facade::application_program::{
    ApplicationWorkflowDefinitionIdentity, ApplicationWorkflowDefinitionReadTarget,
};
use worth_query_host::facade::{
    application_discovery::{
        WorthQueryWorkflowDefinitionDiscovery, WorthQueryWorkflowInstanceRead,
    },
    application_entry::{
        PublishedWorkflowInstanceRef, WorkflowApprovalDecision, WorkflowInstanceStartOutcome,
        WorkflowProgressOutcome, WorkflowProposalOutcome,
        WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationRequestExt,
        WorthQueryWorkflowAssessmentDemandProgress, WorthQueryWorkflowInstanceDiscoveryIntent,
    },
    runtime::ExecutionAllocationPolicy,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    assessment_output::RetentionAssessmentDemand,
    correctable_retention::CorrectableRetentionIntent,
    operator_identity::{authenticate_operator, authenticate_reviewer, request_scope},
    retention_entry::DOCUMENT_IDENTITY,
    schema::{DocumentRetentionSchema as Schema, SetRetentionInput},
    workflow::{
        authoring_intent,
        governed_read::{AuditRead, GovernedDocumentWorkflow, MetadataRead, SubjectRead},
        CertificationAuthenticationOwner, WorkflowAdvanceInput, WorkflowAdvanceIntent,
        WorkflowApprovalIntent, WorkflowInstanceStartInput, WorkflowInstanceStartIntent,
    },
};

const K2_START: u64 = 7002;
const K2_PROPOSE: u64 = 7003;
const K2_JOIN: u64 = 7008;
const K2_ADV_APPROVAL: u64 = 7009;
const K2_APPROVE: u64 = 7010;
const K2_ADV_APPLY: u64 = 7011;
const K2_APPLY: u64 = 7012;
const K2_DONE: u64 = 7013;

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

fn run_assess(wf: &WorkflowHome, inst: PublishedWorkflowInstanceRef, k_run: u64, k_acc: u64) {
    let rt = wf.runtime().runtime();
    let adv = WorkflowAdvanceIntent {
        input: WorkflowAdvanceInput {
            document_identity: DOCUMENT_IDENTITY.to_owned(),
        },
    };
    let mut owner = {
        let sc = request_scope();
        let op = authenticate_operator(rt.installed_schema(), &sc);
        rt.request(&op, &sc)
            .on_branch(inst.branch())
            .mutate(adv.clone())
            .without_source()
            .idempotency(&k_run)
            .prepare_workflow_advance(wf.workflow_vocabulary::<GovernedDocumentWorkflow>().unwrap(), inst.clone())
            .unwrap()
            .into_assessment_demand(RetentionAssessmentDemand::new(DOCUMENT_IDENTITY))
            .unwrap()
            .retain_operation(wf.runtime())
            .unwrap()
    };
    let settled = loop {
        let sc = request_scope();
        let op = authenticate_operator(rt.installed_schema(), &sc);
        let req = rt.request(&op, &sc).on_branch(inst.branch());
        match owner.advance(&req).unwrap() {
            WorthQueryWorkflowAssessmentDemandProgress::Settled(s) => break s,
            WorthQueryWorkflowAssessmentDemandProgress::Progressed => continue,
            WorthQueryWorkflowAssessmentDemandProgress::Pending => panic!("assess2 blocked"),
        }
    };
    let sc = request_scope();
    let op = authenticate_operator(rt.installed_schema(), &sc);
    assert!(matches!(
        rt.request(&op, &sc)
            .on_branch(inst.branch())
            .mutate(adv)
            .without_source()
            .idempotency(&k_acc)
            .prepare_workflow_advance(wf.workflow_vocabulary::<GovernedDocumentWorkflow>().unwrap(), inst)
            .unwrap()
            .accept_assessment(&settled)
            .unwrap(),
        WorkflowProgressOutcome::Completed(_)
    ));
}

pub(super) fn run_second_wf_to_historical(
    wf: &WorkflowHome,
    auth: &CertificationAuthenticationOwner,
) -> WorthQueryWorkflowInstanceDiscoveryIntent<Schema> {
    let rt = wf.runtime().runtime();
    let sc = request_scope();
    let op = authenticate_operator(rt.installed_schema(), &sc);
    let rev = authenticate_reviewer(rt.installed_schema(), &sc);
    let basis = rt.request(&op, &sc).retain_current_read().unwrap();
    let target = ApplicationWorkflowDefinitionReadTarget::<GovernedDocumentWorkflow>::new(
        ApplicationWorkflowDefinitionIdentity::new("reviewed-document").unwrap(),
    );
    let found = rt
        .request(&op, &sc)
        .at(&basis)
        .discover_workflow_definition::<GovernedDocumentWorkflow, MetadataRead>(
            wf.workflow_vocabulary::<GovernedDocumentWorkflow>().unwrap(),
            &target,
            wf_input(),
        )
        .unwrap();
    let WorthQueryWorkflowDefinitionDiscovery::Current(def) = found.definition() else {
        panic!("published definition must be current for instance #2");
    };
    let def = def.clone();
    drop(basis);

    let prep_start = rt
        .request(&op, &sc)
        .mutate(WorkflowInstanceStartIntent {
            input: WorkflowInstanceStartInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&K2_START)
        .prepare_workflow_instance_start(wf.workflow_vocabulary::<GovernedDocumentWorkflow>().unwrap(), def)
        .unwrap();
    let orig2 = prep_start.discovery_intent();
    let WorkflowInstanceStartOutcome::Started(started) = prep_start.execute() else {
        panic!("instance #2 must start");
    };
    let inst = started.instance().clone();

    let WorkflowProposalOutcome::Published(proposed) = rt
        .request(&op, &sc)
        .on_branch(inst.branch())
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&K2_PROPOSE)
        .prepare_workflow_proposal(wf.workflow_vocabulary::<GovernedDocumentWorkflow>().unwrap(), inst.clone())
        .unwrap()
        .execute()
    else {
        panic!("instance #2 proposal must publish");
    };
    run_assess(wf, inst.clone(), 7004, 7005);
    run_assess(wf, inst.clone(), 7006, 7007);
    assert!(matches!(
        advance(wf, inst.clone(), K2_JOIN),
        WorkflowProgressOutcome::Completed(_)
    ));
    let WorkflowProgressOutcome::AwaitingApproval(req_app) =
        advance(wf, inst.clone(), K2_ADV_APPROVAL)
    else {
        panic!("instance #2 must await approval");
    };
    let signing = rt
        .request(&rev, &sc)
        .on_branch(inst.branch())
        .mutate(WorkflowApprovalIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&K2_APPROVE)
        .prepare_workflow_approval(
            wf.workflow_vocabulary::<GovernedDocumentWorkflow>().unwrap(),
            inst.clone(),
            &req_app,
            proposed.proposal(),
            WorkflowApprovalDecision::Approve,
        )
        .unwrap();
    let intent = signing.authentication_intent().cloned().unwrap();
    let ev = poll_ready(auth.authenticate((), &rev, intent, &sc)).unwrap();
    assert!(matches!(
        signing.sign(&ev).unwrap().execute(),
        WorkflowProgressOutcome::Completed(_)
    ));

    let WorkflowProgressOutcome::AwaitingOperation(req_op) =
        advance(wf, inst.clone(), K2_ADV_APPLY)
    else {
        panic!("instance #2 must await CorrectableRetention");
    };
    let Outcome::Committed { .. } = rt
        .request(&op, &sc)
        .on_branch(inst.branch())
        .mutate(CorrectableRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: UPDATED_RETENTION,
            },
        })
        .without_source()
        .idempotency(&K2_APPLY)
        .for_workflow_operation(wf.workflow_vocabulary::<GovernedDocumentWorkflow>().unwrap(), &req_op)
        .unwrap()
        .advanced()
        .execute_in_program(wf.runtime(), ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    else {
        panic!("instance #2 CorrectableRetention must commit");
    };

    let b2 = rt.request(&rev, &sc).retain_current_read().unwrap();
    let WorthQueryWorkflowInstanceRead::Live {
        instance: inst2, ..
    } = rt
        .request(&rev, &sc)
        .at(&b2)
        .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
            wf.workflow_vocabulary::<GovernedDocumentWorkflow>().unwrap(),
            &orig2,
            wf_input(),
            wf_input(),
        )
        .unwrap()
    else {
        panic!("instance #2 must remain Live before terminal advance");
    };
    drop(b2);
    assert!(matches!(
        advance(wf, inst2, K2_DONE),
        WorkflowProgressOutcome::Completed(_)
    ));
    orig2
}
