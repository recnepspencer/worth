use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationAdjacencyDirection,
    output_reuse::{compare_retained_output_dependencies, OutputDependencySelection},
    tests::fixture::{
        installed_authorization_world, live_scope, publish_relational_mutation, AccountAllActivity,
        AccountIdentity, AccountNote, AuthorizationWorld,
    },
    WorthQueryPrincipalResolutionMode,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding,
};
use worth_relational::facade::{
    identity::KindId,
    transactions::{DeleteEntityIntent, EntityMutationIntent, MutationIntent, WorkerIntentBatch},
};

#[test]
fn own_retirement_preserves_each_read_locator_and_checkpoint_currentness() {
    let world = installed_authorization_world(true);
    let (entity, kind, facts) = reads(&world, "account-2");
    let locators = facts.iter().map(Fact::locator_identity).collect::<Vec<_>>();
    assert!(
        rebase_current(&world, facts.clone(), &BTreeSet::from([entity])).is_none(),
        "a selected retirement cannot replace reads while its actual record is live"
    );
    delete(&world, entity);
    let retired = rebase_current(&world, facts.clone(), &BTreeSet::from([entity])).unwrap();
    assert_eq!(
        retired.len(),
        facts.len(),
        "every tracked read remains accounted for"
    );
    assert_eq!(
        retired
            .iter()
            .map(Fact::locator_identity)
            .collect::<Vec<_>>(),
        locators
    );
    assert!(retired.iter().all(|fact| matches!(fact, Fact::RetiredOutputEntity { entity_id, kind: actual, .. } if *entity_id == entity && *actual == kind)));
    assert!(selection(&world, &retired));
    let encoded =
        crate::domain_computation::primary_graph::application_checkpoint::encode_producer_facts(
            &retired,
        )
        .unwrap();
    let restored =
        crate::domain_computation::primary_graph::application_checkpoint::decode_producer_facts(
            &encoded,
        )
        .unwrap();
    assert_eq!(restored.as_ref(), retired.as_slice());
    assert!(selection(&world, &restored));
    let external = rebase_current(&world, facts, &BTreeSet::new()).unwrap();
    assert!(!matches!(external[0], Fact::RetiredOutputEntity { .. }));
    assert!(
        !selection(&world, &external),
        "an external input deleted without an owned retirement remains stale"
    );
    let Fact::RetiredOutputEntity {
        created_at,
        deleted_at,
        read_locator,
        ..
    } = &retired[0]
    else {
        unreachable!()
    };
    for (changed_entity, changed_kind, changed_creation) in [
        (
            EntityId::new(
                entity.partition_id,
                entity.local_slot_value(),
                entity.generation_value() + 1,
            ),
            kind,
            *created_at,
        ),
        (entity, KindId::new(kind.as_u32() + 1), *created_at),
        (
            entity,
            kind,
            worth_relational::facade::identity::VersionId(created_at.as_u64() + 1),
        ),
        (reads(&world, "account-1").0, kind, *created_at),
    ] {
        assert!(!selection(
            &world,
            &[Fact::RetiredOutputEntity {
                entity_id: changed_entity,
                kind: changed_kind,
                created_at: changed_creation,
                deleted_at: *deleted_at,
                read_locator: read_locator.clone(),
            }]
        ));
    }
    let mut changed_deletion = retired[0].clone();
    if let Fact::RetiredOutputEntity { deleted_at, .. } = &mut changed_deletion {
        *deleted_at = worth_relational::facade::identity::VersionId(deleted_at.as_u64() + 1);
    }
    assert!(!selection(&world, &[changed_deletion]));
}

#[test]
fn external_deleted_input_and_live_adjacency_anchor_keep_their_own_comparison() {
    let world = installed_authorization_world(true);
    let (owned, _, own_reads) = reads(&world, "account-2");
    let (external, _, external_reads) = reads(&world, "account-1");
    delete(&world, owned);
    let facts = [own_reads, external_reads.clone()].concat();
    let rebased = rebase_current(&world, facts, &BTreeSet::from([owned])).unwrap();
    assert_eq!(
        &rebased[8..],
        external_reads.as_slice(),
        "external anchors and fields are not converted to retirement truth"
    );
    let native = normal_rebase(&world, rebased.clone());
    assert!(selection(&world, &native));
    delete(&world, external);
    assert!(
        !selection(&world, &native),
        "an independent retirement of an external input invalidates the whole output"
    );
}

fn reads(world: &AuthorizationWorld, key: &str) -> (EntityId, KindId, Vec<Fact>) {
    let selected = world.selected_product();
    let entity = selected
        .resolve_entity(
            AccountIdentity::reference(),
            key.to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let graph = world.application.runtime.primary_graph().unwrap();
    let identity = AccountIdentity::reference();
    let note = AccountNote::reference();
    let kind = graph.layout.entity_kind(identity.entity()).unwrap();
    let locator = graph
        .layout
        .field_locator(identity.entity(), identity.aspect(), identity.field())
        .unwrap()
        .clone();
    let locator = worth_foundational::facade::AspectFieldLocator::new(
        worth_foundational::facade::LocatorAuthority::Authoritative,
        locator.aspect().aspect_key().clone(),
        locator.field_path().clone(),
    );
    let absent = graph
        .layout
        .field_locator(note.entity(), note.aspect(), note.field())
        .unwrap()
        .clone();
    let relation = graph
        .layout
        .relation(AccountAllActivity::reference().name())
        .unwrap()
        .kind;
    let (aspect_revision, field_revision, adjacency_revision, relations) =
        graph.integration_handle().with_open_runtime(|runtime| {
            let view = runtime
                .read_truth()
                .project_snapshot(selected.application_basis().snapshot_handle())
                .unwrap();
            (
                view.entity_aspect_version(entity, locator.aspect().aspect_key()),
                view.entity_field_revision(entity, &locator),
                view.bounded_adjacency_structural_revision(
                    entity,
                    relation,
                    worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing,
                    16,
                )
                .unwrap()
                .revision(),
                crate::domain_computation::primary_graph::application_attempt::observe_adjacency(
                    runtime,
                    selected.application_basis().snapshot_handle(),
                    relation,
                    entity,
                    WorthQueryApplicationAdjacencyDirection::Outgoing,
                    16,
                )
                .unwrap(),
            )
        });
    assert!(
        field_revision.is_some(),
        "the fixture captures an actual native field revision"
    );
    let facts = vec![
        Fact::Entity {
            entity_id: entity,
            kind,
        },
        Fact::SourceEntity { entity_id: entity },
        Fact::Field {
            entity_id: entity,
            kind,
            locator: locator.clone(),
            value: StringApplicationValueBinding::encode(&key.to_owned()).unwrap(),
        },
        if key == "account-1" {
            Fact::Field {
                entity_id: entity,
                kind,
                locator: absent,
                value: StringApplicationValueBinding::encode(&"reviewed".to_owned()).unwrap(),
            }
        } else {
            Fact::AbsentField {
                entity_id: entity,
                kind,
                locator: absent,
            }
        },
        Fact::SourceAspectRevision {
            entity_id: entity,
            aspect: locator.aspect().aspect_key().clone(),
            native_revision: aspect_revision.flatten(),
        },
        Fact::SourceFieldRevision {
            entity_id: entity,
            locator,
            native_revision: field_revision,
        },
        Fact::SourceAdjacencyRevision {
            anchor: entity,
            relation_kind: relation,
            direction: worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing,
            native_revision: adjacency_revision,
            comparison_work_limit: 16,
            endpoints: relations.iter().map(|relation| relation.to).collect(),
        },
        Fact::Adjacency {
            anchor: entity,
            relation_kind: relation,
            direction: WorthQueryApplicationAdjacencyDirection::Outgoing,
            maximum_work_units: 16,
            relations,
        },
    ];
    (entity, kind, facts)
}

fn delete(world: &AuthorizationWorld, entity: EntityId) {
    publish_relational_mutation(
        world,
        WorkerIntentBatch::new("owned-output-retirement").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: entity }),
        )),
    );
}

fn rebase_current(
    world: &AuthorizationWorld,
    facts: Vec<Fact>,
    retired: &BTreeSet<EntityId>,
) -> Option<Vec<Fact>> {
    let selected = world.selected_product();
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_open_runtime(|runtime| {
            rebase(
                runtime,
                selected.application_basis().snapshot_handle(),
                facts,
                retired,
            )
        })
}

fn normal_rebase(world: &AuthorizationWorld, facts: Vec<Fact>) -> std::sync::Arc<[Fact]> {
    let selected = world.selected_product();
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_open_runtime(|runtime| {
            exact(super::super::rebase(
                runtime,
                selected.application_basis().snapshot_handle(),
                super::super::PreparedSourceFactRebase::admit(facts).unwrap(),
                &BTreeSet::new(),
                true,
                64,
                Some(&mut admission()),
            ))
        })
}

/// Facts that need full verification are not exact facts of the commit.
fn exact(rebased: super::super::RebasedSourceFacts) -> std::sync::Arc<[Fact]> {
    rebased
        .retain_exact()
        .unwrap_or_else(|| std::sync::Arc::from([]))
}

fn admission(
) -> crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission
{
    crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission::new(
        worth_relational::facade::mvcc::CompanionPreflightBudget {
            maximum_work_visits: 64,
            maximum_preparation_bytes: 1024 * 1024,
        },
    )
}

fn selection(world: &AuthorizationWorld, facts: &[Fact]) -> bool {
    let selected = world.selected_product();
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_open_runtime(|runtime| {
            matches!(
                compare_retained_output_dependencies(
                    runtime,
                    selected.application_basis().snapshot_handle(),
                    true,
                    Some(facts),
                    &mut 64,
                )
                .unwrap(),
                OutputDependencySelection::Reuse
            )
        })
}

#[path = "selection.rs"]
mod selection_tests;

#[path = "external.rs"]
mod external;
