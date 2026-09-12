use crate::facade::config::{CascadeDeletePolicy, CrossContextPolicy};
use crate::facade::identity::KindId;
use crate::facade::runtime::{RelationalInitialSchemaInstallationDenialKind, RelationalRuntimeApi};
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
    RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaRegistryErrorClass,
    SchemaVersionId,
};
use crate::facade::transactions::{CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch};
use crate::facade::{identity::PartitionId, symbols::ClientKey};

#[test]
fn initial_schema_installation_moves_empty_exact_basis_without_revoking_retained_reads() {
    let mut runtime = RelationalRuntimeApi::builder().build();
    let identity = runtime.main_branch_identity();
    let (old_descriptor, old_basis) = runtime.observe_branch(&identity).unwrap();
    let old_observation = old_basis.observation();
    let old_snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&old_observation)
        .unwrap();
    let retention_before = runtime.retention_cost_counters();
    assert_eq!(
        runtime
            .read_truth()
            .observation_schema_version(&old_observation)
            .unwrap(),
        SchemaVersionId(0)
    );

    install_entity_kind(&mut runtime, KindId(7), "installed");
    let retention_after = runtime.retention_cost_counters();
    assert_eq!(
        retention_after.head_transfers - retention_before.head_transfers,
        1,
        "initial schema replacement transfers the one live head obligation"
    );

    let retained = runtime.readmit_branch_basis(&old_descriptor).unwrap();
    let (new_descriptor, new_basis) = runtime.observe_branch(&identity).unwrap();
    let new_observation = new_basis.observation();
    let new_snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&new_observation)
        .unwrap();
    assert_ne!(
        old_descriptor.schema_commitment(),
        new_descriptor.schema_commitment()
    );
    assert_ne!(
        old_descriptor.reference().generation(),
        new_descriptor.reference().generation()
    );
    assert_eq!(
        runtime.read_truth().snapshot_schema_version(&old_snapshot),
        Some(SchemaVersionId(0))
    );
    assert_eq!(
        runtime.read_truth().snapshot_schema_version(&new_snapshot),
        Some(SchemaVersionId(1))
    );
    assert!(runtime
        .read_truth()
        .read_version(crate::identity::data::VersionId(0))
        .entities()
        .is_empty());

    drop(retained);
    drop(old_observation);
    drop(old_basis);
    assert!(runtime.snapshots().release_snapshot(&old_snapshot).is_ok());
    assert!(matches!(
        runtime.readmit_branch_basis(&old_descriptor),
        Err(crate::branch::RelationalBranchBasisDenial::StaleReferenceGeneration)
    ));
    assert!(runtime.snapshots().release_snapshot(&new_snapshot).is_ok());
}

fn install_entity_kind(
    runtime: &mut crate::runtime::RelationalRuntime,
    kind_id: KindId,
    name: &str,
) {
    runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(
            RelationalSchemaRegistry::new()
                .register_entity_kind(entity(kind_id, name))
                .unwrap(),
        )
        .unwrap();
}

#[test]
fn initial_schema_installation_retains_existing_kinds_and_rejects_duplicates() {
    let initial = RelationalSchemaRegistry::new()
        .register_entity_kind(entity(KindId(4), "existing"))
        .unwrap();
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(initial)
        .build();
    let identity = runtime.main_branch_identity();
    let before_duplicate = runtime.observe_branch(&identity).unwrap().0;
    let duplicate = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(
            RelationalSchemaRegistry::new()
                .register_entity_kind(entity(KindId(4), "replacement"))
                .unwrap(),
        )
        .unwrap_err();
    assert_eq!(
        duplicate.kind(),
        RelationalInitialSchemaInstallationDenialKind::SchemaRejected
    );
    assert_eq!(runtime.config().schema.registry.entity_kinds.len(), 1);
    assert_eq!(
        runtime.config().schema.registry.entity_kinds[&KindId(4)].kind_name,
        "existing"
    );
    assert_eq!(
        runtime.observe_branch(&identity).unwrap().0,
        before_duplicate
    );
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(
            RelationalSchemaRegistry::new()
                .register_entity_kind(entity(KindId(8), "added"))
                .unwrap()
                .register_relation_kind(relation(KindId(9), "added-relation"))
                .unwrap(),
        )
        .unwrap();
    assert_eq!(receipt.retained_entity_kind_count(), 2);
    assert_eq!(receipt.retained_relation_kind_count(), 1);
    assert!(runtime
        .config()
        .schema
        .registry
        .entity_kinds
        .contains_key(&KindId(4)));
    assert!(runtime
        .config()
        .schema
        .registry
        .entity_kinds
        .contains_key(&KindId(8)));

    let repeated = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap_err();
    assert_eq!(
        repeated.kind(),
        RelationalInitialSchemaInstallationDenialKind::InitialInvariantsAlreadySealed
    );
    assert_eq!(runtime.config().schema.registry.entity_kinds.len(), 2);
    assert_eq!(runtime.config().schema.registry.relation_kinds.len(), 1);
}

#[test]
fn held_preparation_port_observes_atomic_initial_schema_replacement() {
    let mut runtime = RelationalRuntimeApi::builder().build();
    let preparation = runtime.preparation_port();
    install_entity_kind(&mut runtime, KindId(7), "held-port-installed");
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("held-port-schema").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(7),
                    client_key: ClientKey::raw("held-port-entity"),
                    fields: Default::default(),
                }),
            )),
        )
        .expect("post-installation transaction stages");

    let candidate = preparation
        .prepare_branch_transaction(transaction)
        .expect("held port validates against the atomically replaced schema world");
    preparation
        .discard_prepared_candidate(candidate)
        .expect("held port discards through the same live owner");
}

#[test]
fn initial_schema_authority_closes_after_first_commit() {
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(
            RelationalSchemaRegistry::new()
                .register_entity_kind(entity(KindId(1), "committed"))
                .unwrap(),
        )
        .build();
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(WorkerIntentBatch::new("close-schema-installation").push(
            MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw("first"),
                fields: Default::default(),
            })),
        ))
        .unwrap();
    transaction.commit(&runtime).unwrap();

    let denial = runtime.prepare_initial_schema_installation().unwrap_err();
    assert_eq!(
        denial.kind(),
        RelationalInitialSchemaInstallationDenialKind::RuntimeAlreadyCommitted
    );
}

#[test]
fn registry_registration_never_overwrites_same_family_authority() {
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(entity(KindId(3), "first"))
        .unwrap();
    let denial = registry
        .register_entity_kind(entity(KindId(3), "second"))
        .unwrap_err();
    assert_eq!(
        denial.class,
        SchemaRegistryErrorClass::DuplicateEntityKind(KindId(3))
    );
}

fn entity(kind_id: KindId, name: &str) -> EntityKindRegistration {
    EntityKindRegistration {
        kind_id,
        kind_name: name.to_string(),
        schema_id: SchemaId("initial-installation".to_string()),
        schema_version_id: SchemaVersionId(1),
        aspect_contract_declarations: KindAspectContractDeclarations::default(),
    }
}

fn relation(kind_id: KindId, name: &str) -> RelationKindRegistration {
    RelationKindRegistration {
        kind_id,
        kind_name: name.to_string(),
        schema_id: SchemaId("initial-installation".to_string()),
        schema_version_id: SchemaVersionId(1),
        cross_context_policy: CrossContextPolicy::Forbid,
        cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
        aspect_contract_declarations: KindAspectContractDeclarations::default(),
        relation_integrity: RelationIntegrityDeclarations::default(),
    }
}
