use super::support::*;
use crate::facade::RuntimeBridgeBuilder;
use crate::policy::{
    BridgeArtifactPolicyBaseline, BridgeDiagnosticsPolicyBaseline, BridgeDiagnosticsTier,
    BridgeExecutionPolicyBaseline, BridgeExecutionPolicyClass, BridgeRuntimePolicy,
    BridgeRuntimePosture,
};

#[test]
fn build_accepts_policy_sections_without_losing_canonical_runtime_policy() {
    let runtime = RuntimeBridgeBuilder::new()
        .with_execution_policy_baseline(BridgeExecutionPolicyBaseline::new(
            BridgeExecutionPolicyClass::DeterministicCanonical,
            BridgeRuntimePosture::Development,
        ))
        .with_diagnostics_policy_baseline(
            BridgeDiagnosticsPolicyBaseline::for_tier(BridgeDiagnosticsTier::Standard)
                .with_route_record_limit(17)
                .with_failure_record_limit(9),
        )
        .with_artifact_policy_baseline(BridgeArtifactPolicyBaseline::new(true, false))
        .with_relational_source(TestSource)
        .with_signal_sink(TestSink)
        .register_mapping(exact_registration("user-profile-name"))
        .build()
        .expect("sectioned policy configuration should build");

    assert_eq!(
        runtime.policy(),
        &BridgeRuntimePolicy::from_sections(
            BridgeExecutionPolicyBaseline::new(
                BridgeExecutionPolicyClass::DeterministicCanonical,
                BridgeRuntimePosture::Development,
            ),
            BridgeDiagnosticsPolicyBaseline::for_tier(BridgeDiagnosticsTier::Standard)
                .with_route_record_limit(17)
                .with_failure_record_limit(9),
            BridgeArtifactPolicyBaseline::new(true, false),
            crate::policy::BridgeConditionalRetentionBudget::development(),
        )
    );
}

#[test]
fn build_policy_sections_are_order_invariant() {
    let first = RuntimeBridgeBuilder::new()
        .with_execution_policy_baseline(BridgeExecutionPolicyBaseline::new(
            BridgeExecutionPolicyClass::DeterministicCanonical,
            BridgeRuntimePosture::Development,
        ))
        .with_diagnostics_policy_baseline(
            BridgeDiagnosticsPolicyBaseline::for_tier(BridgeDiagnosticsTier::Standard)
                .with_route_record_limit(23)
                .with_failure_record_limit(11),
        )
        .with_artifact_policy_baseline(BridgeArtifactPolicyBaseline::new(true, false))
        .with_relational_source(TestSource)
        .with_signal_sink(TestSink)
        .register_mapping(exact_registration("user-profile-name"))
        .build()
        .expect("first policy order should build");

    let second = RuntimeBridgeBuilder::new()
        .with_artifact_policy_baseline(BridgeArtifactPolicyBaseline::new(true, false))
        .with_diagnostics_policy_baseline(
            BridgeDiagnosticsPolicyBaseline::for_tier(BridgeDiagnosticsTier::Standard)
                .with_failure_record_limit(11)
                .with_route_record_limit(23),
        )
        .with_execution_policy_baseline(BridgeExecutionPolicyBaseline::new(
            BridgeExecutionPolicyClass::DeterministicCanonical,
            BridgeRuntimePosture::Development,
        ))
        .with_relational_source(TestSource)
        .with_signal_sink(TestSink)
        .register_mapping(exact_registration("user-profile-name"))
        .build()
        .expect("second policy order should build");

    assert_eq!(first.policy(), second.policy());
}
