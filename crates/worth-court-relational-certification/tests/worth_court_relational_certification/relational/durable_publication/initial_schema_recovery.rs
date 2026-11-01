use super::*;
use std::sync::{Arc, Mutex};
use worth_relational::facade::{
    config::CascadeDeletePolicy,
    durability::{RecoveryFailureClass, RelationalRecoveryStop},
    runtime::RelationalInitialSchemaInstallationDenialKind,
    schema::SchemaVersionId,
};
use worth_testkit_relational::{
    aspect_storage::AspectSchemaFixture,
    durable_publication::{initial_schema::installed_runtime, schema},
};

fn different_schema() -> worth_relational::facade::schema::RelationalSchemaRegistry {
    AspectSchemaFixture {
        schema_version_id: SchemaVersionId(7),
        ..AspectSchemaFixture::with_default_declared_aspects(
            CascadeDeletePolicy::CascadeDeleteRelations,
        )
    }
    .build_registry()
}

#[test]
fn source_recovery_preserves_original_installed_executor_and_readmits_genuine_receipt() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let source_calls = Arc::new(Mutex::new(Vec::new()));
    let (source, _) = installed_runtime(&medium, schema(), source_calls.clone());
    create_entity(execution, &source, "original-source-effect");
    let original_source_calls = source_calls.lock().unwrap().clone();
    assert!(!original_source_calls.is_empty());
    let target_calls = Arc::new(Mutex::new(Vec::new()));
    let (mut runtime, receipt) = installed_runtime(&medium, schema(), target_calls.clone());
    let original_runtime_id = runtime.runtime_instance_id();
    assert!(
        runtime
            .readmit_recovered_initial_schema_installation(receipt.clone())
            .is_err(),
        "a live original receipt is not already a recovered installation"
    );
    let original: Arc<dyn worth_relational::facade::durability::RelationalDurableColdSource> =
        medium.clone();
    let protection =
        worth_relational::facade::durability::RelationalDurableReadProtection::capture(
            &original, execution,
        )
        .unwrap();
    let protected=worth_relational::facade::durability::RelationalDurableRecoverySource::from_protected_source(execution,None,original,&protection).unwrap();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            protected,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover_with_authority(execution, plan)
        .unwrap();
    assert_ne!(runtime.runtime_instance_id(), original_runtime_id);
    assert!(
        target_calls.lock().unwrap().is_empty(),
        "source replay does not invoke the installed mutation handler"
    );
    let readmitted = runtime
        .readmit_recovered_initial_schema_installation(receipt.clone())
        .unwrap();
    assert_eq!(
        readmitted.runtime_instance_id(),
        runtime.runtime_instance_id()
    );
    assert_eq!(
        readmitted.custom_invariant_generation(),
        receipt.custom_invariant_generation()
    );
    assert_eq!(
        readmitted.custom_invariant_inventory_digest(),
        receipt.custom_invariant_inventory_digest()
    );
    assert_eq!(
        readmitted.retained_entity_kind_count(),
        receipt.retained_entity_kind_count()
    );
    assert_eq!(
        readmitted.retained_relation_kind_count(),
        receipt.retained_relation_kind_count()
    );
    let foreign_medium = TestDurableMedium::new();
    let (_, foreign_receipt) = installed_runtime(
        &foreign_medium,
        different_schema(),
        Arc::new(Mutex::new(Vec::new())),
    );
    assert_eq!(
        foreign_receipt.custom_invariant_generation(),
        receipt.custom_invariant_generation()
    );
    assert_eq!(
        foreign_receipt.custom_invariant_inventory_digest(),
        receipt.custom_invariant_inventory_digest()
    );
    assert_eq!(
        runtime
            .readmit_recovered_initial_schema_installation(foreign_receipt)
            .unwrap_err()
            .kind(),
        RelationalInitialSchemaInstallationDenialKind::RecoveredAuthorityMismatch
    );
    let committed = create_entity_outcome(execution, &runtime, "actual-after-recovery");
    let calls = target_calls.lock().unwrap().clone();
    assert!(
        calls
            .iter()
            .any(|(proposed, _)| *proposed == committed.commit.version_id),
        "the same original target-installed callback actually ran for the new committed effect"
    );
    assert_eq!(
        *source_calls.lock().unwrap(),
        original_source_calls,
        "recovery did not substitute or replay the source's executor"
    );
    release_test_commit_snapshot(&runtime, &committed);
    runtime.prepare_close().unwrap().finish();
}

#[test]
fn source_recovery_preserving_executors_still_refuses_foreign_declared_schema_before_cutover() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let (source, _) = installed_runtime(&medium, schema(), Arc::new(Mutex::new(Vec::new())));
    create_entity(execution, &source, "recorded-v1");
    let calls = Arc::new(Mutex::new(Vec::new()));
    let (mut runtime, _) = installed_runtime(&medium, different_schema(), calls.clone());
    let original_runtime_id = runtime.runtime_instance_id();
    let offered = medium.offered().len();
    let result = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .and_then(|plan| {
            runtime
                .durability_recovery()
                .recover_with_authority(execution, plan)
        });
    let RelationalRecoveryStop::Durability(error) = result.unwrap_err() else {
        panic!("the actual declared-schema guard must refuse")
    };
    assert_eq!(error.class, RecoveryFailureClass::DeclaredSchemaMismatch);
    assert_eq!(runtime.runtime_instance_id(), original_runtime_id);
    assert!(calls.lock().unwrap().is_empty());
    assert_eq!(
        medium.offered().len(),
        offered,
        "refusal publishes no Native effect"
    );
}
