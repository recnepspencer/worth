//! D5: recovery binds a runtime to the schema its recovered main head
//! records. Declaring any other registry — even identical contracts under
//! another version label — refuses before service; after a durable schema
//! transition only the transition's target is accepted. Both hold whether
//! the head is replayed from the journal or restored from a checkpoint.

use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

fn registry(version: u32) -> RelationalSchemaRegistry {
    AspectSchemaFixture {
        schema_version_id: SchemaVersionId(version),
        ..AspectSchemaFixture::with_default_declared_aspects(
            CascadeDeletePolicy::CascadeDeleteRelations,
        )
    }
    .build_registry()
}

fn durable_runtime_declaring(version: u32) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(registry(version))
        .build_durable(TestDurableMedium::new().attachment())
}

/// Reopen `source`'s medium in a runtime declaring schema `version`.
fn reopen_declaring(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &RelationalRuntime,
    version: u32,
) -> Result<RelationalRuntime, crate::durability::data::RelationalRecoveryStop> {
    let mut recovered = durable_runtime_declaring(version);
    let plan = medium_recovery_plan(
        execution,
        source,
        &mut recovered,
        RecoveryMode::NormalRecoveryVerification,
    )?;
    recovered.durability_recovery().recover(execution, plan)?;
    Ok(recovered)
}

fn assert_refused_as_declared_schema(
    refused: Result<RelationalRuntime, crate::durability::data::RelationalRecoveryStop>,
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
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let source = durable_runtime_declaring(1);
    create_entity_outcome(execution, &source, "recorded-under-v1");
    for checkpointed in [false, true] {
        if checkpointed {
            checkpoint_durably(execution, &source).expect("the checkpoint captures");
        }
        let case = format!("checkpointed {checkpointed}");
        assert_refused_as_declared_schema(reopen_declaring(execution, &source, 7), 1, 7, &case);
        let reopened = reopen_declaring(execution, &source, 1)
            .unwrap_or_else(|stop| panic!("{case}: the recorded schema reopens: {stop:?}"));
        create_entity_outcome(execution, &reopened, "served-under-v1");
    }
}

#[test]
fn after_a_durable_transition_only_its_target_is_served() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let mut source = durable_runtime_declaring(1);
    create_entity_outcome(execution, &source, "before-transition");
    transition_main(execution, &mut source, 2);
    for checkpointed in [false, true] {
        if checkpointed {
            checkpoint_durably(execution, &source).expect("the checkpoint captures");
        }
        let case = format!("checkpointed {checkpointed}");
        assert_refused_as_declared_schema(reopen_declaring(execution, &source, 1), 2, 1, &case);
        let reopened = reopen_declaring(execution, &source, 2)
            .unwrap_or_else(|stop| panic!("{case}: the transition target reopens: {stop:?}"));
        create_entity_outcome(execution, &reopened, "served-under-v2");
    }
}

fn transition_main(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
    target: u32,
) {
    runtime.set_schema_registry_for_test(registry(target));
    let input = test_owner_transaction_validation_input_for_main(runtime).with_schema_transition(
        schema_transition_for_subscriber_impact(
            SchemaVersionId(target),
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
        ),
        Some(SchemaReconciliationPolicy::PreserveInformation),
    );
    let mut transaction = runtime
        .begin_branch_transaction(input.basis(), input.intent().clone())
        .expect("owner-admitted transaction");
    transaction
        .push_batch(
            batch_create("transitioned"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the batch stages");
    let transitioned = transaction
        .commit(execution, runtime)
        .expect("the schema transition commits");
    assert!(transitioned.envelope().schema_transition.is_some());
}
