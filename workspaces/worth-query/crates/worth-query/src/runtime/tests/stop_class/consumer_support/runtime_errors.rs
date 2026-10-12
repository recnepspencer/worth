use super::super::super::support::*;

pub(in super::super) fn temporal_public_family_admission_error(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace_name: &str,
    reason: &str,
) -> WorthQueryRuntimeError {
    bridge_runtime_with_support(
        request_execution,
        WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
            WorthQueryRuntimeFamilySupport::supported_with_teaching_posture_and_reason(
                WorthQueryRuntimeFacadeFamily::Temporal,
                WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly,
                [WorthQueryAuthorityLane::TemporalExecutionState],
                [],
                ["runtime-backed-temporal-basis-state-inspection"],
                reason,
            ),
        ),
    )
    .workspace(workspace_name)
    .expect("workspace should open")
    .admit_public_api_family(WorthQueryRuntimeFacadeFamily::Temporal)
    .expect_err("support-gated temporal family should fail closed")
}
