pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::payload::sequence::FactSequence;
pub use worth_foundational::facade::AspectFieldLocator;
pub use worth_foundational::facade::CanonicalFieldPath;
pub use worth_foundational::facade::FieldKey;
pub use worth_foundational::facade::LocatorAuthority;
pub use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
pub use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
pub use worth_relational::facade::transactions::AspectFieldPatch;
pub use worth_relational::facade::transactions::EntityMutationIntent;
pub use worth_relational::facade::transactions::MutationIntent;
pub use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
pub use worth_relational::facade::transactions::WorkerIntentBatch;

pub use std::sync::Arc;
pub use std::collections::BTreeSet;
pub use worth_relational::facade::mvcc::CompanionPreflightStop;
pub use worth_query_execution::facade::application_inspection::FactlessCurrentness;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::payload::native::NativeFactPayloadBuilder;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::OwnEffect;
pub use worth_query_execution::facade::application_inspection::OwnEffectOnReads;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::PreparedSourceFactRebase;
pub use worth_query_execution::facade::application_inspection::RebaseVerificationReason;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::RebasedSourceFacts;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::rebase;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::movement::FactMovement;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::movement::Movement;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::source_currentness::WorthQuerySourceCurrentnessFailure;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_reuse::selection::compare_retained_output_dependencies;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_reuse::selection::OutputDependencySelection;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountNote;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

#[test]
fn a_source_revision_the_effect_moved_is_kept_and_reported_superseded() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let (entity, locator) = world
        .application
        .with_host_advancement(|phase| account_note(&phase, &world, "account-1"))
        .expect("the fixture read owns its installed host call");
    let read = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            rebase_at_current(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                vec![WorthQueryApplicationObservedFact::AbsentField {
                    entity_id: entity,
                    kind: (*(*world.application.runtime_for_test())
                        .primary_graph()
                        .unwrap()
                        .layout_for_test())
                    .entity_kind_for_test(AccountIdentity::reference().entity())
                    .unwrap(),
                    locator: planned(&locator),
                }],
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            assert_eq!(
                rebase_result_at_current(
                    &phase,
                    &host_custody.borrowed_request_for_test(),
                    &world,
                    read.iter().cloned().collect()
                ),
                RebasedSourceFacts::Exact(read.clone()),
                "a source revision nothing moved stays exact"
            )
        })
        .expect("the fixture read owns its installed host call");

    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            set_note(
                &host_custody.borrowed_request_for_test(),
                &world,
                entity,
                locator,
                "moved",
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            assert_eq!(
                rebase_result_at_current(
                    &phase,
                    &host_custody.borrowed_request_for_test(),
                    &world,
                    read.iter().cloned().collect()
                ),
                RebasedSourceFacts::SupersededByOwnEffect {
                    facts: read.clone(),
                    ordinals: Arc::from([0]),
                },
                "the fact keeps the revision its source read and is reported stale"
            )
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn unavailable_native_revision_never_authorizes_output_reuse() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let (entity, note) = world
        .application
        .with_host_advancement(|phase| account_note(&phase, &world, "account-1"))
        .expect("the fixture read owns its installed host call");
    let undeclared = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        note.aspect().aspect_key().clone(),
        CanonicalFieldPath::single(FieldKey::new("undeclared").unwrap()),
    );
    let rebased = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            rebase_result_at_current(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                vec![WorthQueryApplicationObservedFact::AbsentField {
                    entity_id: entity,
                    kind: (*(*world.application.runtime_for_test())
                        .primary_graph()
                        .unwrap()
                        .layout_for_test())
                    .entity_kind_for_test(AccountIdentity::reference().entity())
                    .unwrap(),
                    locator: undeclared,
                }],
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        rebased,
        RebasedSourceFacts::VerificationRequired {
            reason,
            ..
        } if reason.reason_for_test() == &RebaseVerificationReason::NativeFactRevisionUnavailable
            && worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::mark_state::FullVerificationReason::from(reason.clone())
                == worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::mark_state::FullVerificationReason::NativeFactRevisionUnavailable(0)
    ));
}

pub fn account_note(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    key: &str,
) -> (
    worth_relational::facade::identity::EntityId,
    AspectFieldLocator,
) {
    let entity = world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .resolve_entity(
            phase_custody,
            AccountIdentity::reference(),
            key.to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id_for_test();
    let note = AccountNote::reference();
    let locator = (*(*world.application.runtime_for_test())
        .primary_graph()
        .unwrap()
        .layout_for_test())
    .field_locator_for_test(note.entity(), note.aspect(), note.field())
    .unwrap()
    .clone();
    (entity, locator)
}

pub fn planned(locator: &AspectFieldLocator) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        locator.aspect().aspect_key().clone(),
        locator.field_path().clone(),
    )
}

pub fn rebase_at_current(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    facts: Vec<WorthQueryApplicationObservedFact>,
) -> worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::payload::FactPayload{
    match rebase_result_at_current(phase_custody, execution, world, facts) {
        RebasedSourceFacts::Exact(facts) => facts,
        other => panic!("fresh native facts must be exact: {other:?}"),
    }
}

pub fn current(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    fact: &WorthQueryApplicationObservedFact,
) -> bool {
    currentness(phase_custody, execution, world, fact)
        .unwrap()
        .movement_for_test()
        == Movement::Unmoved
}

pub fn output_selection(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    facts: &(impl FactSequence + ?Sized),
) -> OutputDependencySelection {
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    (*world.application.runtime_for_test())
        .primary_graph()
        .unwrap()
        .integration_handle_for_test()
        .with_runtime(execution, |runtime| {
            compare_retained_output_dependencies(
                execution,
                runtime,
                selected.application_basis_for_test().snapshot_handle(),
                true,
                Some(facts),
            )
            .unwrap()
        })
}

pub fn currentness(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    fact: &WorthQueryApplicationObservedFact,
) -> Result<FactMovement, WorthQuerySourceCurrentnessFailure> {
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    (*world.application.runtime_for_test())
        .primary_graph()
        .unwrap()
        .integration_handle_for_test()
        .with_runtime(execution, |runtime| {
            fact.source_currentness_in_for_test(
                runtime,
                execution,
                selected.application_basis_for_test().snapshot_handle(),
            )
            .unwrap()
        })
}

pub fn set_note(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    entity: worth_relational::facade::identity::EntityId,
    locator: AspectFieldLocator,
    value: &str,
) {
    let fields = AspectFieldPatch::from(std::collections::BTreeMap::from([(
        locator,
        StringApplicationValueBinding::encode(&value.to_owned()).unwrap(),
    )]));
    publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new("decision-field-currentness").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields,
            }),
        )),
    );
}

pub fn rebase_result_at_current(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    facts: Vec<WorthQueryApplicationObservedFact>,
) -> RebasedSourceFacts {
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());

    (*world
        .application
        .runtime_for_test())
        .primary_graph()
        .unwrap()
        .integration_handle_for_test()
        .with_runtime(execution, |runtime| {
            rebase(execution, runtime, selected.application_basis_for_test().snapshot_handle(), PreparedSourceFactRebase::admit_for_test(facts, [].into(), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None)).unwrap(), &BTreeSet::new(), true)
        })
}

#[path = "counted_read_stops/mod.rs"]
pub mod counted_read_stops;
#[path = "failed_rebase/mod.rs"]
pub mod failed_rebase;
#[path = "indexed_denial/mod.rs"]
pub mod indexed_denial;
#[path = "producer_decision_absence_uses_native_presence_revision/mod.rs"]
pub mod producer_decision_absence_uses_native_presence_revision;
#[path = "producer_decision_field_uses_native_revision_after_value_aba/mod.rs"]
pub mod producer_decision_field_uses_native_revision_after_value_aba;
