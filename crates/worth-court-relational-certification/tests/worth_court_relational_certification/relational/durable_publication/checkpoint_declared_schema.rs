//! D5: recovery binds a runtime to the schema its recovered main head
//! records. Declaring any other registry — even identical contracts under
//! another version label — refuses before service; after a durable schema
//! transition only the transition's target is accepted. Both hold whether
//! the head is replayed from the journal or restored from a checkpoint.
use super::checkpoint_recovery_observation::failure as expect_durability_failure;
use super::checkpoint_schema_versions::registry;
use super::checkpoint_source_capture::capture_durably;
use worth_relational::facade::config::RelationalRuntimeProfile;
use worth_relational::facade::durability::RecoveryAuthorityContinuityMismatch;
use worth_relational::facade::durability::RecoveryFailureClass;
use worth_relational::facade::durability::RecoveryVerificationMode as RecoveryMode;
use worth_relational::facade::durability::RelationalRecoveryStop;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::schema::SchemaVersionId;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::create_entity_outcome;

fn durable_runtime_declaring(
    medium: &std::sync::Arc<TestDurableMedium>,
    version: u32,
) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(registry(version))
        .build_durable(medium.attachment())
        .expect("valid native runtime configuration")
}

/// Reopen `source`'s medium in a runtime declaring schema `version`.
fn reopen_declaring(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    medium: &std::sync::Arc<TestDurableMedium>,
    version: u32,
) -> Result<RelationalRuntime, RelationalRecoveryStop> {
    let mut recovered = durable_runtime_declaring(medium, version);
    let plan = recovered.durability().recovery_plan_from_source(
        execution,
        medium.recovery_source(),
        RecoveryMode::NormalRecoveryVerification,
    )?;
    recovered.durability_recovery().recover(execution, plan)?;
    Ok(recovered)
}

fn assert_refused_as_declared_schema(
    refused: Result<RelationalRuntime, RelationalRecoveryStop>,
    recorded: u32,
    declared: u32,
    case: &str,
) {
    let error = expect_durability_failure(refused.err().unwrap_or_else(|| {
        panic!("{case}: declaring v{declared} over a v{recorded} head recovered")
    }));
    assert_eq!(
        error.class,
        RecoveryFailureClass::DeclaredSchemaMismatch,
        "{case}"
    );
    assert!(
        matches!(
            error.authority_continuity_mismatch,
            Some(RecoveryAuthorityContinuityMismatch::SchemaRegistryShape {
                expected_primary_schema_version,
                found_primary_schema_version,
                ..
            }) if expected_primary_schema_version == SchemaVersionId(recorded)
                && found_primary_schema_version == SchemaVersionId(declared)
        ),
        "{case}: {error:?}"
    );
}

#[test]
fn another_label_with_identical_contracts_refuses_before_service() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let medium = TestDurableMedium::new();
    let source = durable_runtime_declaring(&medium, 1);
    create_entity_outcome(execution, &source, "recorded-under-v1");
    for checkpointed in [false, true] {
        if checkpointed {
            capture_durably(execution, &source, &medium).expect("the checkpoint captures");
        }
        let case = format!("checkpointed {checkpointed}");
        assert_refused_as_declared_schema(reopen_declaring(execution, &medium, 7), 1, 7, &case);
        let reopened = reopen_declaring(execution, &medium, 1)
            .unwrap_or_else(|stop| panic!("{case}: the recorded schema reopens: {stop:?}"));
        create_entity_outcome(execution, &reopened, "served-under-v1");
    }
}

#[test]
fn after_a_durable_transition_only_its_target_is_served() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let medium = TestDurableMedium::new();
    let source = durable_runtime_declaring(&medium, 1);
    create_entity_outcome(execution, &source, "before-transition");
    super::checkpoint_schema_versions::transition_main(execution, &source, 2, "transitioned");
    for checkpointed in [false, true] {
        if checkpointed {
            capture_durably(execution, &source, &medium).expect("the checkpoint captures");
        }
        let case = format!("checkpointed {checkpointed}");
        assert_refused_as_declared_schema(reopen_declaring(execution, &medium, 1), 2, 1, &case);
        let reopened = reopen_declaring(execution, &medium, 2)
            .unwrap_or_else(|stop| panic!("{case}: the transition target reopens: {stop:?}"));
        create_entity_outcome(execution, &reopened, "served-under-v2");
    }
}
