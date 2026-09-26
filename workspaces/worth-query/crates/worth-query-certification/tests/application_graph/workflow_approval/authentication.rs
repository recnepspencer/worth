use std::time::Duration;
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
fn approval_rejects_foreign_owner_wrong_purpose_and_intent_but_replays_without_reuse() {
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
        *intent.subject_coverage(),
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

    let wrong_purpose_signing = prepare!(instance.clone(), 24_013_u64).unwrap();
    let expected = wrong_purpose_signing.authentication_intent().unwrap();
    let wrong_purpose = WorthQueryAuthenticationEventIntent::new(
        "certification-account-login",
        *expected.signing_intent(),
        *expected.subject_coverage(),
    )
    .unwrap();
    let login_event = block_on(application.authentication().authenticate(
        (),
        &principal,
        wrong_purpose,
        &scope,
    ))
    .expect("the same installed owner can authenticate a login without authorizing a signature");
    assert!(matches!(
        wrong_purpose_signing.sign(&login_event),
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

#[test]
fn approval_rejects_expired_signing_event_from_the_installed_named_clock() {
    let (application, _, instance, proposal, required, _) =
        approval_journey("expired-signing-event", 24_100);
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let prepare = |key: u64| {
        runtime
            .request(&principal, &scope)
            .mutate(WorkflowApprovalIntent {
                input: WorkflowAdvanceInput {
                    part_identity: PART_IDENTITY.to_owned(),
                },
            })
            .without_source()
            .idempotency(&key)
            .prepare_workflow_approval(
                &application,
                instance.clone(),
                &required,
                &proposal,
                WorkflowApprovalDecision::Approve,
            )
            .expect("the current approval prepares")
    };
    let stale_signing = prepare(24_110);
    let stale_event = block_on(application.authentication().authenticate(
        (),
        &principal,
        stale_signing.authentication_intent().unwrap().clone(),
        &scope,
    ))
    .expect("the installed authentication owner issues the intended event");
    application.advance_authentication_clock_for_test(Duration::from_secs(61));
    assert!(matches!(
        stale_signing.sign(&stale_event),
        Err(WorthQueryWorkflowAdvancePreparationDenial::Authentication(
            worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventDenial::EventExpired
        ))
    ));

    let fresh_signing = prepare(24_111);
    let fresh_event = block_on(application.authentication().authenticate(
        (),
        &principal,
        fresh_signing.authentication_intent().unwrap().clone(),
        &scope,
    ))
    .expect("a fresh event issues under the advanced named clock");
    assert!(matches!(
        fresh_signing.sign(&fresh_event).unwrap().execute(),
        WorkflowProgressOutcome::Completed(performed) if !performed.replayed()
    ));
}
