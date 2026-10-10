use super::*;

#[test]
fn intent_admission_certification_proof_shape_audit_freezes_shared_phase_progressions() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = certify_intent_admission(request_execution);
    let audit = bundle.proof_shape_audit();

    assert_eq!(
        audit.admitted_phase_progression(),
        &[
            WorthQueryIntentDecisionTraceStage::RawIntent,
            WorthQueryIntentDecisionTraceStage::Eligibility,
            WorthQueryIntentDecisionTraceStage::AdmittedDecision,
            WorthQueryIntentDecisionTraceStage::ExecutionHandoff,
            WorthQueryIntentDecisionTraceStage::ExecutionOutcome,
        ]
    );
    assert_eq!(
        audit.advisory_phase_progression(),
        &[
            WorthQueryIntentDecisionTraceStage::RawIntent,
            WorthQueryIntentDecisionTraceStage::Eligibility,
            WorthQueryIntentDecisionTraceStage::AdvisoryStop,
        ]
    );
    assert_eq!(
        audit.violation_phase_progression(),
        &[
            WorthQueryIntentDecisionTraceStage::RawIntent,
            WorthQueryIntentDecisionTraceStage::Eligibility,
            WorthQueryIntentDecisionTraceStage::ViolationStop,
        ]
    );
}
