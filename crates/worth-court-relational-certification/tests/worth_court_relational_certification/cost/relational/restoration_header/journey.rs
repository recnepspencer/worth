//! A public Native create/grow/retire/restore journey with independent route width.
use std::time::{Duration, Instant};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId},
    mvcc::*,
    runtime::*,
    schema::{register_current_publication_storage, RelationalSchemaRegistry},
    symbols::ClientKey,
    transactions::*,
};
use worth_testkit_relational::aspect_storage::{
    aspect_key, field_key,
    publications::{
        begin_main_transaction, release_test_commit_snapshot, string_aspect_field_patch,
    },
    AspectSchemaFixture,
};

pub(super) struct Journey {
    pub(super) restored: CommitResult,
    pub(super) seconds: [Duration; 4],
}

pub(super) fn run(
    execution: &ExecutionRequest<'_, '_>,
    publications: usize,
    members: usize,
    edges: usize,
) -> Journey {
    let fixture = AspectSchemaFixture::with_default_declared_aspects(
        worth_relational::facade::config::CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let registry = register_current_publication_storage(
        fixture.build_registry(),
        &fixture.schema_id,
        fixture.schema_version_id,
    )
    .unwrap();
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .unwrap();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let installed =
        installed.with_source_scope_owned_producers([("scale-realizer".to_owned(), KindId(1))]);
    let start = Instant::now();
    let source = target("authored-source");
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            creates(std::slice::from_ref(&source)),
            execution.allocation_policy(),
        )
        .unwrap();
    let born = transaction.commit(execution, &runtime).unwrap();
    let source_id = born.created_entity(&source).unwrap();
    release_test_commit_snapshot(&runtime, &born);
    let mut all_members = Vec::new();
    let mut all_edges = Vec::new();
    let mut priors = Vec::new();
    let mut route_source = source_id;
    for route in 0..publications {
        let meaning = meaning(route, route_source);
        let targets = output_targets(route, 0, members);
        let (batch, relations) = create_batch(route_source, &targets, route, 0, edges);
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let absence = runtime
            .readmit_required_output_route_absence(execution, &installed, &basis, &meaning)
            .unwrap();
        let born = publish(
            &runtime,
            execution,
            &installed,
            batch,
            RequiredOutputPublication::Birth {
                admission: owner.admit_required_output("scale-realizer", [1; 32]),
                absence,
                meaning,
                roles: targets
                    .iter()
                    .enumerate()
                    .map(|(i, t)| {
                        role(
                            i,
                            EntityReference::Created(t.clone()),
                            RequiredOutputRolePosture::Create,
                        )
                    })
                    .collect(),
            },
        );
        let ids = collect(
            &runtime,
            &born,
            &targets,
            &relations,
            &mut all_members,
            &mut all_edges,
        );
        let next_source = ids[0];
        priors.push((
            born.required_output_publication(&installed).unwrap(),
            ids,
            route_source,
        ));
        route_source = next_source;
    }
    let created = start.elapsed();
    let start = Instant::now();
    for (route, (prior, ids, source)) in priors.into_iter().enumerate() {
        let targets = output_targets(route, 1, members);
        let (batch, relations) = create_batch(source, &targets, route, 1, edges);
        let mut roles = ids
            .iter()
            .enumerate()
            .map(|(i, id)| {
                role(
                    i,
                    EntityReference::Existing(*id),
                    RequiredOutputRolePosture::Preserve,
                )
            })
            .collect::<Vec<_>>();
        roles.extend(targets.iter().enumerate().map(|(i, t)| {
            role(
                i + ids.len(),
                EntityReference::Created(t.clone()),
                RequiredOutputRolePosture::Create,
            )
        }));
        let admission = owner
            .admit_required_output_succession(
                owner.admit_required_output("scale-realizer", [2; 32]),
                prior.authority(),
            )
            .unwrap();
        let grown = publish(
            &runtime,
            execution,
            &installed,
            batch,
            RequiredOutputPublication::Preserve {
                admission,
                predecessor: prior,
                meaning: meaning(route, source),
                roles,
            },
        );
        collect(
            &runtime,
            &grown,
            &targets,
            &relations,
            &mut all_members,
            &mut all_edges,
        );
    }
    let grown = start.elapsed();
    let start = Instant::now();
    all_members.push(source_id);
    let roles = all_members
        .iter()
        .enumerate()
        .map(|(i, id)| {
            role(
                i,
                EntityReference::Existing(*id),
                RequiredOutputRolePosture::Retire,
            )
        })
        .collect::<Vec<_>>();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let predecessor = runtime
        .readmit_authored_source_retirement(execution, &installed, &basis, source_id, &roles)
        .unwrap()
        .unwrap();
    let admission = owner
        .admit_authored_output_retirements(predecessor.authorities())
        .unwrap();
    let mut batch = WorkerIntentBatch::new("retire-source");
    for relation_id in all_edges {
        batch = batch.push(MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent { relation_id },
        )));
    }
    for entity_id in all_members {
        batch = batch.push(MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id },
        )));
    }
    let retired = publish(
        &runtime,
        execution,
        &installed,
        batch,
        RequiredOutputPublication::AuthoredCascadeRetirement {
            admission,
            predecessor,
        },
    );
    release_test_commit_snapshot(&runtime, &retired);
    let retirement = start.elapsed();
    let start = Instant::now();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let fresh_source = target("restored-source");
    let restoration = runtime
        .prepare_authored_source_restoration(
            execution,
            &installed,
            &basis,
            retired.commit.commit_id,
            source_id,
            fresh_source.clone(),
        )
        .unwrap()
        .unwrap();
    let admission = owner
        .admit_authored_output_retirements(restoration.authorities())
        .unwrap();
    let batch = restoration
        .extend_batch(execution, creates(std::slice::from_ref(&fresh_source)))
        .unwrap();
    let restored = publish(
        &runtime,
        execution,
        &installed,
        batch,
        RequiredOutputPublication::AuthoredCascadeRestoration {
            admission,
            restoration,
        },
    );
    super::republication::complete(&runtime, execution, &installed, &owner, &restored);
    let restoration = start.elapsed();
    assert_ne!(restored.created_entity(&fresh_source).unwrap(), source_id);
    release_test_commit_snapshot(&runtime, &restored);
    Journey {
        restored,
        seconds: [created, grown, retirement, restoration],
    }
}

fn target(key: impl Into<String>) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw(key.into()),
    }
}
fn output_targets(route: usize, round: usize, members: usize) -> Vec<CreatedEntityRef> {
    (0..members)
        .map(|i| target(format!("route-{route}-round-{round}-member-{i}")))
        .collect()
}
fn fields(aspect: &str, value: &str) -> AspectFieldPatch {
    string_aspect_field_patch([(aspect_key(aspect), field_key(aspect), value)])
}
fn creates(targets: &[CreatedEntityRef]) -> WorkerIntentBatch {
    targets
        .iter()
        .fold(WorkerIntentBatch::new("authored-input"), |batch, t| {
            batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: t.partition_id,
                kind_id: t.kind_id,
                client_key: t.client_key.clone(),
                fields: fields("name", "record"),
            })))
        })
}
fn role(
    index: usize,
    target: EntityReference,
    posture: RequiredOutputRolePosture,
) -> RequiredOutputRoleTarget {
    RequiredOutputRoleTarget {
        role: format!("member-{index}"),
        entity_name: "node".into(),
        kind: KindId(1),
        posture,
        target,
    }
}
fn meaning(route: usize, source: EntityId) -> RequiredOutputPublicationMeaning {
    RequiredOutputPublicationMeaning {
        implementation: RequiredOutputProducingImplementation::from_checkpoint(
            [0x11; 16], [0x22; 32],
        )
        .unwrap(),
        family: format!("route-{route}"),
        source_scope: source,
        source_partition: worth_foundational::facade::CanonicalDigestId::new([0; 32]),
        source_epoch: worth_foundational::facade::CanonicalDigestId::new([1; 32]),
    }
}
fn create_batch(
    source: EntityId,
    targets: &[CreatedEntityRef],
    route: usize,
    round: usize,
    edges: usize,
) -> (WorkerIntentBatch, Vec<CreatedRelationRef>) {
    let mut batch = creates(targets);
    let mut relations = Vec::new();
    assert!(edges <= targets.len() * targets.len());
    for i in 0..edges {
        let source_index = i % targets.len();
        let group = i / targets.len();
        let target = if group == 0 {
            EntityReference::Existing(source)
        } else {
            EntityReference::Created(targets[(source_index + group) % targets.len()].clone())
        };
        let edge = CreatedRelationRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: ClientKey::raw(format!("route-{route}-round-{round}-edge-{i}")),
            source: EntityReference::Created(targets[i % targets.len()].clone()),
            target,
        };
        batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: edge.partition_id,
                kind_id: edge.kind_id,
                client_key: edge.client_key.clone(),
                source: edge.source.clone(),
                target: edge.target.clone(),
                fields: fields("label", "association"),
            },
        )));
        relations.push(edge);
    }
    (batch, relations)
}
fn collect(
    runtime: &RelationalRuntime,
    result: &CommitResult,
    targets: &[CreatedEntityRef],
    relations: &[CreatedRelationRef],
    members: &mut Vec<EntityId>,
    edges: &mut Vec<worth_relational::facade::identity::RelationId>,
) -> Vec<EntityId> {
    let ids = targets
        .iter()
        .map(|t| result.created_entity(t).unwrap())
        .collect::<Vec<_>>();
    members.extend(&ids);
    edges.extend(
        relations
            .iter()
            .map(|r| result.created_relation(r).unwrap()),
    );
    release_test_commit_snapshot(runtime, result);
    ids
}
pub(super) fn publish(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    batch: WorkerIntentBatch,
    publication: RequiredOutputPublication,
) -> CommitResult {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let proposal = transaction
        .validate(execution, runtime)
        .unwrap()
        .bind_required_output_publication(installed, publication)
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("public journey must publish")
    };
    runtime.settle_performed_publication(performed).unwrap()
}
