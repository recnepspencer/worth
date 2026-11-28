use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn durability_contract_failure_descriptor_semantics_version_mismatch_is_explicit() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "main-a");
    let mut plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    plan.descriptor_semantics_version = DescriptorSemanticsVersion(99);

    let mut recovered = runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::SchemaMismatch);
    assert!(matches!(
        error.authority_continuity_mismatch,
        Some(
            RecoveryAuthorityContinuityMismatch::DescriptorSemanticsVersion {
                expected: DescriptorSemanticsVersion(99),
                found: DescriptorSemanticsVersion(1),
            }
        )
    ));
}

#[test]
fn durability_certification_recovery_authority_continuity_is_explained_and_counted() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "main-a");
    runtime.performance_access().reset_counters();
    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();

    let layer = ReplayVerificationLayer::DigestParity;
    let continuity = &plan.authority_continuity;
    assert_eq!(plan.verification_plan.mode(), RecoveryMode::NormalRecoveryVerification);
    assert_eq!(continuity.verification_outcome, RecoveryVerificationOutcome::VerifiedAtLayer(layer));
    assert_eq!(continuity.first_mismatch, None);
    assert_eq!(continuity.descriptor_version_parity, RecoveryAuthorityParity::VerifiedAtLayer(layer));
    assert_eq!(continuity.schema_transition_parity, RecoveryAuthorityParity::VerifiedAtLayer(layer));
    let mut recovered = runtime_with_test_schema();
    recovered.durability_recovery().recover(execution, plan).unwrap();

    let counters = runtime.performance_access().counters();
    assert!(counters.replay_digest_parity_checks >= 1);
    assert_eq!(counters.descriptor_version_mismatches_encountered, 0);
}

#[test]
fn contradictory_descriptor_version_refuses_before_recovery_or_owner_effects() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "main-a");
    let owner = runtime.runtime_instance_id();
    let head = runtime.history().branch_head(&BranchId("main".into()));
    medium.rewrite_artifact_envelope(
        execution,
        crate::facade::publication::PatchStreamPosition(1),
        |envelope| envelope.descriptor_semantics_version = DescriptorSemanticsVersion(99),
    );
    // The unchanged original semantic image still says version 1. Native must
    // reject that contradiction before offering a schema-continuity plan.
    let refused = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap_err();
    assert!(matches!(
        refused,
        crate::durability::data::RelationalRecoveryStop::Durability(error)
            if error.class == RecoveryFailureClass::CorruptCanonicalArtifact
                && error.detail.contains("PositionAdmission(SemanticResult(SchemaMismatch))")
    ));
    assert_eq!(runtime.runtime_instance_id(), owner);
    assert_eq!(
        runtime.history().branch_head(&BranchId("main".into())),
        head
    );
}

#[test]
fn durability_contract_failure_descriptor_canonical_basis_version_mismatch_is_explicit() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let mut runtime = durable_runtime_for_medium(&medium);
    let _ = create_entity_outcome(execution, &runtime, "main-a");

    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::with_default_declared_aspects(
                CascadeDeletePolicy::CascadeDeleteRelations,
            )
        }
        .build_registry(),
    );
    let mut txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
                .with_schema_transition(
                    schema_transition_for_subscriber_impact(
                        SchemaVersionId(2),
                        SchemaSubscriberImpact::ConsumableSurfaceChanged,
                    ),
                    Some(SchemaReconciliationPolicy::PreserveInformation),
                );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn.push_batch(
        batch_create("transitioned"),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(execution, &runtime).unwrap();

    medium.rewrite_artifact_envelope(
        execution,
        crate::facade::publication::PatchStreamPosition(2),
        |envelope| {
            if let Some(descriptor) = envelope.schema_continuation_descriptor.as_mut() {
                descriptor.bridge.canonical_basis_version = DescriptorCanonicalBasisVersion(99);
            }
        },
    );

    let plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();

    assert_eq!(
        plan.authority_continuity.descriptor_version_parity,
        RecoveryAuthorityParity::Drift
    );
    assert!(matches!(
        plan.authority_continuity.first_mismatch,
        Some(
            RecoveryAuthorityContinuityMismatch::DescriptorCanonicalBasisVersion {
                expected: DescriptorCanonicalBasisVersion(1),
                found: DescriptorCanonicalBasisVersion(99),
            }
        )
    ));
    assert_eq!(
        plan.authority_continuity.verification_outcome,
        RecoveryVerificationOutcome::Rejected {
            layer: ReplayVerificationLayer::DigestParity,
            detail: "descriptor canonical basis version mismatch".to_string(),
        }
    );
}
