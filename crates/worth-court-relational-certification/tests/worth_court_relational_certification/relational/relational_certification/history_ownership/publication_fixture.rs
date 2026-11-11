//! Generic model, source and owned output produced through the Native owner.
use super::*;
use worth_relational::facade::mvcc::*;

pub(super) struct PublishedModel {
    pub runtime: RelationalRuntime,
    pub owner: worth_proof::InstalledApplicationPublicationOwner,
    pub installed: RelationalInstalledApplicationPublicationAuthority,
    pub model: EntityId,
    pub source: EntityId,
    pub output: EntityId,
    pub meaning: RequiredOutputPublicationMeaning,
}
pub(super) fn registry() -> RelationalSchemaRegistry {
    let fixture = AspectSchemaFixture::default();
    register_current_publication_storage(
        super::registry(),
        &fixture.schema_id,
        fixture.schema_version_id,
    )
    .unwrap()
}
use worth_testkit_relational::aspect_storage::AspectSchemaFixture;
impl PublishedModel {
    pub fn new(execution: &ExecutionRequest<'_, '_>) -> Self {
        Self::with_before_publication(execution).0
    }
    pub fn with_before_publication(
        execution: &ExecutionRequest<'_, '_>,
    ) -> (
        Self,
        worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    ) {
        let mut runtime = RelationalRuntimeApi::builder()
            .schema_registry(registry())
            .build()
            .unwrap();
        let receipt = runtime
            .prepare_initial_schema_installation()
            .unwrap()
            .install_with_custom_invariants(
                RelationalSchemaRegistry::new(),
                vec![ownership().invariant_registration().unwrap()],
            )
            .unwrap();
        let (owner, installed) = receipt.issue_application_publication_owner();
        let installed =
            installed.with_source_scope_owned_producers([("generator".into(), KindId(3))]);
        let model = create_entity(execution, &runtime, "model");
        let source = member(execution, &runtime, "source");
        create_relation(execution, &runtime, model, source, "owns-source");
        let meaning = RequiredOutputPublicationMeaning {
            implementation: RequiredOutputProducingImplementation::from_checkpoint(
                [0x11; 16], [0x22; 32],
            )
            .unwrap(),
            family: "generic-output".into(),
            source_scope: source,
            source_partition: worth_foundational::facade::CanonicalDigestId::new([0; 32]),
            source_epoch: worth_foundational::facade::CanonicalDigestId::new([1; 32]),
        };
        let (_, before) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let output = publish(execution, &runtime, &owner, &installed, &meaning, "output");
        (
            Self {
                runtime,
                owner,
                installed,
                model,
                source,
                output,
                meaning,
            },
            before,
        )
    }
}

pub(super) fn publish(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    owner: &worth_proof::InstalledApplicationPublicationOwner,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    meaning: &RequiredOutputPublicationMeaning,
    key: &str,
) -> EntityId {
    publish_records(execution, runtime, owner, installed, meaning, key, false).0
}
pub(super) fn publish_related(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    owner: &worth_proof::InstalledApplicationPublicationOwner,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    meaning: &RequiredOutputPublicationMeaning,
    key: &str,
) -> (EntityId, worth_relational::facade::identity::RelationId) {
    let (entity, relation) =
        publish_records(execution, runtime, owner, installed, meaning, key, true);
    (entity, relation.unwrap())
}
fn publish_records(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    owner: &worth_proof::InstalledApplicationPublicationOwner,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    meaning: &RequiredOutputPublicationMeaning,
    key: &str,
    related: bool,
) -> (
    EntityId,
    Option<worth_relational::facade::identity::RelationId>,
) {
    let mut tx = begin(runtime);
    let generated = CreatedEntityRef {
        partition_id: PartitionId(4),
        kind_id: KindId(3),
        client_key: ClientKey::raw(key),
    };
    let mut batch = WorkerIntentBatch::new("generator").push(MutationIntent::Create(
        CreateIntent::Entity(EntitySpec {
            partition_id: generated.partition_id,
            kind_id: generated.kind_id,
            client_key: generated.client_key.clone(),
            fields: AspectFieldPatch::default(),
        }),
    ));
    let relation = CreatedRelationRef {
        partition_id: PartitionId(4),
        kind_id: KindId(4),
        client_key: ClientKey::raw(format!("{key}:relation")),
        source: EntityReference::Existing(meaning.source_scope),
        target: EntityReference::Created(generated.clone()),
    };
    if related {
        batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: relation.partition_id,
                kind_id: relation.kind_id,
                client_key: relation.client_key.clone(),
                source: EntityReference::Existing(meaning.source_scope),
                target: EntityReference::Created(generated.clone()),
                fields: AspectFieldPatch::default(),
            },
        )));
    }
    tx.push_batch(batch, execution.allocation_policy()).unwrap();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let absence = runtime
        .readmit_required_output_route_absence(execution, installed, &basis, meaning)
        .unwrap();
    let proposal = tx
        .validate(execution, runtime)
        .unwrap()
        .bind_required_output_publication(
            installed,
            RequiredOutputPublication::Birth {
                admission: owner.admit_required_output("generator", [1; 32]),
                absence,
                meaning: meaning.clone(),
                roles: vec![RequiredOutputRoleTarget {
                    role: "result".into(),
                    entity_name: "Member".into(),
                    kind: KindId(3),
                    posture: RequiredOutputRolePosture::Create,
                    target: EntityReference::Created(generated.clone()),
                }],
            },
        )
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("genuine producer publication must perform");
    };
    let born = runtime.settle_performed_publication(performed).unwrap();
    let output = born.created_entity(&generated).unwrap();
    let edge = related.then(|| born.created_relation(&relation).unwrap());
    release_test_commit_snapshot(runtime, &born);
    (output, edge)
}

pub(super) fn replace_related(
    execution: &ExecutionRequest<'_, '_>,
    world: &PublishedModel,
    meaning: &RequiredOutputPublicationMeaning,
    old: EntityId,
    old_edge: worth_relational::facade::identity::RelationId,
) -> (EntityId, worth_relational::facade::identity::RelationId) {
    let (_, basis) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    let prior = world
        .runtime
        .prepare_historical_output_selection(execution, &world.installed, &basis, &basis, meaning)
        .unwrap();
    let predecessor = prior.publication().clone();
    let admission = world
        .owner
        .admit_required_output_succession(
            world.owner.admit_required_output("generator", [2; 32]),
            predecessor.authority(),
        )
        .unwrap();
    let generated = CreatedEntityRef {
        partition_id: PartitionId(4),
        kind_id: KindId(3),
        client_key: ClientKey::raw("replacement"),
    };
    let relation = CreatedRelationRef {
        partition_id: PartitionId(4),
        kind_id: KindId(4),
        client_key: ClientKey::raw("replacement:relation"),
        source: EntityReference::Existing(meaning.source_scope),
        target: EntityReference::Created(generated.clone()),
    };
    let batch = WorkerIntentBatch::new("generator")
        .push(MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent {
                relation_id: old_edge,
            },
        )))
        .push(MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id: old },
        )))
        .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: generated.partition_id,
            kind_id: generated.kind_id,
            client_key: generated.client_key.clone(),
            fields: AspectFieldPatch::default(),
        })))
        .push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: relation.partition_id,
                kind_id: relation.kind_id,
                client_key: relation.client_key.clone(),
                source: relation.source.clone(),
                target: relation.target.clone(),
                fields: AspectFieldPatch::default(),
            },
        )));
    let mut tx = begin(&world.runtime);
    tx.push_batch(batch, execution.allocation_policy()).unwrap();
    let mut meaning = meaning.clone();
    meaning.source_epoch = worth_foundational::facade::CanonicalDigestId::new([2; 32]);
    let proposal = tx
        .validate(execution, &world.runtime)
        .unwrap()
        .bind_required_output_publication(
            &world.installed,
            RequiredOutputPublication::Preserve {
                admission,
                predecessor,
                meaning,
                roles: vec![
                    RequiredOutputRoleTarget {
                        role: "result".into(),
                        entity_name: "Member".into(),
                        kind: KindId(3),
                        posture: RequiredOutputRolePosture::Retire,
                        target: EntityReference::Existing(old),
                    },
                    RequiredOutputRoleTarget {
                        role: "replacement".into(),
                        entity_name: "Member".into(),
                        kind: KindId(3),
                        posture: RequiredOutputRolePosture::Create,
                        target: EntityReference::Created(generated.clone()),
                    },
                ],
            },
        )
        .unwrap();
    let prepared = world
        .runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = world
        .runtime
        .publication_port()
        .compare_and_publish(execution, prepared)
    else {
        panic!("owning producer replacement must publish");
    };
    let committed = world
        .runtime
        .settle_performed_publication(performed)
        .unwrap();
    let entity = committed.created_entity(&generated).unwrap();
    let edge = committed.created_relation(&relation).unwrap();
    release_test_commit_snapshot(&world.runtime, &committed);
    (entity, edge)
}
