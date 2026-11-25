//! Home readmission metadata authority is linear, phase-bound, and kind-sealed.
mod teardown_and_debug;

use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_relational::facade::{
    branch::RelationalBranchBasisDenial,
    config::{CascadeDeletePolicy, CrossContextPolicy},
    durability::RecoveryVerificationMode,
    identity::{EntityId, KindId, PartitionId, RelationId},
    mvcc::RelationalPublicationOutcome,
    runtime::{HomeReadmissionMetadataKinds, HomeReadmissionMetadataReadDenial, RelationalRuntime},
    schema::{
        EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
        RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
    },
    symbols::ClientKey,
    transactions::{
        AspectFieldPatch, CreateIntent, CreatedEntityRef, EntityReference, EntitySpec,
        MutationIntent, RecordRef, RelationSpec, WorkerIntentBatch,
    },
};
use worth_testkit_relational::{
    aspect_storage::publications::{begin_main_transaction, changed_entities},
    durable_publication::{field_patch, memory_runtime},
    shared_relational_certification_test_execution_policy::serial_request,
};

const APP_ENTITY_KIND: KindId = KindId(1);
const METADATA_ENTITY_KIND: KindId = KindId(100);
const METADATA_RELATION_KIND: KindId = KindId(101);
const APP_RELATION_KIND: KindId = KindId(102);

fn runtime_with_admitted_metadata_kinds() -> RelationalRuntime {
    let mut runtime = memory_runtime();
    let (schema_id, schema_version_id) = runtime
        .config()
        .schema
        .registry
        .authoritative_schema_basis()
        .expect("valid schema basis")
        .unwrap_or_else(|| (SchemaId("test".into()), SchemaVersionId(1)));
    let additions = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: METADATA_ENTITY_KIND,
            kind_name: "owner-metadata-entity".into(),
            schema_id: schema_id.clone(),
            schema_version_id,
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .expect("metadata entity kind")
        .register_relation_kind(RelationKindRegistration {
            kind_id: METADATA_RELATION_KIND,
            kind_name: "owner-metadata-relation".into(),
            schema_id: schema_id.clone(),
            schema_version_id,
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
            relation_integrity: RelationIntegrityDeclarations::default(),
        })
        .expect("metadata relation kind")
        .register_relation_kind(RelationKindRegistration {
            kind_id: APP_RELATION_KIND,
            kind_name: "application-relation".into(),
            schema_id,
            schema_version_id,
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
            relation_integrity: RelationIntegrityDeclarations::default(),
        })
        .expect("app relation kind");
    let metadata_kinds =
        HomeReadmissionMetadataKinds::new([METADATA_ENTITY_KIND], [METADATA_RELATION_KIND]);
    runtime
        .prepare_initial_schema_installation()
        .expect("uncommitted runtime")
        .install_with_managed_sources_and_readmission_metadata(
            additions,
            Vec::new(),
            Vec::new(),
            metadata_kinds,
        )
        .expect("install schema and seal metadata kinds");
    runtime
}

fn create_generated_group(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> Vec<EntityId> {
    let source = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: APP_ENTITY_KIND,
        client_key: ClientKey::raw("materialized-source"),
    };
    let target = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: APP_ENTITY_KIND,
        client_key: ClientKey::raw("materialized-target"),
    };
    let batch = WorkerIntentBatch::new("generated-group")
        .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: source.partition_id,
            kind_id: source.kind_id,
            client_key: source.client_key.clone(),
            fields: field_patch("name", "name", "materialized-source"),
        })))
        .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: target.partition_id,
            kind_id: target.kind_id,
            client_key: target.client_key.clone(),
            fields: field_patch("name", "name", "materialized-target"),
        })))
        .push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("materialized-edge"),
                source: EntityReference::Created(source),
                target: EntityReference::Created(target),
                fields: AspectFieldPatch::default(),
            },
        )));
    let mut tx = begin_main_transaction(runtime);
    tx.push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
        .expect("generated group fits");
    let commit = tx.commit(execution, runtime).expect("generated group commits");
    changed_entities(&commit)
}

fn seed_entity(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    kind_id: KindId,
    key: &str,
) -> EntityId {
    let mut tx = begin_main_transaction(runtime);
    tx.push_batch(
        WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(
            EntitySpec {
                partition_id: PartitionId::main(),
                kind_id,
                client_key: ClientKey::raw(key),
                fields: AspectFieldPatch::default(),
            },
        ))),
        ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("entity fits");
    let commit = tx.commit(execution, runtime).expect("entity commits");
    changed_entities(&commit)[0]
}

fn seed_relation(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    kind_id: KindId,
    source: EntityId,
    target: EntityId,
    key: &str,
) -> RelationId {
    let mut tx = begin_main_transaction(runtime);
    tx.push_batch(
        WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id,
                client_key: ClientKey::raw(key),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(target),
                fields: AspectFieldPatch::default(),
            },
        ))),
        ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("relation fits");
    let commit = tx.commit(execution, runtime).expect("relation commits");
    commit
        .changed_records
        .iter()
        .find_map(|r| match r {
            RecordRef::Relation(id) => Some(*id),
            _ => None,
        })
        .expect("created relation")
}

#[test]
fn home_readmission_metadata_view_enforces_phase_expiry_and_admitted_kinds() {
    let host_request = serial_request();
    let execution = &ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_admitted_metadata_kinds();
    let suspended_entities = create_generated_group(execution, &runtime);
    let live_app_a = seed_entity(execution, &runtime, APP_ENTITY_KIND, "live-app-a");
    let live_app_b = seed_entity(execution, &runtime, APP_ENTITY_KIND, "live-app-b");
    let _live_app_rel = seed_relation(
        execution, &runtime, APP_RELATION_KIND, live_app_a, live_app_b, "live-app-rel",
    );
    let live_meta_a = seed_entity(execution, &runtime, METADATA_ENTITY_KIND, "live-meta-a");
    let live_meta_b = seed_entity(execution, &runtime, METADATA_ENTITY_KIND, "live-meta-b");
    let live_meta_rel = seed_relation(
        execution, &runtime, METADATA_RELATION_KIND, live_meta_a, live_meta_b, "live-meta-rel",
    );

    let identity = runtime.main_branch_identity();
    let services = runtime.owner_component_services();
    runtime.durability_authority().checkpoint(execution).expect("checkpoint before suspension");
    let recovery_plan = |rt: &RelationalRuntime| {
        rt.durability()
            .recovery_plan(execution, RecoveryVerificationMode::NormalRecoveryVerification)
            .expect("recovery plan")
    };
    let live_plan_1 = recovery_plan(&runtime);
    let live_plan_2 = recovery_plan(&runtime);
    let (_, source_basis) = services.basis_port().observe_branch(&identity).expect("exact basis");
    let prepared_suspension = services
        .materialization_port()
        .prepare_generated_materialization_suspension(execution, &source_basis, &suspended_entities)
        .expect("prepare generated suspension");
    let (candidate, completion) = prepared_suspension.into_parts();
    let RelationalPublicationOutcome::Performed(performed) = services
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("generated suspension candidate must publish")
    };
    let suspension_commit = services
        .settlement_port()
        .settle_performed_publication(performed)
        .expect("settle generated suspension");
    completion
        .complete(suspension_commit)
        .expect("complete generated suspension");

    // 1. Recover suspended runtime: ordinary projection refuses.
    let mut recovered = runtime_with_admitted_metadata_kinds();
    let (_, recovered_authority) = recovered
        .durability_recovery()
        .recover_with_authority(execution, recovery_plan(&runtime))
        .expect("recover runtime with authority");
    let recovered_identity = recovered.main_branch_identity();
    let (_, recovered_basis) = recovered
        .observe_branch(&recovered_identity)
        .expect("observe recovered suspended branch");
    assert_eq!(
        recovered
            .read_truth()
            .project_observation(&recovered_basis.observation())
            .unwrap_err(),
        RelationalBranchBasisDenial::MaterializationUnavailable
    );

    // 2. Foreign recovered runtime's authority is rejected.
    let foreign_source = runtime_with_admitted_metadata_kinds();
    foreign_source.durability_authority().checkpoint(execution).expect("foreign checkpoint");
    let mut foreign_recovered = runtime_with_admitted_metadata_kinds();
    let (_, foreign_authority) = foreign_recovered
        .durability_recovery()
        .recover_with_authority(execution, recovery_plan(&foreign_source))
        .expect("foreign recover");
    let foreign_metadata_auth = foreign_authority
        .begin_home_readmission_metadata()
        .expect("foreign authority issued once");
    assert!(matches!(
        recovered
            .read_truth()
            .project_home_readmission_metadata(&foreign_metadata_auth, &recovered_basis.observation()),
        Err(RelationalBranchBasisDenial::ForeignRuntime { .. })
    ));

    // 3. Single-shot issuance: second attempt on the same authority fails.
    let matching_auth = recovered_authority
        .begin_home_readmission_metadata()
        .expect("first authority issuance succeeds");
    assert_eq!(
        recovered_authority.begin_home_readmission_metadata().unwrap_err(),
        RelationalBranchBasisDenial::OwnerUnavailable
    );

    // 4. Metadata view refuses application kinds while allowing sealed metadata kinds.
    let view = recovered
        .read_truth()
        .project_home_readmission_metadata(&matching_auth, &recovered_basis.observation())
        .expect("matching authority projects recovered branch");
    assert_eq!(
        view.bounded_entities_of_kind(execution, APP_ENTITY_KIND, usize::MAX).unwrap_err(),
        HomeReadmissionMetadataReadDenial::KindNotAdmitted(APP_ENTITY_KIND)
    );
    assert_eq!(
        view.bounded_relations_of_kind(execution, APP_RELATION_KIND, usize::MAX).unwrap_err(),
        HomeReadmissionMetadataReadDenial::KindNotAdmitted(APP_RELATION_KIND)
    );
    let meta_entities = view
        .bounded_entities_of_kind(execution, METADATA_ENTITY_KIND, usize::MAX)
        .expect("metadata entity scan executes")
        .expect("metadata entity scan fits bound");
    let entity_ids: Vec<_> = meta_entities.records().iter().map(|r| r.entity_id).collect();
    assert_eq!(entity_ids, vec![live_meta_a, live_meta_b]);
    for suspended in &suspended_entities {
        assert!(!entity_ids.contains(suspended));
    }
    assert!(!entity_ids.contains(&live_app_a));
    let meta_relations = view
        .bounded_relations_of_kind(execution, METADATA_RELATION_KIND, usize::MAX)
        .expect("metadata relation scan executes")
        .expect("metadata relation scan fits bound");
    let rel_ids: Vec<_> = meta_relations.records().iter().map(|r| r.relation_id).collect();
    assert_eq!(rel_ids, vec![live_meta_rel]);

    // 5. Service handoff (`finish_home_readmission`) expires open view and authority.
    recovered_authority.finish_home_readmission();
    assert_eq!(
        view.bounded_entities_of_kind(execution, METADATA_ENTITY_KIND, usize::MAX).unwrap_err(),
        HomeReadmissionMetadataReadDenial::PhaseClosed
    );
    assert_eq!(
        view.bounded_relations_of_kind(execution, METADATA_RELATION_KIND, usize::MAX).unwrap_err(),
        HomeReadmissionMetadataReadDenial::PhaseClosed
    );
    drop(view);
    assert_eq!(
        recovered
            .read_truth()
            .project_home_readmission_metadata(&matching_auth, &recovered_basis.observation())
            .unwrap_err(),
        RelationalBranchBasisDenial::OwnerUnavailable
    );

    // 6. Post-recovery commit auto-closes the readmission phase without `finish_home_readmission`.
    let mut recovered_commit = runtime_with_admitted_metadata_kinds();
    let (_, commit_authority) = recovered_commit
        .durability_recovery()
        .recover_with_authority(execution, live_plan_1)
        .expect("recover live runtime");
    let commit_identity = recovered_commit.main_branch_identity();
    let (_, before_commit_basis) = recovered_commit
        .observe_branch(&commit_identity)
        .expect("observe recovered branch before commit");
    let commit_meta_auth = commit_authority
        .begin_home_readmission_metadata()
        .expect("issue authority before commit");
    let open_view = recovered_commit
        .read_truth()
        .project_home_readmission_metadata(&commit_meta_auth, &before_commit_basis.observation())
        .expect("open view during active phase");
    assert_eq!(
        open_view
            .bounded_entities_of_kind(execution, METADATA_ENTITY_KIND, usize::MAX)
            .expect("scan before commit")
            .expect("fits")
            .records()
            .len(),
        2
    );
    let _post_service = seed_entity(execution, &recovered_commit, METADATA_ENTITY_KIND, "post-service");
    let (_, after_commit_basis) = recovered_commit
        .observe_branch(&commit_identity)
        .expect("observe branch after post-recovery commit");
    assert_eq!(
        open_view
            .bounded_entities_of_kind(execution, METADATA_ENTITY_KIND, usize::MAX)
            .unwrap_err(),
        HomeReadmissionMetadataReadDenial::PhaseClosed
    );
    drop(open_view);
    assert_eq!(
        recovered_commit
            .read_truth()
            .project_home_readmission_metadata(&commit_meta_auth, &before_commit_basis.observation())
            .unwrap_err(),
        RelationalBranchBasisDenial::OwnerUnavailable
    );
    assert_eq!(
        recovered_commit
            .read_truth()
            .project_home_readmission_metadata(&commit_meta_auth, &after_commit_basis.observation())
            .unwrap_err(),
        RelationalBranchBasisDenial::OwnerUnavailable
    );

    // 7. Committing before issuance also closes issuance.
    let mut recovered_late = runtime_with_admitted_metadata_kinds();
    let (_, late_authority) = recovered_late
        .durability_recovery()
        .recover_with_authority(execution, live_plan_2)
        .expect("recover third runtime");
    let _late_commit = seed_entity(execution, &recovered_late, METADATA_ENTITY_KIND, "late-commit");
    assert_eq!(
        late_authority.begin_home_readmission_metadata().unwrap_err(),
        RelationalBranchBasisDenial::OwnerUnavailable
    );
}
