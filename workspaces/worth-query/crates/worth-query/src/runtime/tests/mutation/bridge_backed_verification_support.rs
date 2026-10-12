use super::super::support::*;

fn task_relation_runtime(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRuntime {
    stateful_bridge_runtime_with_support(
        request_execution,
        &["Task", "TaskRelation"],
        admitted_profile("direct_entity_identity")
            .with_bridge_backed_verification_support(
                "verify_existing",
                "direct_relation_identity",
                true,
                true,
                None,
            )
            .with_bridge_backed_verification_support(
                "probe_existing",
                "direct_relation_identity",
                true,
                true,
                None,
            )
            .with_bridge_backed_verification_support(
                "update_existing_verified",
                "direct_relation_identity",
                true,
                true,
                None,
            )
            .with_bridge_backed_verification_support(
                "delete_existing_verified",
                "direct_relation_identity",
                true,
                true,
                None,
            ),
    )
}

fn admitted_profile(target_binding_family: &str) -> WorthQueryRuntimeSupportProfile {
    [
        "verify_existing",
        "probe_existing",
        "update_existing_verified",
        "delete_existing_verified",
    ]
    .into_iter()
    .fold(
        WorthQueryRuntimeSupportProfile::bridge_backed(
            "test-subscription-activation",
            "test-preview-basis",
            "test-inspector-evidence",
        ),
        |profile, operation_family| {
            profile.with_bridge_backed_verification_support(
                operation_family,
                target_binding_family,
                true,
                true,
                None,
            )
        },
    )
}

fn verification_row<'a>(
    support: &'a WorthQueryAuthoritativeMutationEvidenceSupport,
    operation_family: &str,
    target_binding_family: &str,
) -> &'a WorthQueryBridgeBackedVerificationSupportRow {
    support
        .bridge_backed_verification_support_rows()
        .iter()
        .find(|row| {
            row.operation_family() == operation_family
                && row.target_binding_family() == target_binding_family
        })
        .expect("support row should exist")
}

mod entity_verification;
mod relation_verification;
