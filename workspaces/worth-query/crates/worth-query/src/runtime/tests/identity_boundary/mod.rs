use super::support::*;

#[test]
fn evidence_identity_resists_joined_string_folklore() {
    let authority = crate::runtime::WorthQueryRuntimeEvidenceAuthority::new();
    let left = crate::runtime::WorthQueryPreviewBasisAdmission::new(
        &authority,
        test_session_label("preview|basis"),
        WorthQueryEffectPolicy::SandboxedWriteIntent,
        crate::runtime::WorthQueryBasisAdmissionEvidenceRow::rows_from_values([
            "alpha",
            "beta|gamma",
        ]),
    );
    let right = crate::runtime::WorthQueryPreviewBasisAdmission::new(
        &authority,
        test_session_label("preview"),
        WorthQueryEffectPolicy::SandboxedWriteIntent,
        crate::runtime::WorthQueryBasisAdmissionEvidenceRow::rows_from_values([
            "basis|alpha",
            "beta|gamma",
        ]),
    );

    assert_ne!(
        left.admission_digest(),
        right.admission_digest(),
        "canonical evidence identity must resist joined-string delimiter collisions"
    );
}

#[test]
fn stop_class_remains_typed_under_message_rewording() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let first_error = bridge_runtime_with_support(
        request_execution,
        WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
            WorthQueryRuntimeFamilySupport::supported_with_teaching_posture_and_reason(
                WorthQueryRuntimeFacadeFamily::Temporal,
                WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly,
                [WorthQueryAuthorityLane::TemporalExecutionState],
                [],
                ["runtime-backed-temporal-basis-state-inspection"],
                "first temporal wording",
            ),
        ),
    )
    .workspace("identity-boundary-reword-first")
    .expect("workspace should open")
    .admit_public_api_family(WorthQueryRuntimeFacadeFamily::Temporal)
    .expect_err("temporal admission should fail closed");
    let second_error = bridge_runtime_with_support(
        request_execution,
        WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
            WorthQueryRuntimeFamilySupport::supported_with_teaching_posture_and_reason(
                WorthQueryRuntimeFacadeFamily::Temporal,
                WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly,
                [WorthQueryAuthorityLane::TemporalExecutionState],
                [],
                ["runtime-backed-temporal-basis-state-inspection"],
                "second temporal wording",
            ),
        ),
    )
    .workspace("identity-boundary-reword-second")
    .expect("workspace should open")
    .admit_public_api_family(WorthQueryRuntimeFacadeFamily::Temporal)
    .expect_err("temporal admission should fail closed");

    for error in [&first_error, &second_error] {
        match error.stop_class() {
            WorthQueryStopClass::FamilyAdmissionDenied {
                family,
                status,
                teaching_posture,
                ..
            } => {
                assert_eq!(family, WorthQueryRuntimeFacadeFamily::Temporal);
                assert_eq!(status, WorthQueryRuntimeFamilySupportStatus::Supported);
                assert_eq!(
                    teaching_posture,
                    Some(WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly)
                );
            }
            other => panic!("expected typed family-admission stop class, got {other:?}"),
        }
    }
    assert_ne!(
        first_error.to_string(),
        second_error.to_string(),
        "message wording must remain presentation while stop-class meaning stays stable"
    );
}

#[test]
fn session_label_identity_holds_under_collision_pressure() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = bridge_runtime_with_support_and_intent_authority(
        request_execution,
        intent_support_profile(),
        TestIntentAuthority,
    );
    let preview_label =
        WorthQuerySessionLabel::scoped_strs("worth.kernel", ["preview"]).expect("label");
    let render_collision =
        WorthQuerySessionLabel::scoped_strs("worth", ["kernel", "preview"]).expect("label");

    runtime
        .preview(request_execution, preview_label.clone())
        .expect("first preview label should admit")
        .discard();
    runtime
        .preview(request_execution, render_collision.clone())
        .expect("display-colliding but identity-distinct preview label should admit")
        .discard();
    runtime
        .branch(request_execution, preview_label.clone())
        .expect("branch family should admit same identity independently");

    let error = match runtime.branch(request_execution, preview_label.clone()) {
        Ok(_) => panic!("same-family branch replay should collide"),
        Err(error) => error,
    };
    match error.stop_class() {
        WorthQueryStopClass::SessionLabelCollision {
            authority_lane,
            label,
        } => {
            assert_eq!(authority_lane, WorthQueryAuthorityLane::BranchLocalTruth);
            assert_eq!(label, &preview_label);
        }
        other => panic!("expected typed session-label collision, got {other:?}"),
    }

    assert_eq!(preview_label.display(), render_collision.display());
    assert_ne!(
        preview_label.identity_digest(),
        render_collision.identity_digest()
    );
}
