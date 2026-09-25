use worth_foundational::facade::CanonicalDigestId;
use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventIntent;
use worth_query_host::facade::application_entry::WorkflowProgressOutcome;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryWorkflowAdvancePreparationDenial,
};

use super::super::bounded_dimension_model::{
    dimension_entry::PART_IDENTITY,
    operator_identity::{authenticate_operator, block_on, request_scope},
    workflow::{
        install_certification_authentication, WorkflowAdvanceInput, WorkflowApprovalIntent,
    },
};
use super::*;

#[test]
fn approval_rejects_foreign_owner_and_wrong_signing_intent_but_replays_without_reuse() {
    let (application, _, instance, proposal, required, _) =
        approval_journey("authenticated", 24_000);
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    macro_rules! prepare {
        ($instance:expr, $key:expr) => {{
            runtime
                .request(&principal, &scope)
                .mutate(WorkflowApprovalIntent {
                    input: WorkflowAdvanceInput {
                        part_identity: PART_IDENTITY.to_owned(),
                    },
                })
                .without_source()
                .idempotency(&$key)
                .prepare_workflow_approval(
                    &application,
                    $instance,
                    &required,
                    &proposal,
                    WorkflowApprovalDecision::Approve,
                )
        }};
    }
    let foreign_signing = prepare!(instance.clone(), 24_010_u64).unwrap();
    let intent = foreign_signing.authentication_intent().unwrap().clone();
    let foreign = install_certification_authentication(runtime.installed_schema());
    let foreign_event =
        block_on(foreign.authenticate((), &principal, intent.clone(), &scope)).unwrap();
    assert!(matches!(
        foreign_signing.sign(&foreign_event),
        Err(WorthQueryWorkflowAdvancePreparationDenial::Authentication(
            worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventDenial::WrongOwner
        ))
    ));

    let wrong_intent = WorthQueryAuthenticationEventIntent::new(
        "workflow-approval-signature",
        CanonicalDigestId::new([0; 32]),
        intent.subject_coverage().clone(),
    )
    .unwrap();
    let wrong_event = block_on(application.authentication().authenticate(
        (),
        &principal,
        wrong_intent,
        &scope,
    ))
    .unwrap();
    assert!(matches!(
        prepare!(instance.clone(), 24_011_u64).unwrap().sign(&wrong_event),
        Err(WorthQueryWorkflowAdvancePreparationDenial::Authentication(
            worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventDenial::IntentMismatch
        ))
    ));

    let signing = prepare!(instance.clone(), 24_012_u64).unwrap();
    let intent = signing.authentication_intent().unwrap().clone();
    let event = block_on(
        application
            .authentication()
            .authenticate((), &principal, intent, &scope),
    )
    .unwrap();
    assert!(matches!(
        signing.sign(&event).unwrap().execute(),
        WorkflowProgressOutcome::Completed(_)
    ));
    let replay = prepare!(instance, 24_012_u64).unwrap();
    assert!(replay.authentication_intent().is_none());
    assert!(matches!(
        replay.execute_replay().unwrap(),
        WorkflowProgressOutcome::Completed(performed) if performed.replayed()
    ));
}
