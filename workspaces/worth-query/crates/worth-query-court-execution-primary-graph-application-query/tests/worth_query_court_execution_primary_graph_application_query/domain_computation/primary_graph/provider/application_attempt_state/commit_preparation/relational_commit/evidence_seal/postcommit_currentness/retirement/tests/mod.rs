pub use std::collections::BTreeSet;
pub use worth_relational::facade::identity::EntityId;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact as Fact;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::retirement::rebase;
pub use crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::retirement::tests;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationSourceAdjacencyEndpoints as AdmittedAdjacencyEndpoints;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::payload::sequence::FactSequence;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationAdjacencyDirection;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_reuse::selection::compare_retained_output_dependencies;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_reuse::selection::OutputDependencySelection;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountAllActivity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountNote;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
pub use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
pub use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
pub use worth_relational::facade::identity::KindId;
pub use worth_relational::facade::transactions::DeleteEntityIntent;
pub use worth_relational::facade::transactions::EntityMutationIntent;
pub use worth_relational::facade::transactions::MutationIntent;
pub use worth_relational::facade::transactions::WorkerIntentBatch;

#[test]
fn external_deleted_input_and_live_adjacency_anchor_keep_their_own_comparison() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    world
        .application
        .with_host_advancement(|phase| {
            let borrowed = phase.borrowed_request_for_test();
            let execution = &borrowed;
            let host_custody = phase.test_host_request_for_test();

            let (owned, _, own_reads) = reads(
                &host_custody.phase_for_application_for_test(&world.application),
                execution,
                &world,
                "account-2",
            );
            let (external, _, external_reads) = reads(
                &host_custody.phase_for_application_for_test(&world.application),
                execution,
                &world,
                "account-1",
            );
            delete(&host_custody.borrowed_request_for_test(), &world, owned);
            let facts = [own_reads, external_reads.clone()].concat();
            let rebased = rebase_current(
                &host_custody.phase_for_application_for_test(&world.application),
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
                &host_custody.phase_for_application_for_test(&world.application),
                execution,
                &world,
                rebased.clone(),
            );
            assert!(selection(
                &host_custody.phase_for_application_for_test(&world.application),
                execution,
                &world,
                &native
            ));
            delete(&host_custody.borrowed_request_for_test(), &world, external);
            assert!(
                !selection(
                    &host_custody.phase_for_application_for_test(&world.application),
                    execution,
                    &world,
                    &native
                ),
                "an independent retirement of an external input invalidates the whole output"
            );
        })
        .expect("the fixture block owns its installed host call")
}

pub fn reads(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    key: &str,
) -> (EntityId, KindId, Vec<Fact>) {
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    let request = live_scope();
    let entity = selected
        .resolve_entity(
            phase_custody,
            AccountIdentity::at("AccountIdentity"),
            key.to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id_for_test();
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let identity = AccountIdentity::at("AccountIdentity");
    let note = AccountNote::at("AccountNote");
    let kind = (*graph.layout_for_test())
        .entity_kind_for_test(identity.entity())
        .unwrap();
    let locator = (*graph.layout_for_test())
        .field_locator_for_test(identity.entity(), identity.aspect(), identity.field())
        .unwrap()
        .clone();
    let locator = worth_foundational::facade::AspectFieldLocator::new(
        worth_foundational::facade::LocatorAuthority::Authoritative,
        locator.aspect().aspect_key().clone(),
        locator.field_path().clone(),
    );
    let absent = (*graph.layout_for_test())
        .field_locator_for_test(note.entity(), note.aspect(), note.field())
        .unwrap()
        .clone();
    let relation = (*(*graph.layout_for_test())
        .relation(AccountAllActivity::reference().name())
        .unwrap()
        .kind_for_test());
    let (aspect_revision, field_revision, adjacency_revision, relations) = graph
        .integration_handle_for_test()
        .with_runtime(execution, |runtime| {
            let view = runtime
                .read_truth()
                .project_snapshot(selected.application_basis_for_test().snapshot_handle())
                .unwrap();
            (
                view.read_entity_aspect_version(execution, entity, locator.aspect().aspect_key())
                    .unwrap(),
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
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::adjacency::observe_adjacency(
                    runtime,
                    execution,
                    selected.application_basis_for_test().snapshot_handle(),
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
            endpoints: AdmittedAdjacencyEndpoints::from_exact_iterator_for_test(
                relations.len(),
                relations.iter().map(|relation| (*relation.to_for_test())),
                StorageControl::new_for_test(
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

pub fn delete(
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

pub fn rebase_current(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    facts: Vec<Fact>,
    retired: &BTreeSet<EntityId>,
) -> Option<Vec<Fact>> {
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap()
        .integration_handle_for_test()
        .with_runtime(execution, |runtime| {
            rebase(
                runtime,
                selected.application_basis_for_test().snapshot_handle(),
                facts,
                retired,
            )
        })
}

pub fn normal_rebase(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    facts: Vec<Fact>,
) -> worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::payload::FactPayload{
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    (*world
        .application
        .runtime_for_test())
        .primary_graph_for_test()
        .unwrap()
        .integration_handle_for_test()
        .with_runtime(execution, |runtime| {
            exact(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::rebase(execution, runtime, selected.application_basis_for_test().snapshot_handle(), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::PreparedSourceFactRebase::admit_for_test(facts, [].into(), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None)).unwrap(), &BTreeSet::new(), true))
        })
}

/// Facts that need full verification are not exact facts of the commit.
pub fn exact(
    rebased: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::RebasedSourceFacts,
) -> worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::payload::FactPayload{
    rebased
        .retain_exact_for_test()
        .unwrap_or_else(|| std::sync::Arc::<[Fact]>::from([]).into())
}

pub fn selection(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    facts: &(impl FactSequence + ?Sized),
) -> bool {
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap()
        .integration_handle_for_test()
        .with_runtime(execution, |runtime| {
            matches!(
                compare_retained_output_dependencies(
                    execution,
                    runtime,
                    selected.application_basis_for_test().snapshot_handle(),
                    true,
                    Some(facts),
                )
                .unwrap(),
                OutputDependencySelection::Reuse
            )
        })
}

#[path = "external/mod.rs"]
pub mod external;
#[path = "own_retirement_preserves_each_read_locator_and_checkpoint_curren/mod.rs"]
pub mod own_retirement_preserves_each_read_locator_and_checkpoint_curren;
#[path = "selection_tests/mod.rs"]
pub mod selection_tests;
