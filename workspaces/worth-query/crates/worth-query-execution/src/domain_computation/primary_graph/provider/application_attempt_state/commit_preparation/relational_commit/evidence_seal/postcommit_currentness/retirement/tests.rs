use super::*;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::{
    endpoints::AdmittedAdjacencyEndpoints, StorageControl,
};
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;
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

mod own_retirement_preserves_each_read_locator_and_checkpoint_curren;

#[test]
fn external_deleted_input_and_live_adjacency_anchor_keep_their_own_comparison() {
    let world = installed_authorization_world(true);
    world
        .application
        .with_host_advancement(|phase| {
            let borrowed = phase.borrowed_request_for_test();
            let execution = &borrowed;
            let host_custody = phase.test_host_request();

            let (owned, _, own_reads) = reads(
                &host_custody.phase_for_application(&world.application),
                execution,
                &world,
                "account-2",
            );
            let (external, _, external_reads) = reads(
                &host_custody.phase_for_application(&world.application),
                execution,
                &world,
                "account-1",
            );
            delete(&host_custody.borrowed_request_for_test(), &world, owned);
            let facts = [own_reads, external_reads.clone()].concat();
            let rebased = rebase_current(
                &host_custody.phase_for_application(&world.application),
                &host_custody.borrowed_request_for_test(),
                &world,
                facts,
                &BTreeSet::from([owned]),
            )
            .unwrap();
            assert_eq!(
                &rebased[8..],
                external_reads.as_slice(),
                "external anchors and fields are not converted to retirement truth"
            );
            let native = normal_rebase(
                &host_custody.phase_for_application(&world.application),
                execution,
                &world,
                rebased.clone(),
            );
            assert!(selection(
                &host_custody.phase_for_application(&world.application),
                execution,
                &world,
                &native
            ));
            delete(&host_custody.borrowed_request_for_test(), &world, external);
            assert!(
                !selection(
                    &host_custody.phase_for_application(&world.application),
                    execution,
                    &world,
                    &native
                ),
                "an independent retirement of an external input invalidates the whole output"
            );
        })
        .expect("the fixture block owns its installed host call")
}

fn reads(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    key: &str,
) -> (EntityId, KindId, Vec<Fact>) {
    let selected = world.selected_product(&(phase_custody).test_host_request());
    let request = live_scope();
    let entity = selected
        .resolve_entity(
            phase_custody,
            AccountIdentity::reference(),
            key.to_owned(),
            &request,
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
    let (aspect_revision, field_revision, adjacency_revision, relations) = graph
        .integration_handle()
        .with_runtime(execution, |runtime| {
            let view = runtime
                .read_truth()
                .project_snapshot(selected.application_basis().snapshot_handle())
                .unwrap();
            (
                view.entity_aspect_version(entity, locator.aspect().aspect_key()),
                view.entity_field_revision(execution, entity, &locator)
                    .expect("fixture read request remains live"),
                view.bounded_adjacency_structural_revision(
                    execution,
                    entity,
                    relation,
                    worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing,
                    16,
                )
                .unwrap()
                .revision(),
                crate::domain_computation::primary_graph::application_attempt::observe_adjacency(
                    runtime,
                    execution,
                    selected.application_basis().snapshot_handle(),
                    relation,
                    entity,
                    WorthQueryApplicationAdjacencyDirection::Outgoing,
                    16,
                )
                .unwrap()
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
            endpoints: AdmittedAdjacencyEndpoints::from_exact_iterator(
                relations.len(),
                relations.iter().map(|relation| relation.to),
                StorageControl::new(
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    Some(&request),
                ),
            )
            .expect("the unchanged ordered relation endpoints are admitted"),
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

fn delete(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    entity: EntityId,
) {
    publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new("owned-output-retirement").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: entity }),
        )),
    );
}

fn rebase_current(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    facts: Vec<Fact>,
    retired: &BTreeSet<EntityId>,
) -> Option<Vec<Fact>> {
    let selected = world.selected_product(&(phase_custody).test_host_request());
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_runtime(execution, |runtime| {
            rebase(
                runtime,
                selected.application_basis().snapshot_handle(),
                facts,
                retired,
            )
        })
}

fn normal_rebase(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    facts: Vec<Fact>,
) -> crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload {
    let selected = world.selected_product(&(phase_custody).test_host_request());
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_runtime(execution, |runtime| {
            exact(super::super::rebase(execution,
                runtime,
                selected.application_basis().snapshot_handle(),
                super::super::PreparedSourceFactRebase::admit(facts, [].into(), crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None)).unwrap(),
                &BTreeSet::new(),
                true,
                Some(64),
                &mut admission(),
            ))
        })
}

/// Facts that need full verification are not exact facts of the commit.
fn exact(
    rebased: super::super::RebasedSourceFacts,
) -> crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload {
    rebased
        .retain_exact()
        .unwrap_or_else(|| std::sync::Arc::<[Fact]>::from([]).into())
}

fn admission(
) -> crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission
{
    crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission::new(
        worth_relational::facade::mvcc::CompanionPreflightBudget {
            maximum_work_visits: Some(64),
            maximum_preparation_bytes: None,
        },
    )
}

fn selection(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    facts: &(impl FactSequence + ?Sized),
) -> bool {
    let selected = world.selected_product(&(phase_custody).test_host_request());
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_runtime(execution, |runtime| {
            matches!(
                compare_retained_output_dependencies(
                    execution,
                    runtime,
                    selected.application_basis().snapshot_handle(),
                    true,
                    Some(facts),
                    &mut world
                        .application
                        .primary_provider
                        .graph
                        .source_owner
                        .invalidation_owner
                        .edit_admission_with_work_limit(Some(64)),
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
