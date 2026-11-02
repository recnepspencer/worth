use super::*;
use crate::durability::data::{RecoveryVerificationMode, RelationalCheckpointArtifactPolicy};
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableRecoverySource};
use crate::runtime::RelationalRuntime;
use crate::tests::support::*;
use std::sync::Arc;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
fn source_plan(
    runtime: &RelationalRuntime,
    source: Arc<dyn RelationalDurableColdSource>,
    execution: &ExecutionRequest<'_, '_>,
) -> RecoveryPlan {
    runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            RelationalDurableRecoverySource::new(None, vec![], vec![], source),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap()
}
#[test]
fn fresh_source_keeps_actual_runtime_root_and_installed_schema_for_complete() {
    let policy = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&policy);
    let mut runtime = durable_runtime_with_test_schema();
    let original_root = runtime
        .history
        .branch_cell(runtime.main_branch_identity().branch_id())
        .unwrap()
        .root()
        .unwrap();
    let mut unused = original_root
        .schema_authority()
        .registry()
        .relation_kinds
        .values()
        .next()
        .unwrap()
        .clone();
    unused.kind_id = crate::identity::data::KindId(901);
    unused.kind_name = "unused.fresh.physical".into();
    unused.aspect_contract_declarations = Default::default();
    unused.relation_integrity = crate::schema::data::RelationIntegrityDeclarations::new(
        vec![crate::schema::data::EndpointKindContractDeclaration {
            contract_id: "unused.fresh.no.self.edge".into(),
            allowed_source_kinds: vec![crate::identity::data::KindId(1)],
            allowed_target_kinds: vec![crate::identity::data::KindId(1)],
            self_edges_allowed: false,
            cross_context_policy: crate::config::data::CrossContextPolicy::AllowExplicit,
        }],
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let expected_constraint = unused.relation_integrity.clone();
    let initialization = runtime.begin_fresh_home_initialization(&execution).unwrap();
    let installation = runtime
        .prepare_fresh_home_schema_installation(&execution, initialization)
        .unwrap()
        .install_with_managed_sources(
            &execution,
            crate::schema::data::RelationalSchemaRegistry::new()
                .register_relation_kind(unused)
                .unwrap(),
            vec![],
            vec![],
        )
        .unwrap();
    drop(original_root);
    let id = runtime.runtime_instance_id();
    let identity = runtime.main_branch_identity();
    let original = runtime
        .history
        .branch_cell(identity.branch_id())
        .unwrap()
        .root()
        .unwrap();
    let schema = original.retained_schema_authority();
    let source = runtime.durability.cold_source().unwrap();
    let plan = source_plan(&runtime, source.clone(), &execution);
    let admitted = runtime
        .durability_recovery()
        .admit_fresh_home_source(&execution, plan)
        .unwrap();
    admitted
        .validate_fresh_source_runtime(&execution, &runtime)
        .unwrap();
    runtime
        .validate_fresh_initial_schema_installation(&execution, &admitted, &installation)
        .unwrap();
    assert_eq!(runtime.runtime_instance_id(), id);
    assert!(Arc::ptr_eq(
        &runtime.durability.cold_source().unwrap(),
        &source
    ));
    let actual = runtime
        .history
        .branch_cell(identity.branch_id())
        .unwrap()
        .root()
        .unwrap();
    assert!(Arc::ptr_eq(&original, &actual));
    assert!(Arc::ptr_eq(&schema, &actual.retained_schema_authority()));
    let complete = runtime
        .complete_checkpoint_service()
        .prepare(&execution, RelationalCheckpointArtifactPolicy::Embedded)
        .unwrap();
    let image = crate::durability::log::native_checkpoint_codec::decode_checkpoint(
        &execution,
        complete.checkpoint().bytes(),
    )
    .unwrap()
    .checkpoint;
    assert_eq!(
        image
            .canonical_directory
            .as_ref()
            .unwrap()
            .manifest
            .retained_empty_components
            .len(),
        1
    );
    assert!(image
        .canonical_directory
        .as_ref()
        .unwrap()
        .manifest
        .schemas
        .iter()
        .any(
            |image| image.readmit_registry().unwrap().0.authority_snapshot()
                == schema.registry().authority_snapshot()
        ));
    assert!(
        image
            .canonical_directory
            .as_ref()
            .unwrap()
            .manifest
            .schemas
            .iter()
            .any(|image| {
                let (registry, _) = image.readmit_registry().unwrap();
                registry
                    .relation_kinds
                    .get(&crate::identity::data::KindId(901))
                    .is_some_and(|kind| kind.relation_integrity == expected_constraint)
            }),
        "unused physical constraint comes from original installation, not records"
    );
    let mut foreign = durable_runtime_with_test_schema();
    let wrong = foreign
        .prepare_initial_schema_installation()
        .unwrap()
        .install(crate::schema::data::RelationalSchemaRegistry::new())
        .unwrap();
    assert!(runtime
        .validate_fresh_initial_schema_installation(&execution, &admitted, &wrong)
        .is_err());
    assert!(admitted
        .validate_fresh_source_runtime(&execution, &foreign)
        .is_err());
}
#[test]
fn fresh_source_refuses_foreign_and_nonempty_actual_sources_without_rebuild() {
    let policy = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&policy);
    let mut runtime = durable_runtime_with_test_schema();
    let foreign = durable_runtime_with_test_schema();
    let original = runtime.runtime_instance_id();
    let plan = source_plan(
        &runtime,
        foreign.durability.cold_source().unwrap(),
        &execution,
    );
    assert!(runtime
        .durability_recovery()
        .admit_fresh_home_source(&execution, plan)
        .is_err());
    assert_eq!(runtime.runtime_instance_id(), original);
    let source = runtime.durability.cold_source().unwrap();
    let stale = source_plan(&runtime, source, &execution);
    let written = create_entity_outcome(&execution, &runtime, "not-genesis");
    release_test_commit_snapshot(&runtime, &written);
    assert!(runtime
        .durability_recovery()
        .admit_fresh_home_source(&execution, stale)
        .is_err());
    assert_eq!(runtime.runtime_instance_id(), original);
    assert!(runtime.history().latest_commit().is_some());
}
#[test]
fn fresh_receipt_refuses_actual_movement_and_preserves_original_cancellation() {
    let policy = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&policy);
    let mut runtime = durable_runtime_with_test_schema();
    let plan = source_plan(
        &runtime,
        runtime.durability.cold_source().unwrap(),
        &execution,
    );
    let admitted = runtime
        .durability_recovery()
        .admit_fresh_home_source(&execution, plan)
        .unwrap();
    let cancelled = CancellationSource::new();
    let stopped = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 20)),
        cancelled.token(),
        None,
    );
    let stopped_execution = ExecutionRequest::serial(&stopped);
    cancelled.cancel();
    assert!(matches!(
        admitted.validate_fresh_source_runtime(&stopped_execution, &runtime),
        Err(Stop::Request(_))
    ));
    let plan = source_plan(
        &runtime,
        runtime.durability.cold_source().unwrap(),
        &execution,
    );
    assert!(matches!(
        runtime
            .durability_recovery()
            .admit_fresh_home_source(&stopped_execution, plan),
        Err(Stop::Request(_))
    ));
    let written = create_entity_outcome(&execution, &runtime, "advanced-genesis");
    release_test_commit_snapshot(&runtime, &written);
    assert!(admitted
        .validate_fresh_source_runtime(&execution, &runtime)
        .is_err());
}

#[test]
fn fresh_source_refuses_real_selected_empty_checkpoint() {
    let policy = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&policy);
    let mut runtime = durable_runtime_with_test_schema();
    checkpoint_durably(&execution, &runtime).unwrap();
    let plan = recovery_plan_for(
        &execution,
        &runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    assert!(plan.checkpoint.is_some());
    let id = runtime.runtime_instance_id();
    assert!(runtime
        .durability_recovery()
        .admit_fresh_home_source(&execution, plan)
        .is_err());
    assert_eq!(runtime.runtime_instance_id(), id);
}
