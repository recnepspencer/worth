//! Two independent installations of the same declared codec world.
use worth_foundational::facade::*;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    config::RelationalRuntimeProfile,
    durability::{RecoveredRelationalRuntimeAuthority, RelationalNativeCheckpoint},
    identity::{EntityId, KindId, PartitionId},
    indexes::{DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind},
    mvcc::{
        RelationalInstalledApplicationPublicationAuthority, RelationalPublicationOutcome,
        RelationalTransactionIntent,
    },
    runtime::{
        RecoveredCheckpointProgramActivationUpdate, RelationalInitialSchemaInstallationReceipt,
        RelationalRuntime, RelationalRuntimeApi,
    },
    schema::*,
    symbols::ClientKey,
    transactions::*,
};

pub(super) const INDEX: DerivedIndexId = DerivedIndexId(1);
pub(super) const SOURCE_VALUE: &str =
    r#"{"label":"authored nondefault","points":[2,7,11],"mode":{"Nested":9}}"#;

pub(super) struct World {
    pub(super) runtime: RelationalRuntime,
    pub(super) authored: EntityId,
    pub(super) activation: EntityId,
    pub(super) output: EntityId,
}

pub(super) fn installed(
    version: u8,
) -> (
    RelationalRuntime,
    RelationalInitialSchemaInstallationReceipt,
) {
    installed_with(registry(version), Vec::new())
}

pub(super) fn installed_with(
    registry: RelationalSchemaRegistry,
    invariants: Vec<worth_relational::facade::runtime::CustomInvariantRegistration>,
) -> (
    RelationalRuntime,
    RelationalInitialSchemaInstallationReceipt,
) {
    let mut runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(registry)
        .build()
        .expect("valid native runtime configuration");
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install_with_custom_invariants(RelationalSchemaRegistry::new(), invariants)
        .unwrap();
    (runtime, receipt)
}

pub(super) fn world(execution: &worth_execution::ExecutionRequest<'_, '_>, genuine: bool) -> World {
    let (runtime, receipt) = installed(1);
    world_with_installation(execution, runtime, receipt, genuine)
}

pub(super) fn world_with_installation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: RelationalRuntime,
    receipt: RelationalInitialSchemaInstallationReceipt,
    genuine: bool,
) -> World {
    let (owner, installed) = receipt.issue_application_publication_owner();
    let authored = create(execution, &runtime, 1, "authored", SOURCE_VALUE, None);
    let activation = create(execution, &runtime, 2, "activation", "program.v1", None);
    let output = create(
        execution,
        &runtime,
        3,
        "derived",
        r#"{"old":41}"#,
        genuine.then(|| {
            (
                &installed,
                owner.admit_required_output("native.fixture.producer", [9; 32]),
            )
        }),
    );
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: INDEX,
            name: "derived canonical equality".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: locator("derived"),
            },
            branch_scoped: true,
        })
        .expect("index fixture has identity capacity");
    assert_eq!(
        index.index_id, INDEX,
        "fresh owner assigns the fixture index"
    );
    build_index(execution, &runtime);
    World {
        runtime,
        authored,
        activation,
        output,
    }
}

pub(super) fn basis(runtime: &RelationalRuntime) -> AdmittedRelationalBranchBasis {
    runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1
}

pub(super) fn capture(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> RelationalNativeCheckpoint {
    runtime
        .durability_authority()
        .native_checkpoint_with_policy(
execution,
worth_execution::ExecutionAllocationPolicy::SystemAllocation,
worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: None },
)
        .unwrap()
}

/// Restore `native` for codec transition through its own external members.
pub(crate) fn restore_codec_transition(
    target: &mut RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    native: &RelationalNativeCheckpoint,
) -> Result<
    (
        worth_relational::facade::durability::RecoveryOutcome,
        worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority,
    ),
    worth_relational::facade::durability::RelationalRecoveryStop,
> {
    target
        .durability_recovery()
        .restore_native_checkpoint_for_codec_transition_with_authority(
            execution,
            native,
            worth_testkit_relational::cold_recovery::members_source(native),
        )
}

pub(super) fn activation(id: EntityId) -> RecoveredCheckpointProgramActivationUpdate {
    RecoveredCheckpointProgramActivationUpdate {
        entity_id: id,
        field_locator: locator("activation"),
        expected: string("program.v1"),
        replacement: string("program.v2"),
    }
}

pub(super) fn activation_proof(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
    recovered: &RecoveredRelationalRuntimeAuthority,
    basis: &AdmittedRelationalBranchBasis,
    owner: &worth_proof::InstalledApplicationPublicationOwner,
    id: EntityId,
) -> worth_proof::AdmittedCheckpointProgramActivation {
    let commitment = runtime
        .durability_recovery()
        .checkpoint_program_activation_commitment(execution, recovered, basis, &activation(id))
        .unwrap();
    owner.admit_checkpoint_program_activation(commitment)
}

pub(super) fn create(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    kind: u32,
    key: &str,
    value: &str,
    producer: Option<(
        &RelationalInstalledApplicationPublicationAuthority,
        worth_proof::AdmittedRequiredOutputPublication,
    )>,
) -> EntityId {
    let aspect = match kind {
        1 => "source",
        2 => "activation",
        3 => "derived",
        _ => panic!("unknown fixture kind"),
    };
    let created = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(kind),
        client_key: ClientKey::raw(key),
    };
    let mut transaction = runtime
        .begin_branch_transaction(&basis(runtime), RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id: created.partition_id,
                    kind_id: created.kind_id,
                    client_key: created.client_key.clone(),
                    fields: AspectFieldPatch::from_locator(locator(aspect), string(value)),
                },
            ))),
            execution.allocation_policy(),
        )
        .unwrap();
    let proposal = transaction.validate(execution, runtime).unwrap();
    let proposal = match producer {
        Some((installed, proof)) => proposal
            .bind_required_output_publication(
                installed,
                super::publication_fixture::birth(proof, vec![created.clone()]),
            )
            .unwrap(),
        None => proposal,
    };
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let performed = match runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    {
        RelationalPublicationOutcome::Performed(performed) => performed,
        other => panic!("fixture publication: {other:?}"),
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    let id = committed.created_entity(&created).unwrap();
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    id
}

pub(super) fn build_index(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) {
    let basis = basis(runtime);
    let result = runtime.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: basis.observation().commit_id().unwrap(),
            branch_id: basis.descriptor().branch_id().clone(),
            index_ids: vec![INDEX],
        },
        &basis,
    );
    assert!(result.failed_indexes.is_empty(), "{result:?}");
    assert_eq!(result.generations.len(), 1);
}

pub(super) fn locator(aspect: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        AspectKey::new(aspect).unwrap(),
        CanonicalFieldPath::single(FieldKey::new("value").unwrap()),
    )
}

fn string(value: &str) -> AspectValue {
    AspectValue::String(InternedString::from(value))
}

pub(super) fn registry(version: u8) -> RelationalSchemaRegistry {
    let mut registry = RelationalSchemaRegistry::new();
    for (kind, name, codec, revision) in [
        (1, "source", Some(1), 1),
        (2, "activation", None, 1),
        (3, "derived", Some(version + 1), u64::from(version)),
    ] {
        let mut field = FieldDeclaration::new(
            FieldKey::new("value").unwrap(),
            ScalarAspectType::String,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            AspectEvolutionPolicy::AdditiveFieldsAllowed,
        )
        .unwrap();
        if let Some(codec) = codec {
            field = field.with_codec(CanonicalDigestId::new([codec; 32]));
        }
        let contract = AspectContract::struct_aspect(
            AspectKey::new(name).unwrap(),
            AspectIdentity(u64::from(kind)),
            AspectContractRevision(revision),
            StructAspectShape::new([field]).unwrap(),
        );
        registry = registry
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(kind),
                kind_name: name.into(),
                schema_id: SchemaId("native-codec-world".into()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                    DeclaredAspectContractBinding {
                        binding: AspectBinding::EntityField {
                            field: FieldKey::new("value").unwrap(),
                        },
                        contract,
                    },
                ]),
            })
            .unwrap();
    }
    registry
}

pub(super) fn rewrite_output(execution: &worth_execution::ExecutionRequest<'_, '_>, world: &World) {
    let mut transaction = world
        .runtime
        .begin_branch_transaction(
            &basis(&world.runtime),
            RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("ordinary equal output rewrite").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: world.output,
                    fields: AspectFieldPatch::from_locator(
                        locator("derived"),
                        string(r#"{"old":41}"#),
                    ),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(execution, &world.runtime).unwrap();
    world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
}

/// Serialize the native row's public exact-snapshot payload and identity.
pub(super) fn protected_payload(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    id: EntityId,
) -> Vec<u8> {
    let basis = basis(runtime);
    let snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let view = runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .unwrap()
        .unwrap();
    let row = view.get_entity(id).unwrap();
    let payload = rmp_serde::to_vec(&(
        id,
        Some(row.kind.kind_id),
        id.generation_value(),
        row.lineage_id,
        &row.authoritative_aspect_state,
    ))
    .unwrap();
    runtime.snapshots().release_snapshot(&snapshot).unwrap();
    payload
}
