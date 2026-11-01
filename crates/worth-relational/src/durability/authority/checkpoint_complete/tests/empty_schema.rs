//! Original Empty authority survives root retirement and is actually consumed.
use crate::durability::data::{
    RelationalCheckpointArtifactPolicy, RelationalNativeCheckpointCaptureDenial as Denial,
};
use crate::schema::data::RelationalSchemaRegistry;
use crate::tests::support::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
#[test]
fn complete_empty_archive_and_initial_installation_preserve_unused_original_schema() {
    let setup = crate::test_execution_policy::serial_request();
    let setup_execution = ExecutionRequest::serial(&setup);
    let mut runtime = runtime_with_test_schema();
    let identity = runtime.main_branch_identity();
    let original = runtime
        .history
        .branch_cell(identity.branch_id())
        .unwrap()
        .root()
        .unwrap();
    let old_root = std::sync::Arc::downgrade(&original);
    let old_schema = original.retained_schema_authority();
    let expected = old_schema.registry().authority_snapshot();
    assert!(!old_schema.registry().entity_kinds.is_empty());
    drop(original);
    runtime.archive_branch(&setup_execution, &identity).unwrap();
    // This registration has no records, so C/T payload usage cannot fabricate
    // its physical constraint. The actual new original authority must carry it.
    let mut unused = old_schema
        .registry()
        .relation_kinds
        .values()
        .next()
        .unwrap()
        .clone();
    unused.kind_id = crate::identity::data::KindId(901);
    unused.kind_name = "unused.complete.physical".to_owned();
    unused.aspect_contract_declarations = Default::default();
    unused.relation_integrity = crate::schema::data::RelationIntegrityDeclarations::new(
        vec![crate::schema::data::EndpointKindContractDeclaration {
            contract_id: "unused.no.self.edge".into(),
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
    runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(
            RelationalSchemaRegistry::new()
                .register_relation_kind(unused)
                .unwrap(),
        )
        .unwrap();
    let replacement = runtime
        .history
        .branch_cell(identity.branch_id())
        .unwrap()
        .root()
        .unwrap();
    let replacement_schema = replacement.retained_schema_authority();
    assert!(!std::sync::Arc::ptr_eq(&old_schema, &replacement_schema));
    let expected_replacement = replacement_schema.registry().authority_snapshot();
    assert_ne!(expected, expected_replacement);
    drop(replacement);
    drop(old_schema); // Only actual producer DATA may retain the original authority.
    runtime.run_branch_root_reclamation_pass();
    assert!(
        old_root.upgrade().is_none(),
        "schema DATA cannot depend on the retired original Empty Root"
    );
    let budget = SerialMemoryBudget::new(Some(16 * 1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let execution = ExecutionRequest::serial(&host);
    let complete = runtime
        .complete_checkpoint_service()
        .prepare(&execution, RelationalCheckpointArtifactPolicy::Embedded)
        .unwrap();
    let held = budget.charged_memory_bytes();
    assert!(held > 0);
    let alias = complete.checkpoint().clone();
    let decoded = crate::durability::log::native_checkpoint_codec::decode_checkpoint(
        &setup_execution,
        complete.checkpoint().bytes(),
    )
    .unwrap()
    .checkpoint;
    let directory = decoded.canonical_directory.as_ref().unwrap();
    directory.manifest.assert_original_flat_wire();
    assert!(directory.manifest.commits.is_empty());
    assert!(directory.manifest.canonical_members.entries().is_empty());
    assert_eq!(directory.manifest.coverage.up_to_branch_transition.0, 1);
    assert_eq!(directory.manifest.retained_empty_components.len(), 1);
    assert!(directory.manifest.schemas.iter().any(|image| image
        .readmit_registry()
        .unwrap()
        .0
        .authority_snapshot()
        == expected));
    assert!(
        directory.manifest.schemas.iter().any(|image| {
            let (schema, _) = image.readmit_registry().unwrap();
            schema.authority_snapshot() == expected_replacement
                && schema
                    .relation_kinds
                    .get(&crate::identity::data::KindId(901))
                    .is_some_and(|registration| {
                        registration.relation_integrity == expected_constraint
                    })
        }),
        "the unused physical constraint must survive in the full original image"
    );
    assert!(directory.manifest.roots.is_empty());
    drop(decoded);
    drop(complete);
    assert!(
        budget.charged_memory_bytes() > 0,
        "same real Native alias keeps the prepared output custody"
    );
    drop(alias);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
#[test]
fn complete_refuses_recovered_empty_without_original_birth_authority() {
    let setup = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&setup);
    let source = durable_runtime_with_test_schema();
    let ordinary = source
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();
    let mut receiver = durable_runtime_with_test_schema();
    receiver
        .durability_recovery()
        .restore_native_checkpoint(&execution, &ordinary)
        .unwrap();
    assert!(matches!(
        receiver
            .complete_checkpoint_service()
            .prepare(&execution, RelationalCheckpointArtifactPolicy::Embedded),
        Err(Denial::OriginalAuthorityUnavailable)
    ));
}
