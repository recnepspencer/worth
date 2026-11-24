//! Phase 5E 3B-2: a named producer route's successor round. The round's
//! naming plan takes its chain head's keys and partitions with its commands,
//! replaces the plan of the round it supersedes, and refuses as a structural
//! effect when it names a key the head never chose. A plan naming nothing
//! leaves the re-created records unnamed: completeness refuses the commit.
use super::*;
use std::collections::BTreeMap;
use worth_relational::facade::lineage::{
    CommitNamingPlan, InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry,
    LineageNameDenial, LineageRecordTarget, PendingBirthTarget, PendingCommitBirth,
    PersistentNameOfRecord, PersistentNameSubjectKind,
};
use worth_relational::facade::schema::*;
use worth_testkit_relational::aspect_storage::AspectSchemaFixture;

mod retry;

const NODE_SCHEME: &str = "worth.cert.coupled.node.v1";
const LINK_SCHEME: &str = "worth.cert.coupled.link.v1";
const LINK: KindId = KindId(2);
const KEYS: [&str; 2] = ["owned-output", "owned-second"];

fn scheme(
    id: &str,
    subject: PersistentNameSubjectKind,
    kind: &str,
) -> InstalledPersistentNameScheme {
    InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
        id,
        subject,
        kind,
        ["authored"],
        false,
        false,
    )
    .unwrap()
}

/// The fixture's entity kind (1) and relation kind (2), both schemed.
pub(super) fn named_registry() -> RelationalSchemaRegistry {
    let fixture = AspectSchemaFixture::with_default_declared_aspects(
        worth_relational::facade::config::CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let registry = register_current_publication_storage(
        fixture.build_registry(),
        &fixture.schema_id,
        fixture.schema_version_id,
    )
    .unwrap();
    let mut schemes = InstalledPersistentNameSchemeRegistry::new();
    let entity = PersistentNameSubjectKind::Entity;
    let relation = PersistentNameSubjectKind::Relation;
    schemes
        .register(scheme(NODE_SCHEME, entity, &fixture.entity_kind_name))
        .unwrap();
    schemes
        .register(scheme(LINK_SCHEME, relation, &fixture.relation_kind_name))
        .unwrap();
    registry
        .install_persistent_name_schemes(
            schemes,
            BTreeMap::from([
                ((entity, KindId(1)), NODE_SCHEME.to_owned()),
                ((relation, LINK), LINK_SCHEME.to_owned()),
            ]),
        )
        .unwrap()
}

/// The ordinary owned-source world over `registry`, its seeds named, with
/// every seeded entity's identity in seed order.
pub(super) fn named_world(
    execution: &ExecutionRequest<'_, '_>,
    registry: RelationalSchemaRegistry,
    seeds: WorkerIntentBatch,
) -> (World, Vec<EntityId>) {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .unwrap();
    named_world_on(execution, runtime, seeds)
}

/// The same world over an already built `runtime`.
pub(super) fn named_world_on(
    execution: &ExecutionRequest<'_, '_>,
    mut runtime: RelationalRuntime,
    seeds: WorkerIntentBatch,
) -> (World, Vec<EntityId>) {
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let installed = installed.with_source_scope_owned_producers([("owned".into(), KindId(1))]);
    let mut tx = begin_main_transaction(&runtime);
    let names = seeds.intents.iter().filter_map(|intent| match intent {
        MutationIntent::Create(CreateIntent::Entity(spec)) => Some(entity_birth(
            CreatedEntityRef {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                client_key: spec.client_key.clone(),
            },
            &format!("seed.{}", spec.client_key.as_raw_str().unwrap()),
        )),
        _ => None,
    });
    let births: Vec<PendingCommitBirth> = names.collect();
    tx.attach_naming_plan(CommitNamingPlan {
        pending_births: births.clone(),
        ..CommitNamingPlan::default()
    });
    tx.push_batch(seeds, execution.allocation_policy()).unwrap();
    let result = tx.commit(execution, &runtime).unwrap();
    let source = result.created_entity(&target("source")).unwrap();
    let carrier = result.created_entity(&target("carrier")).unwrap();
    let ids = births
        .iter()
        .map(|birth| match &birth.target {
            PendingBirthTarget::CreatedEntity(created) => result.created_entity(created).unwrap(),
            PendingBirthTarget::CreatedRelation(_) => unreachable!("seeds create entities"),
        })
        .collect();
    release_test_commit_snapshot(&runtime, &result);
    let world = World {
        runtime,
        owner,
        installed,
        source,
        carrier,
        summary: false,
        optional_struct_only: false,
    };
    (world, ids)
}

pub(super) fn seeds() -> WorkerIntentBatch {
    create("source").push(create_intent("carrier"))
}

fn entity_birth(target: CreatedEntityRef, occurrence: &str) -> PendingCommitBirth {
    PendingCommitBirth {
        target: PendingBirthTarget::CreatedEntity(target),
        scheme_id: NODE_SCHEME.to_owned(),
        occurrence_key: occurrence.to_owned(),
        role_path: "authored".to_owned(),
        parent_identities: Vec::new(),
        referenced_names: Vec::new(),
        canonical_discriminator: Vec::new(),
    }
}

fn at(key: &str, partition: PartitionId) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: partition,
        kind_id: KindId(1),
        client_key: ClientKey::raw(key),
    }
}

fn link(partition: PartitionId) -> CreatedRelationRef {
    CreatedRelationRef {
        partition_id: partition,
        kind_id: LINK,
        client_key: ClientKey::raw("owned-link"),
        source: EntityReference::Created(at(KEYS[0], partition)),
        target: EntityReference::Created(at(KEYS[1], partition)),
    }
}

/// One round's commands: both outputs and the link between them, all
/// created in `partition` (a successor's own mutation partition before the
/// fold lowers them onto its head's).
pub(super) fn round(partition: PartitionId) -> Vec<WorkerIntentBatch> {
    let mut batch = WorkerIntentBatch::new("round");
    for key in KEYS {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: partition,
            kind_id: KindId(1),
            client_key: ClientKey::raw(key),
            fields: string_aspect_field_patch([(aspect_key("name"), field_key("name"), key)]),
        })));
    }
    let link = link(partition);
    vec![batch.push(MutationIntent::Create(CreateIntent::Relation(
        RelationSpec {
            partition_id: link.partition_id,
            kind_id: link.kind_id,
            client_key: link.client_key,
            source: link.source,
            target: link.target,
            fields: AspectFieldPatch::default(),
        },
    )))]
}

/// One round's names: both outputs and their link under `occurrence`.
pub(super) fn names(partition: PartitionId, occurrence: &str) -> CommitNamingPlan {
    let mut births: Vec<PendingCommitBirth> = KEYS
        .iter()
        .map(|key| entity_birth(at(key, partition), &format!("{occurrence}.{key}")))
        .collect();
    births.push(PendingCommitBirth {
        target: PendingBirthTarget::CreatedRelation(link(partition)),
        scheme_id: LINK_SCHEME.to_owned(),
        ..entity_birth(at(KEYS[0], partition), &format!("{occurrence}.link"))
    });
    CommitNamingPlan {
        pending_births: births,
        ..CommitNamingPlan::default()
    }
}

/// The names the committed outputs and their link carry, by record.
pub(super) fn committed_names(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    result: &CommitResult,
) -> Vec<String> {
    let origins = result
        .required_output_publications(&world.installed, execution)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let mut targets: Vec<LineageRecordTarget> = origins[0]
        .members()
        .map(|member| LineageRecordTarget::Entity(member.3))
        .collect();
    assert_eq!(targets.len(), KEYS.len(), "both outputs publish");
    let outputs = targets.clone();
    let observation = world.basis().observation();
    world
        .runtime
        .with_persistent_names_at_observation(execution, &observation, |view| {
            let LineageRecordTarget::Entity(first) = outputs[0] else {
                unreachable!()
            };
            let relation = world
                .runtime
                .read_truth()
                .project_observation(&observation)
                .unwrap()
                .bounded_outgoing_relations_of_kind(execution, first, LINK, None)
                .unwrap()
                .expect("no traversal quota")
                .into_records();
            targets.push(LineageRecordTarget::Relation(relation[0].relation_id));
            targets
                .iter()
                .map(|target| match view.name_of_record(*target) {
                    PersistentNameOfRecord::Live { name, .. } => format!("{name:?}"),
                    other => panic!("{target:?} is not live under a name: {other:?}"),
                })
                .collect()
        })
        .unwrap()
}

/// Head round named under `first`, then one successor round created in its
/// own partition and named under `second` (or unnamed when `None`).
pub(super) fn fold(
    execution: &ExecutionRequest<'_, '_>,
    world: &World,
    first: &str,
    second: Option<CommitNamingPlan>,
) -> Result<CommitResult, TransactionCommitError> {
    let proposal = world.prepare(
        execution,
        &assemble(|execution, candidate| {
            candidate.append_named_publication(
                execution,
                birth(world, execution, &KEYS),
                &none(),
                round(PartitionId::main()),
                Some(names(PartitionId::main(), first)),
            )?;
            let token = candidate.next_producer_round(execution, 0)?;
            candidate.append_named_successor_publication(
                execution,
                token,
                birth(world, execution, &KEYS),
                &none(),
                round(PartitionId::new(5)),
                second.clone(),
            )
        }),
    )?;
    let candidate = world
        .runtime
        .prepare_validated_proposal(execution, proposal)?;
    let RelationalPublicationOutcome::Performed(performed) = world
        .runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("the named group performs")
    };
    Ok(world
        .runtime
        .settle_performed_publication(performed)
        .unwrap())
}

/// The one-round answer: a head round named under `occurrence`.
fn one_round(
    execution: &ExecutionRequest<'_, '_>,
    world: &World,
    occurrence: &str,
) -> CommitResult {
    let proposal = world
        .prepare(
            execution,
            &assemble(|execution, candidate| {
                candidate.append_named_publication(
                    execution,
                    birth(world, execution, &KEYS),
                    &none(),
                    round(PartitionId::main()),
                    Some(names(PartitionId::main(), occurrence)),
                )
            }),
        )
        .unwrap();
    world.publish(execution, proposal)
}

#[test]
fn a_named_successor_round_folds_onto_its_head_under_its_own_names() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let reference = named_world(execution, named_registry(), seeds()).0;
    let one = one_round(execution, &reference, "round.2");
    let expected = committed_names(&reference, execution, &one);
    release_test_commit_snapshot(&reference.runtime, &one);

    let world = named_world(execution, named_registry(), seeds()).0;
    let second = names(PartitionId::new(5), "round.2");
    let folded = fold(execution, &world, "round.1", Some(second)).unwrap();
    assert_eq!(
        committed_names(&world, execution, &folded),
        expected,
        "the successor's plan names the head's records; the superseded plan is gone"
    );
    release_test_commit_snapshot(&world.runtime, &folded);
}

#[test]
fn a_successor_round_naming_nothing_refuses_missing_birth() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = named_world(execution, named_registry(), seeds()).0;
    let before = world.basis();
    let err = fold(execution, &world, "round.1", None).unwrap_err();
    assert!(
        matches!(
            err.lineage_name_denial_ref(),
            Some(LineageNameDenial::MissingBirth { .. })
        ),
        "the superseded round's names never stand in for the successor's: {err:?}"
    );
    assert_eq!(world.basis(), before, "no effect");
}

#[test]
fn a_successor_plan_naming_a_key_its_head_never_chose_refuses() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = named_world(execution, named_registry(), seeds()).0;
    let before = world.basis();
    let mut stray = names(PartitionId::new(5), "round.2");
    stray.pending_births[0].target =
        PendingBirthTarget::CreatedEntity(at("never-created", PartitionId::new(5)));
    let err = fold(execution, &world, "round.1", Some(stray)).unwrap_err();
    assert_eq!(refusal(err), postponed(Reason::StructuralEffect));
    assert_eq!(world.basis(), before, "no effect");
}
