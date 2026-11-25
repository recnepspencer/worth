//! Standalone byte restoration carries no original birth authority.
use super::*;
use worth_relational::facade::durability::{
    RelationalCheckpointArtifactPolicy, RelationalNativeCheckpointCaptureDenial as Denial,
};
#[test]
fn complete_refuses_recovered_empty_without_original_birth_authority() {
    let setup = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&setup);
    let medium = TestDurableMedium::new();
    let source = durable_runtime(&medium);
    let ordinary = source
        .durability_authority()
        .native_checkpoint_with_policy(&execution, execution.allocation_policy(), worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: None })
        .unwrap();
    let receiver_medium = TestDurableMedium::new();
    let mut receiver = durable_runtime(&receiver_medium);
    receiver
        .durability_recovery()
        .restore_native_checkpoint(
            &execution,
            &ordinary,
            worth_testkit_relational::cold_recovery::members_source(&ordinary),
        )
        .unwrap();
    assert!(matches!(
        receiver.complete_checkpoint_service().prepare(
            &execution,
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None
            }
        ),
        Err(Denial::OriginalAuthorityUnavailable)
    ));
}

#[test]
fn complete_empty_archive_and_initial_installation_preserve_unused_original_schema() {
    use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
    use worth_relational::facade::{
        config::CrossContextPolicy,
        identity::KindId,
        schema::{
            EndpointKindContractDeclaration, RelationIntegrityDeclarations,
            RelationalSchemaRegistry,
        },
    };
    use worth_testkit_relational::durable_publication::memory_runtime;

    let setup = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let setup_execution = ExecutionRequest::serial(&setup);
    let mut runtime = memory_runtime();
    let identity = runtime.main_branch_identity();
    let original = runtime.admit_branch_basis(&identity).unwrap();
    let old_root = original.custody_witness();
    let old_schema = runtime.config().schema.registry;
    let expected = old_schema.authority_snapshot();
    assert!(old_schema.entity_registration(KindId(1)).is_ok());
    drop(original);
    runtime.archive_branch(&setup_execution, &identity).unwrap();
    let mut unused = old_schema.relation_registration(KindId(2)).unwrap().clone();
    unused.kind_id = KindId(901);
    unused.kind_name = "unused.complete.physical".to_owned();
    unused.aspect_contract_declarations = Default::default();
    unused.relation_integrity = RelationIntegrityDeclarations::new(
        vec![EndpointKindContractDeclaration {
            contract_id: "unused.no.self.edge".into(),
            allowed_source_kinds: vec![KindId(1)],
            allowed_target_kinds: vec![KindId(1)],
            self_edges_allowed: false,
            cross_context_policy: CrossContextPolicy::AllowExplicit,
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
    let expected_replacement = runtime.config().schema.registry.authority_snapshot();
    assert_ne!(
        old_root.schema_authority_digest(),
        expected_replacement.digest()
    );
    assert_ne!(expected, expected_replacement);
    drop(old_schema);
    runtime.run_branch_root_reclamation_pass();
    assert!(
        !old_root.root_retained(),
        "schema DATA cannot depend on the retired original Empty Root"
    );
    let budget = SerialMemoryBudget::new(Some(16 * 1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let execution = ExecutionRequest::serial(&host);
    let complete = runtime
        .complete_checkpoint_service()
        .prepare(&execution, RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: None })
        .unwrap();
    let held = budget.charged_memory_bytes();
    assert!(held > 0);
    let alias = complete.checkpoint().clone();
    #[derive(serde::Deserialize)]
    struct Captured {
        manifest: Directory,
    }
    #[derive(serde::Deserialize)]
    struct Directory {
        coverage: worth_relational::facade::durability::CheckpointCoverage,
        schemas: Vec<Schema>,
        commits: Vec<serde::de::IgnoredAny>,
        canonical_members: Vec<serde::de::IgnoredAny>,
        retained_empty_components: Vec<serde::de::IgnoredAny>,
        roots: Vec<serde::de::IgnoredAny>,
    }
    #[derive(serde::Deserialize)]
    struct Schema {
        schema_root: [u8; 32],
        relation_registrations: Vec<Relation>,
    }
    #[derive(serde::Deserialize)]
    struct Relation {
        kind_id: KindId,
        relation_integrity: RelationIntegrityDeclarations,
    }
    let checkpoint = complete.checkpoint();
    let decoded: Captured = rmp_serde::from_slice(
        checkpoint
            .bytes()
            .strip_prefix(b"WORTH-REL-CHK23\n")
            .unwrap(),
    )
    .unwrap();
    let directory = &decoded.manifest;
    assert!(directory.commits.is_empty());
    assert!(directory.canonical_members.is_empty());
    assert_eq!(directory.coverage.up_to_branch_transition.0, 1);
    assert_eq!(directory.retained_empty_components.len(), 1);
    assert!(directory
        .schemas
        .iter()
        .any(|schema| schema.schema_root == expected.digest()));
    assert!(
        directory.schemas.iter().any(|schema| {
            schema.schema_root == expected_replacement.digest()
                && schema.relation_registrations.iter().any(|registration| {
                    registration.kind_id == KindId(901)
                        && registration.relation_integrity == expected_constraint
                })
        }),
        "the unused physical constraint must survive in the full original image"
    );
    assert!(directory.roots.is_empty());
    drop(decoded);
    drop(complete);
    assert!(
        budget.charged_memory_bytes() > 0,
        "same real Native alias keeps the prepared output custody"
    );
    drop(alias);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
