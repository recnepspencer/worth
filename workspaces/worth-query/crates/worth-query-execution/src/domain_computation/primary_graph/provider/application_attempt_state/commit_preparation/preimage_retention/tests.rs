//! Refusal values use the actual retention classifier and pre-effect receiver.
use {super::retention_failure, crate::domain_computation::vocabulary::application_aftermath::retained_preimage::WorthQueryPreImageRetentionDenial as Retention};
use {crate::domain_computation::WorthQueryProviderSessionCommitStop as Stop, crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind as Control, crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind as Kind, crate::domain_computation::WorthQueryProviderSessionProtocolStage as Stage, crate::domain_computation::WorthQueryProviderSessionRecoveryPosture as Posture};
use worth_relational::facade::content::{
    RelationalContentDenial as Content, RelationalContentRejection as Rejection,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

#[test]
fn retention_refusals_keep_exact_cause_and_pre_effect_posture() {
    use crate::domain_computation::vocabulary::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial as Resource;
    for (denial, expected) in [
        (
            Retention::Content(Content::Execution(Cause::WorkExhausted)),
            Kind::ExecutionResource {
                denial: Resource::WorkExhausted,
                partition_identity: None,
                policy_ancestor: None,
            },
        ),
        (
            Retention::Content(Content::SizeLimit {
                observed: 97,
                maximum: 31,
            }),
            Kind::Content(Rejection::SizeLimit {
                observed: 97,
                maximum: 31,
            }),
        ),
    ] {
        let failure = retention_failure(denial);
        assert_eq!(failure.checked_read_stop(), Some(Ok(expected)));
        let Stop::PreEffectDenied(failure) = super::super::preparation_stop(failure) else {
            panic!("pre-transaction retention must refuse without commit recovery");
        };
        assert_eq!(failure.kind(), expected);
        assert_eq!(failure.preimage_retention_denial(), Some(denial));
        assert_eq!(failure.stage(), Stage::Commit);
        assert_eq!(failure.recovery_posture(), Posture::Closed);
    }
    let Stop::ControlStopped(control) = super::super::preparation_stop(retention_failure(
        Retention::Content(Content::Execution(Cause::Cancelled)),
    )) else {
        panic!("original cancellation must retain its preparation control posture");
    };
    assert_eq!(control.kind(), Control::Cancelled);
    let Stop::Denied(failure) =
        super::super::preparation_stop(retention_failure(Retention::EmptyDemand))
    else {
        panic!("semantic retention failure must preserve protocol cleanup");
    };
    assert_eq!(
        failure.preimage_retention_denial(),
        Some(Retention::EmptyDemand)
    );
    assert_eq!(failure.kind(), Kind::ProviderRejected);
    assert_eq!(failure.recovery_posture(), Posture::Closed);
}
