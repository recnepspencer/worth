use super::*;

#[test]
fn owned_request_response_preserves_every_native_policy_family_after_serde_roundtrip() {
    let duration = TemporalDuration::temporal_duration(5).unwrap();
    let policies = BridgeOwnedAsyncResourcePolicies {
        lifecycle: ResourceLifecyclePolicyDeclaration::default(),
        retry: ResourceRetryPolicyDeclaration::ExponentialBackoff {
            initial_delay: duration,
            multiplier: 2,
        },
        timeout: ResourceTimeoutPolicyDeclaration::TotalRequestLifetimeTimeout {
            timeout: duration,
        },
        cancellation: ResourceCancellationPolicyDeclaration::RuntimeDenialOnly,
        stale_after: ResourceStaleAfterPolicyDeclaration::RuntimeStaleAfter {
            stale_after: duration,
        },
        supersession:
            ResourceSupersessionPolicyDeclaration::OverlappingGenerationRetainsOldHostWork,
        revalidation: ResourceRevalidationPolicyDeclaration::ExplicitOrDependencyChange,
        observation: ResourceObservationPolicyDeclaration::LifecycleOutputAndDeniedCompletion,
        output_continuity: ResourceOutputContinuityPolicyDeclaration::HideWhilePending,
        retention: ResourceRetentionPolicyDeclaration::RetainAllTransitions,
        diagnostics: ResourceDiagnosticsPolicyDeclaration::RetainedOnly,
        replay: ResourceReplayPolicyDeclaration::IdenticalOnly,
    };
    let original =
        BridgeOwnedAsyncRequestResponseDeclaration::new("resource:quote", 42, 64, 2, 3, 4)
            .with_resource_policies(policies.clone());
    let bytes = serde_json::to_vec(&original.description()).unwrap();
    let description: BridgeOwnedAsyncDeclarationDescription =
        serde_json::from_slice(&bytes).unwrap();
    assert_eq!(description.policies.as_ref(), Some(&policies));
    let installed = BridgeOwnedAsyncRequestResponseDeclaration::from_description(description)
        .signal_declaration(NodeId::new(0, 0))
        .unwrap();
    assert_eq!(installed.lifecycle_policy(), policies.lifecycle);
    assert_eq!(installed.retry_policy(), &policies.retry);
    assert_eq!(installed.timeout_policy(), &policies.timeout);
    assert_eq!(installed.cancellation_policy(), &policies.cancellation);
    assert_eq!(installed.stale_after_policy(), &policies.stale_after);
    assert_eq!(installed.supersession_policy(), &policies.supersession);
    assert_eq!(installed.revalidation_policy(), &policies.revalidation);
    assert_eq!(installed.observation_policy(), &policies.observation);
    assert_eq!(
        installed.output_continuity_policy(),
        &policies.output_continuity
    );
    assert_eq!(installed.retention_policy(), &policies.retention);
    assert_eq!(installed.diagnostics_policy(), &policies.diagnostics);
    assert_eq!(installed.replay_policy(), &policies.replay);
}

#[test]
fn missing_policy_section_uses_the_original_safe_owned_declaration() {
    let previous_wire = serde_json::json!({
        "identity": "resource:quote",
        "payload_contract": 42,
        "max_payload_bytes": 64,
        "retry_max_attempts": 2,
        "retry_delay_ticks": 3,
        "timeout_ticks": 4,
        "managed_queue_capacity": null,
        "retention": "Mandatory"
    });
    let description = serde_json::from_value(previous_wire).unwrap();
    assert_eq!(
        BridgeOwnedAsyncRequestResponseDeclaration::from_description(description),
        BridgeOwnedAsyncRequestResponseDeclaration::new("resource:quote", 42, 64, 2, 3, 4)
    );
}
