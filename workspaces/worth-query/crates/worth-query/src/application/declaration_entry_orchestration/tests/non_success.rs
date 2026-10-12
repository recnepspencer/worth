use crate::application::{
    WorthQueryDeclarationEntryOrchestrationAutomationRefusalClass,
    WorthQueryDeclarationEntryOrchestrationCostPosture,
    WorthQueryDeclarationEntryOrchestrationMaterializationGate,
    WorthQueryDeclarationEntryOrchestrationOutcome,
    WorthQueryDeclarationEntryOrchestrationRefusalClass,
    WorthQueryDeclarationEntryOrchestrationStage,
    WorthQueryDeclarationEntryOrchestrationStepDisposition,
    WorthQueryDeclarationEntryOrchestrationTerminalError,
    WorthQueryDeclarationReceiptTerminalError, WorthQueryDeclarationRoutePlanDenialCause,
    WorthQueryDeclarationRoutePlanTerminalError,
};

use super::domain::{
    admitted_handle, AdmittedFamily, DeferredFamily, DeferredRouteFamily, DeniedFamily,
    ExpensiveAutomationFamily, ExplicitIntentFamily, FailedFamily, Input, RebindRequiredFamily,
    StaleFamily, UnsupportedReceiptFamily,
};

#[test]
fn progression_non_success_posture_stays_typed() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");

    assert!(matches!(
        handle.orchestrate_declaration_entry_checked(Input::<DeferredFamily>::new("edge:42")),
        WorthQueryDeclarationEntryOrchestrationOutcome::Deferred(_)
    ));
    assert!(matches!(
        handle.orchestrate_declaration_entry_checked(Input::<DeniedFamily>::new("edge:42")),
        WorthQueryDeclarationEntryOrchestrationOutcome::Denied(_)
    ));
    assert!(matches!(
        handle.orchestrate_declaration_entry_checked(Input::<FailedFamily>::new("edge:42")),
        WorthQueryDeclarationEntryOrchestrationOutcome::Failed(_)
    ));
}

#[test]
fn stale_and_rebind_required_remain_distinct() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let collaborative = admitted_handle(request_execution, "collaborative");
    let restricted = admitted_handle(request_execution, "restricted");

    assert!(matches!(
        collaborative.orchestrate_declaration_entry_checked(Input::<StaleFamily>::new("edge:42")),
        WorthQueryDeclarationEntryOrchestrationOutcome::Stale(_)
    ));
    assert!(matches!(
        restricted
            .orchestrate_declaration_entry_checked(Input::<RebindRequiredFamily>::new("edge:42")),
        WorthQueryDeclarationEntryOrchestrationOutcome::RebindRequired(_)
    ));
}

#[test]
fn intent_required_route_is_refused_without_hidden_route_lowering() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let checked =
        handle.orchestrate_declaration_entry_checked(Input::<ExplicitIntentFamily>::new("edge:42"));
    let proof =
        handle.orchestrate_declaration_entry_proof(Input::<ExplicitIntentFamily>::new("edge:42"));

    match checked {
        WorthQueryDeclarationEntryOrchestrationOutcome::Refused(refusal) => {
            assert_eq!(
                refusal.refusal_class(),
                WorthQueryDeclarationEntryOrchestrationRefusalClass::ExplicitIntentRequired
            );
            assert_eq!(
                refusal.automation_refusal_class(),
                WorthQueryDeclarationEntryOrchestrationAutomationRefusalClass::ExplicitIntentRequired
            );
            assert_eq!(
                refusal.stop_stage(),
                WorthQueryDeclarationEntryOrchestrationStage::RoutePlanned
            );
        }
        _ => panic!("expected explicit-intent refusal"),
    }

    let last = proof
        .stage_records()
        .last()
        .expect("stop stage should exist");
    assert_eq!(
        last.stage(),
        WorthQueryDeclarationEntryOrchestrationStage::RoutePlanned
    );
    assert!(last.is_stop());
}

#[test]
fn unsupported_receipt_kind_is_refused_without_hidden_continuation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let checked = handle
        .orchestrate_declaration_entry_checked(Input::<UnsupportedReceiptFamily>::new("edge:42"));
    match checked {
        WorthQueryDeclarationEntryOrchestrationOutcome::Refused(refusal) => {
            assert_eq!(
                refusal.refusal_class(),
                WorthQueryDeclarationEntryOrchestrationRefusalClass::UnsupportedAutomation
            );
            assert_eq!(
                refusal.automation_refusal_class(),
                WorthQueryDeclarationEntryOrchestrationAutomationRefusalClass::UnsupportedAutomation
            );
            assert_eq!(
                refusal.stop_stage(),
                WorthQueryDeclarationEntryOrchestrationStage::ReceiptIssued
            );
        }
        _ => panic!("expected unsupported-automation refusal"),
    }
}

#[test]
fn expensive_but_legal_declaration_refuses_automation_at_route() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let checked = handle
        .orchestrate_declaration_entry_checked(Input::<ExpensiveAutomationFamily>::new("edge:42"));
    let proof = handle
        .orchestrate_declaration_entry_proof(Input::<ExpensiveAutomationFamily>::new("edge:42"));

    match checked {
        WorthQueryDeclarationEntryOrchestrationOutcome::Refused(refusal) => {
            assert_eq!(
                refusal.refusal_class(),
                WorthQueryDeclarationEntryOrchestrationRefusalClass::ExpensiveWorkNotAdmittedByDefault
            );
            assert_eq!(
                refusal.automation_refusal_class(),
                WorthQueryDeclarationEntryOrchestrationAutomationRefusalClass::ExpensiveAutomationForbidden
            );
            assert_eq!(
                refusal.stop_stage(),
                WorthQueryDeclarationEntryOrchestrationStage::RoutePlanned
            );
        }
        _ => panic!("expected expensive automation refusal"),
    }
    let last = proof
        .stage_records()
        .last()
        .expect("stop stage should exist");
    assert_eq!(
        last.stage(),
        WorthQueryDeclarationEntryOrchestrationStage::RoutePlanned
    );
    assert_eq!(
        last.disposition(),
        WorthQueryDeclarationEntryOrchestrationStepDisposition::ExplicitForCaller
    );
    assert_eq!(last.materialization_tier(), None);
}

#[test]
fn expensive_route_can_still_continue_on_the_explicit_path() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");

    let envelope = handle
        .declare_review_progress_describe_plan_receipt_and_envelope(Input::<
            ExpensiveAutomationFamily,
        >::new("edge:42"))
        .unwrap_or_else(|_| panic!("explicit lowering should still envelope"));

    assert_eq!(
        envelope.declaration_family_key(),
        "ExpensiveAutomationFamily"
    );
}

#[test]
fn deferred_route_lowering_records_receipt_stop_honestly() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "collaborative");
    let checked =
        handle.orchestrate_declaration_entry_checked(Input::<DeferredRouteFamily>::new("edge:42"));
    let proof =
        handle.orchestrate_declaration_entry_proof(Input::<DeferredRouteFamily>::new("edge:42"));

    match checked {
        WorthQueryDeclarationEntryOrchestrationOutcome::Deferred(outcome) => {
            assert_eq!(
                outcome.stop_stage(),
                WorthQueryDeclarationEntryOrchestrationStage::ReceiptIssued
            );
        }
        _ => panic!("expected deferred route outcome"),
    }

    let route = proof
        .stage_records()
        .iter()
        .find(|record| record.stage() == WorthQueryDeclarationEntryOrchestrationStage::RoutePlanned)
        .expect("route stage should exist");
    assert!(route.is_reached());

    let last = proof
        .stage_records()
        .last()
        .expect("receipt stop should exist");
    assert_eq!(
        last.stage(),
        WorthQueryDeclarationEntryOrchestrationStage::ReceiptIssued
    );
    assert!(last.is_stop());
}

#[test]
fn ordinary_surface_preserves_terminal_error_classes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "restricted");

    assert!(matches!(
        handle.orchestrate_declaration_entry(Input::<RebindRequiredFamily>::new("edge:42")),
        Err(WorthQueryDeclarationEntryOrchestrationTerminalError::RebindRequired(_))
    ));
    assert!(matches!(
        handle.orchestrate_declaration_entry(Input::<ExplicitIntentFamily>::new("edge:42")),
        Err(WorthQueryDeclarationEntryOrchestrationTerminalError::Refused(_))
    ));
}

#[test]
fn refusal_taxonomies_remain_publicly_nameable_even_before_all_classes_are_reached() {
    let broad_classes = [
        WorthQueryDeclarationEntryOrchestrationRefusalClass::UnsupportedAutomation,
        WorthQueryDeclarationEntryOrchestrationRefusalClass::ExplicitIntentRequired,
        WorthQueryDeclarationEntryOrchestrationRefusalClass::StrongerProofRequired,
        WorthQueryDeclarationEntryOrchestrationRefusalClass::AuthorityTransitionRequired,
        WorthQueryDeclarationEntryOrchestrationRefusalClass::ExpensiveWorkNotAdmittedByDefault,
        WorthQueryDeclarationEntryOrchestrationRefusalClass::PreparedButNotExecutedContinuation,
    ];
    let automation_classes = [
        WorthQueryDeclarationEntryOrchestrationAutomationRefusalClass::ExplicitIntentRequired,
        WorthQueryDeclarationEntryOrchestrationAutomationRefusalClass::ExpensiveAutomationForbidden,
        WorthQueryDeclarationEntryOrchestrationAutomationRefusalClass::AuthorityTransitionRequired,
        WorthQueryDeclarationEntryOrchestrationAutomationRefusalClass::PreparedButNotExecuted,
        WorthQueryDeclarationEntryOrchestrationAutomationRefusalClass::UnsupportedAutomation,
        WorthQueryDeclarationEntryOrchestrationAutomationRefusalClass::StrongerProofRequired,
    ];

    let broad_labels = broad_classes.map(|class| class.as_str());
    let automation_labels = automation_classes.map(|class| class.as_str());
    assert_eq!(
        broad_labels,
        [
            "unsupported_automation",
            "explicit_intent_required",
            "stronger_proof_required",
            "authority_transition_required",
            "expensive_work_not_admitted_by_default",
            "prepared_but_not_executed_continuation",
        ]
    );
    assert_eq!(
        automation_labels,
        [
            "explicit_intent_required",
            "expensive_automation_forbidden",
            "authority_transition_required",
            "prepared_but_not_executed",
            "unsupported_automation",
            "stronger_proof_required",
        ]
    );
}

#[test]
fn materialization_taxonomies_remain_publicly_nameable_even_before_all_gates_are_reached() {
    let cost_labels = [
        WorthQueryDeclarationEntryOrchestrationCostPosture::OrdinaryDefault,
        WorthQueryDeclarationEntryOrchestrationCostPosture::ExplicitlyLean,
        WorthQueryDeclarationEntryOrchestrationCostPosture::ExplicitlyRich,
        WorthQueryDeclarationEntryOrchestrationCostPosture::PreparedButNotExecuted,
        WorthQueryDeclarationEntryOrchestrationCostPosture::ExpensiveByDefault,
    ]
    .map(|posture| posture.as_str());
    let gate_labels = [
        WorthQueryDeclarationEntryOrchestrationMaterializationGate::AdmittedByDefault,
        WorthQueryDeclarationEntryOrchestrationMaterializationGate::ExplicitRequestRequired,
        WorthQueryDeclarationEntryOrchestrationMaterializationGate::ForbiddenOnOrdinaryLane,
        WorthQueryDeclarationEntryOrchestrationMaterializationGate::PreparedOnly,
        WorthQueryDeclarationEntryOrchestrationMaterializationGate::UnsupportedForCurrentArtifactSet,
    ]
    .map(|gate| gate.as_str());

    assert_eq!(
        cost_labels,
        [
            "ordinary_default",
            "explicitly_lean",
            "explicitly_rich",
            "prepared_but_not_executed",
            "expensive_by_default",
        ]
    );
    assert_eq!(
        gate_labels,
        [
            "admitted_by_default",
            "explicit_request_required",
            "forbidden_on_ordinary_lane",
            "prepared_only",
            "unsupported_for_current_artifact_set",
        ]
    );
}

#[test]
fn product_orchestration_denies_wrong_world_progressed_artifacts_without_panicking() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let collaborative = admitted_handle(request_execution, "collaborative");
    let restricted = admitted_handle(request_execution, "restricted");
    let progressed = collaborative
        .declare_review_and_progress(Input::<AdmittedFamily>::new("edge:42"))
        .unwrap_or_else(|_| panic!("progression should admit"));

    assert!(matches!(
        restricted.orchestrate_routes_from_progressed(progressed.clone()),
        Err(WorthQueryDeclarationRoutePlanTerminalError::Denied(denial))
            if denial.cause() == WorthQueryDeclarationRoutePlanDenialCause::WrongAdmittedWorld
    ));
    assert!(matches!(
        restricted.orchestrate_receipt_from_progressed(progressed.clone()),
        Err(WorthQueryDeclarationReceiptTerminalError::Denied(_))
    ));
    assert!(matches!(
        restricted.orchestrate_envelope_from_progressed_checked(progressed),
        crate::application::WorthQueryDeclarationEnvelopeChecked::Denied(_)
    ));
}
