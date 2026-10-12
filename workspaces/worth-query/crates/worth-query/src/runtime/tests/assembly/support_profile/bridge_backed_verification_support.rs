use super::super::super::support::*;

fn support_row<'a>(
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
        .expect("bridge-backed verification support row should exist")
}

#[test]
fn runtime_authoritative_mutation_support_exposes_bridge_backed_verification_rows() {
    let scaffold = WorthQueryRuntime::public_authoritative_mutation_evidence_support_for_posture(
        WorthQueryRuntimeBackendPosture::Scaffold,
    );
    let primary = WorthQueryRuntime::public_authoritative_mutation_evidence_support_for_posture(
        WorthQueryRuntimeBackendPosture::Primary,
    );

    assert_eq!(scaffold.bridge_backed_verification_support_rows().len(), 8);
    assert_eq!(primary.bridge_backed_verification_support_rows().len(), 8);

    for operation_family in [
        "verify_existing",
        "probe_existing",
        "update_existing_verified",
        "delete_existing_verified",
    ] {
        for target_binding_family in ["direct_entity_identity", "direct_relation_identity"] {
            let scaffold_row = support_row(&scaffold, operation_family, target_binding_family);
            assert_eq!(
                scaffold_row.current_posture_status(),
                WorthQueryBridgeBackedVerificationSupportStatus::Admitted
            );
            assert!(scaffold_row.scaffold_profile_supported());
            assert!(!scaffold_row.primary_bridge_backed_runtime_supported());
            assert_eq!(scaffold_row.denial_class_when_unsupported(), None);

            let primary_row = support_row(&primary, operation_family, target_binding_family);
            assert_eq!(
                primary_row.current_posture_status(),
                WorthQueryBridgeBackedVerificationSupportStatus::Denied
            );
            assert!(primary_row.scaffold_profile_supported());
            assert!(!primary_row.primary_bridge_backed_runtime_supported());
            let expected_denial = if operation_family == "probe_existing" {
                Some("backend_probe_unsupported")
            } else {
                Some("backend_verification_unsupported")
            };
            assert_eq!(primary_row.denial_class_when_unsupported(), expected_denial);
        }
    }
}

#[test]
fn runtime_authoritative_mutation_closeout_carries_bridge_backed_verification_guidance() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.bridge-backed-verification-closeout")
        .expect("workspace should open");
    let closeout = workspace.public_authoritative_mutation_evidence_closeout();

    assert!(closeout.safe_to_build_now().iter().any(
        |line| line.contains("machine-readable by operation family and target-binding family")
    ));
    assert!(closeout
        .must_not_assume_yet()
        .iter()
        .any(|line| line.contains("bridge-backed verified-existing support rows")));
    assert!(closeout
        .migration_guidance()
        .iter()
        .any(|line| line.contains("read bridge-backed verified-existing support rows")));
}

#[test]
fn runtime_authoritative_mutation_support_uses_support_profile_rows_not_only_posture() {
    let profile = WorthQueryRuntimeSupportProfile::bridge_backed(
        "test-subscription-activation",
        "test-preview-basis",
        "test-inspector-evidence",
    )
    .with_bridge_backed_verification_support(
        "probe_existing",
        "direct_relation_identity",
        true,
        true,
        None,
    );

    let support =
        WorthQueryRuntime::public_authoritative_mutation_evidence_support_for_support_profile(
            &profile,
        );
    let admitted = support_row(&support, "probe_existing", "direct_relation_identity");
    let denied = support_row(&support, "verify_existing", "direct_relation_identity");

    assert_eq!(
        admitted.current_posture_status(),
        WorthQueryBridgeBackedVerificationSupportStatus::Admitted
    );
    assert!(admitted.primary_bridge_backed_runtime_supported());
    assert_eq!(admitted.denial_class_when_unsupported(), None);

    assert_eq!(
        denied.current_posture_status(),
        WorthQueryBridgeBackedVerificationSupportStatus::Denied
    );
    assert_eq!(
        denied.denial_class_when_unsupported(),
        Some("backend_verification_unsupported")
    );
}
