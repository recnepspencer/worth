use crate::application::{
    WorthQueryDeclarationEntryContributionCompositionFailureClass,
    WorthQueryDeclarationEntryContributionEvidence,
    WorthQueryDeclarationEntryContributionEvidenceSet, WorthQueryDeclarationEntryInspectionInput,
    WorthQueryDeclarationEntryReadinessRequest, WorthQueryDeclarationEntryRetainedSubjectInput,
};

use super::support::{
    admitted_branch_plan, admitted_declaration_support, admitted_declaration_workflow,
    admitted_lower_runtime_explanation, admitted_plan, admitted_plan_support,
    bridge_signal_envelope, handle, lower_runtime_envelope, BridgeSignalFamily, Input,
};

#[test]
fn subject_aware_readiness_rejects_mismatched_declaration_digest() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "preview");
    let error = match handle.try_declaration_entry_readiness::<Input<BridgeSignalFamily>>(
        WorthQueryDeclarationEntryReadinessRequest::base()
            .for_retained_subject(
                WorthQueryDeclarationEntryRetainedSubjectInput::envelope_checked(
                    crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(
                        bridge_signal_envelope(&handle, "edge:42"),
                    ),
                ),
            )
            .with_contribution_evidence(WorthQueryDeclarationEntryContributionEvidenceSet::new(
                vec![WorthQueryDeclarationEntryContributionEvidence::from(
                    admitted_declaration_support("wrong:digest", "support", "mismatch"),
                )],
            )),
    ) {
        Ok(_) => panic!("subject-aware readiness should reject mismatched digest"),
        Err(error) => error,
    };
    assert_eq!(
        error.failure_class(),
        WorthQueryDeclarationEntryContributionCompositionFailureClass::TargetDigestMismatch
    );
}

#[test]
fn subject_aware_readiness_rejects_wrong_handle_subject() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let source = handle(request_execution, "source");
    let target = handle(request_execution, "target");
    let error = match target.try_declaration_entry_readiness::<Input<BridgeSignalFamily>>(
        WorthQueryDeclarationEntryReadinessRequest::base().for_retained_subject(
            WorthQueryDeclarationEntryRetainedSubjectInput::envelope_checked(
                crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(
                    bridge_signal_envelope(&source, "edge:42"),
                ),
            ),
        ),
    ) {
        Ok(_) => panic!("wrong-handle readiness subject should deny"),
        Err(error) => error,
    };
    assert_eq!(
        error.failure_class(),
        WorthQueryDeclarationEntryContributionCompositionFailureClass::RetainedSubjectMismatch
    );
}

#[test]
fn workflow_and_plan_bound_evidence_fail_closed() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "preview");
    let declaration_digest = bridge_signal_envelope(&handle, "edge:42")
        .declaration_digest()
        .to_string();
    let workflow_error = match handle.inspect_declaration_entry(
        WorthQueryDeclarationEntryInspectionInput::envelope_checked(
            crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(
                bridge_signal_envelope(&handle, "edge:42"),
            ),
        )
        .with_contribution_evidence(
            WorthQueryDeclarationEntryContributionEvidenceSet::new(vec![
                WorthQueryDeclarationEntryContributionEvidence::from(
                    admitted_declaration_workflow(&declaration_digest, "workflow", "preview-only"),
                ),
            ]),
        ),
    ) {
        Ok(_) => panic!("workflow evidence should fail closed"),
        Err(error) => error,
    };
    let workflow_error = workflow_error
        .contribution_composition_error()
        .expect("workflow mismatch should be a composition error");
    assert_eq!(
        workflow_error.failure_class(),
        WorthQueryDeclarationEntryContributionCompositionFailureClass::TargetDigestMismatch
    );

    let readiness_error = match handle.try_declaration_entry_readiness::<Input<BridgeSignalFamily>>(
        WorthQueryDeclarationEntryReadinessRequest::base().with_contribution_evidence(
            WorthQueryDeclarationEntryContributionEvidenceSet::new(vec![
                WorthQueryDeclarationEntryContributionEvidence::from(admitted_plan_support(
                    &admitted_plan(),
                    "support",
                    "plan-bound",
                )),
            ]),
        ),
    ) {
        Ok(_) => panic!("plan-bound readiness evidence should fail closed"),
        Err(error) => error,
    };
    assert_eq!(
        readiness_error.failure_class(),
        WorthQueryDeclarationEntryContributionCompositionFailureClass::TargetFamilyTooStrong
    );
}

#[test]
fn admitted_plan_bound_evidence_rejects_mismatched_retained_plan_scope() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "preview");
    let retained_plan = admitted_plan();
    let evidence_plan = admitted_branch_plan("branch:mismatched-plan");
    let error = match handle.try_declaration_entry_readiness::<Input<BridgeSignalFamily>>(
        WorthQueryDeclarationEntryReadinessRequest::base()
            .for_retained_subject(
                WorthQueryDeclarationEntryRetainedSubjectInput::envelope_checked(
                    crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(
                        bridge_signal_envelope(&handle, "edge:42"),
                    ),
                ),
            )
            .with_admitted_plan_scope(retained_plan)
            .with_contribution_evidence(WorthQueryDeclarationEntryContributionEvidenceSet::new(
                vec![WorthQueryDeclarationEntryContributionEvidence::from(
                    admitted_plan_support(&evidence_plan, "support", "wrong-plan"),
                )],
            )),
    ) {
        Ok(_) => panic!("mismatched admitted-plan proof should deny"),
        Err(error) => error,
    };
    assert_eq!(
        error.failure_class(),
        WorthQueryDeclarationEntryContributionCompositionFailureClass::TargetDigestMismatch
    );
}

#[test]
fn lower_runtime_bound_evidence_rejects_mismatched_retained_lower_runtime_scope() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "preview");
    let retained_lower_runtime = lower_runtime_envelope("lower-runtime:retained");
    let evidence_lower_runtime = lower_runtime_envelope("lower-runtime:evidence");
    let error = match handle.inspect_declaration_entry(
        WorthQueryDeclarationEntryInspectionInput::envelope_checked(
            crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(
                bridge_signal_envelope(&handle, "edge:42"),
            ),
        )
        .with_lower_runtime_boundary_scope(retained_lower_runtime)
        .with_contribution_evidence(
            WorthQueryDeclarationEntryContributionEvidenceSet::new(vec![
                WorthQueryDeclarationEntryContributionEvidence::from(
                    admitted_lower_runtime_explanation(
                        &evidence_lower_runtime,
                        "explain",
                        "wrong-lower-runtime",
                    ),
                ),
            ]),
        ),
    ) {
        Ok(_) => panic!("mismatched lower-runtime proof should deny"),
        Err(error) => error,
    };
    let error = error
        .contribution_composition_error()
        .expect("lower-runtime mismatch should surface as contribution error");
    assert_eq!(
        error.failure_class(),
        WorthQueryDeclarationEntryContributionCompositionFailureClass::TargetDigestMismatch
    );
}
