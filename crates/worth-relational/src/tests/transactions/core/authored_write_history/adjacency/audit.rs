//! Exact canonical bytes and their digest are private to the original-payload owner.
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::payload_digest::CanonicalEnvelopeDigestFixturePort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::payload_encoding::CanonicalCommitEnvelopeFixturePort as _;
use super::*;
use crate::config::data::CascadeDeletePolicy;

fn audit_runtime() -> RelationalRuntime {
    runtime_with_declared_aspect_schema_profile(
        RelationalRuntimeProfile::CertificationCore,
        CascadeDeletePolicy::RetainDanglingForAudit,
    )
}

#[test]
fn audit_lifecycle_recovery_preserves_exact_original_payload_bytes() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = audit_runtime();
    let a = create_entity(&execution, &runtime, "audit-source");
    let b = create_entity(&execution, &runtime, "audit-target");
    create_relation(&execution, &runtime, a, b, "audit-edge");
    let retired = delete_entity(&execution, &runtime, b);
    let canonical = runtime
        .history()
        .demand_commit_envelope(&execution, retired.commit.commit_id)
        .unwrap()
        .unwrap();
    let digest = canonical.authoritative_payload_digest_for_test().unwrap();
    let encoded = canonical.encode_authoritative_payload_for_test().unwrap();
    runtime
        .durability_authority()
        .checkpoint(&execution)
        .unwrap();
    let plan = runtime
        .durability()
        .recovery_plan(
            &execution,
            crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = audit_runtime();
    recovered
        .durability_recovery()
        .recover(&execution, plan)
        .unwrap();
    let cold = recovered
        .history()
        .demand_commit_envelope(&execution, retired.commit.commit_id)
        .unwrap()
        .unwrap();
    assert_eq!(cold.descriptive_touches(), canonical.descriptive_touches());
    assert_eq!(
        cold.authoritative_payload_digest_for_test().unwrap(),
        digest
    );
    assert_eq!(
        cold.encode_authoritative_payload_for_test().unwrap(),
        encoded
    );
    release_test_commit_snapshot(&runtime, &retired);
}
