use super::*;

#[test]
fn authoritative_admitted_decision_materializes_family_specific_plan_and_handoff() {
    let request = crate::intent_admission::WorthQueryRawIntentAdmissionRequest::authoritative_runtime_entrypoint(
        WorthQueryIntentDeclaration::strategy_commit(
            "phase-three-authoritative-intent",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            test_intent_input([("entity", "task-1")]),
        ),
    )
    .expect("authoritative request should build");

    match admit_runtime_intent_request(request.clone()) {
        WorthQueryIntentAdmissionDecision::Admitted(
            crate::intent_admission::WorthQueryAdmittedIntentPlan::Authoritative(plan),
        ) => {
            let handoff = WorthQueryAuthoritativeIntentExecutionHandoff::from_plan(plan.clone());
            assert_eq!(plan.family(), request.family());
            assert_eq!(plan.request_digest(), request.request_digest());
            assert_eq!(handoff.family(), request.family());
            assert_eq!(handoff.request_digest(), request.request_digest());
            assert_eq!(handoff.decision_digest(), plan.decision_digest());
        }
        other => panic!("expected authoritative admitted plan, got {other:?}"),
    }
}

#[test]
fn effect_admitted_decision_materializes_family_specific_plan_and_handoff() {
    let request =
        crate::intent_admission::WorthQueryRawIntentAdmissionRequest::effect_runtime_entrypoint(
            WorthQueryIntentDeclaration::strategy_commit(
                "phase-three-effect-intent",
                "strategy.intent.reconcile",
                "1.0",
                "intent.reconcile.input.v1",
                test_intent_input([("entity", "task-1")]),
            )
            .with_source_lane(WorthQueryIntentSourceLane::EffectTriggered),
        )
        .expect("effect request should build");

    match admit_runtime_intent_request(request.clone()) {
        WorthQueryIntentAdmissionDecision::Admitted(
            crate::intent_admission::WorthQueryAdmittedIntentPlan::EffectTriggered(plan),
        ) => {
            let handoff = WorthQueryEffectTriggeredIntentExecutionHandoff::from_plan(plan.clone());
            assert_eq!(plan.family(), request.family());
            assert_eq!(plan.request_digest(), request.request_digest());
            assert_eq!(handoff.family(), request.family());
            assert_eq!(handoff.request_digest(), request.request_digest());
            assert_eq!(handoff.decision_digest(), plan.decision_digest());
        }
        other => panic!("expected effect admitted plan, got {other:?}"),
    }
}

#[test]
fn read_admitted_decision_materializes_read_plan_and_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = read_runtime(request_execution);
    let mut workspace =
        WorthQueryWorkspace::new("phase-three-read", runtime).expect("workspace should build");
    let family = identity_read_family(&mut workspace, "tasks");
    let request =
        crate::intent_admission::WorthQueryRawIntentAdmissionRequest::read_family_entrypoint(
            crate::intent_admission::WorthQueryReadExecutionIntentSeed::current_runtime(family),
        )
        .expect("read request should build");

    match admit_runtime_intent_request(request.clone()) {
        WorthQueryIntentAdmissionDecision::Admitted(
            crate::intent_admission::WorthQueryAdmittedIntentPlan::ReadExecution(plan),
        ) => {
            let handoff =
                crate::intent_admission::WorthQueryReadExecutionHandoff::from_plan(plan.clone());
            assert_eq!(plan.family(), request.family());
            assert_eq!(plan.request_digest(), request.request_digest());
            assert_eq!(handoff.family(), request.family());
            assert_eq!(handoff.request_digest(), request.request_digest());
            assert_eq!(handoff.decision_digest(), plan.decision_digest());
        }
        other => panic!("expected read admitted plan, got {other:?}"),
    }
}

#[test]
fn violation_decision_materializes_explicit_stop_artifact() {
    let request = crate::intent_admission::WorthQueryRawIntentAdmissionRequest::authoritative_runtime_entrypoint(
        WorthQueryIntentDeclaration::strategy_commit(
            "phase-three-violation-stop",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            test_intent_input([("entity", "task-1")]),
        )
        .with_target_lane(WorthQueryAuthorityLane::PreviewTruth),
    )
    .expect("request should build");

    let stop = admit_runtime_intent_request(request.clone())
        .into_non_admitted_stop()
        .expect("violation decision should stop");
    assert!(matches!(
        stop,
        WorthQueryIntentNonAdmittedStop::Violation(_)
    ));
    assert_eq!(stop.family(), request.family());
    assert_eq!(stop.request_digest(), request.request_digest());
    assert_eq!(stop.stage(), "authority-admission");
    assert!(!stop.stop_digest().is_empty());
}

#[test]
fn basis_observation_admitted_decision_materializes_scoped_plan_without_handoff() {
    let request =
        crate::intent_admission::WorthQueryRawIntentAdmissionRequest::basis_observation_lane(
            crate::basis_lifecycle::RawBasisIntent::CurrentHead,
        )
        .expect("basis request should build");

    match admit_runtime_intent_request(request.clone()) {
        WorthQueryIntentAdmissionDecision::Admitted(
            crate::intent_admission::WorthQueryAdmittedIntentPlan::BasisObservation(plan),
        ) => {
            assert_eq!(plan.family(), request.family());
            assert_eq!(plan.request_digest(), request.request_digest());
            assert_eq!(plan.execution_seam(), None);
            let _scoped = plan.scope();
        }
        other => panic!("expected basis admitted plan, got {other:?}"),
    }
}

#[test]
fn projection_consumption_admitted_decision_materializes_contract_plan_without_handoff() {
    let declaration = crate::projection_consumption::declare_projection_consumption(
        crate::projection_consumption::ProjectionConsumptionSource::test_only(
            crate::projection_consumption::ProjectionSourceFamily::QueryReadReceipt,
            Some("query-digest"),
            Some("basis-digest"),
            Some("result-digest"),
            Some("shape-digest"),
            "query-read:test",
        ),
        crate::projection_consumption::ProjectionConsumptionBindingContext::test_only_with_projection_metadata(
            "shape-digest",
            "query-digest",
            "shape-digest",
            "projection.identity",
            "narrowed-shape-digest",
            "policy-digest",
            "tenant-schema-digest",
            crate::projection_consumption::test_authorized_field_paths(&["field.visible"]),
        ),
        crate::projection_consumption::ProjectMaterializedFacts::declare()
            .display_field_path(crate::projection_consumption::projection_fact_field_path_from_segments([worth_foundational::facade::FieldKey::new("field").expect("projection fact field segment should admit"), worth_foundational::facade::FieldKey::new("visible").expect("projection fact field segment should admit")])),
    )
    .expect("projection declaration should build");
    let request =
        crate::intent_admission::WorthQueryRawIntentAdmissionRequest::projection_consumption(
            declaration,
        )
        .expect("projection request should build");

    match admit_runtime_intent_request(request.clone()) {
        WorthQueryIntentAdmissionDecision::Admitted(
            crate::intent_admission::WorthQueryAdmittedIntentPlan::ProjectionConsumption(plan),
        ) => {
            assert_eq!(plan.family(), request.family());
            assert_eq!(plan.request_digest(), request.request_digest());
            assert_eq!(plan.execution_seam(), None);
            let _contract = plan.bind_contract();
        }
        other => panic!("expected projection admitted plan, got {other:?}"),
    }
}
