use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;
use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::CanonicalFieldPath;
use worth_foundational::facade::FieldKey;
use worth_foundational::facade::LocatorAuthority;
use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::EntityMutationIntent;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

use super::*;
use crate::domain_computation::primary_graph::application_attempt::FactMovement;
use crate::domain_computation::primary_graph::application_attempt::Movement;
use crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure;
use crate::domain_computation::primary_graph::output_reuse::compare_retained_output_dependencies;
use crate::domain_computation::primary_graph::output_reuse::OutputDependencySelection;
use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world;
use crate::domain_computation::primary_graph::tests::fixture::live_scope;
use crate::domain_computation::primary_graph::tests::fixture::publish_relational_mutation;
use crate::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use crate::domain_computation::primary_graph::tests::fixture::AccountNote;
use crate::domain_computation::primary_graph::tests::fixture::AuthorizationWorld;
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;

#[path = "tests/failed_rebase.rs"]
mod failed_rebase;
#[path = "tests/indexed_denial.rs"]
mod indexed_denial;

#[path = "tests/producer_decision_field_uses_native_revision_after_value_aba.rs"]
mod producer_decision_field_uses_native_revision_after_value_aba;

#[path = "tests/producer_decision_absence_uses_native_presence_revision.rs"]
mod producer_decision_absence_uses_native_presence_revision;

#[test]
fn a_source_revision_the_effect_moved_is_kept_and_reported_superseded() {
    let world = installed_authorization_world(true);
    let (entity, locator) = world
        .application
        .with_host_advancement(|phase| account_note(&phase, &world, "account-1"))
        .expect("the fixture read owns its installed host call");
    let read = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            rebase_at_current(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                vec![WorthQueryApplicationObservedFact::AbsentField {
                    entity_id: entity,
                    kind: world
                        .application
                        .runtime
                        .primary_graph()
                        .unwrap()
                        .layout
                        .entity_kind(AccountIdentity::reference().entity())
                        .unwrap(),
                    locator: planned(&locator),
                }],
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
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
    let world = installed_authorization_world(true);
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
            let host_custody = phase.test_host_request();
            rebase_result_at_current(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                vec![WorthQueryApplicationObservedFact::AbsentField {
                    entity_id: entity,
                    kind: world
                        .application
                        .runtime
                        .primary_graph()
                        .unwrap()
                        .layout
                        .entity_kind(AccountIdentity::reference().entity())
                        .unwrap(),
                    locator: undeclared,
                }],
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        rebased,
        RebasedSourceFacts::VerificationRequired {
            reason: RebaseVerificationReason::NativeFactRevisionUnavailable(0),
            ..
        }
    ));
}

pub(super) fn account_note(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    key: &str,
) -> (
    worth_relational::facade::identity::EntityId,
    AspectFieldLocator,
) {
    let entity = world
        .selected_product(&(phase_custody).test_host_request())
        .resolve_entity(
            phase_custody,
            AccountIdentity::reference(),
            key.to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let note = AccountNote::reference();
    let locator = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout
        .field_locator(note.entity(), note.aspect(), note.field())
        .unwrap()
        .clone();
    (entity, locator)
}

pub(super) fn planned(locator: &AspectFieldLocator) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        locator.aspect().aspect_key().clone(),
        locator.field_path().clone(),
    )
}

pub(super) fn rebase_at_current(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    facts: Vec<WorthQueryApplicationObservedFact>,
) -> super::FactPayload {
    match rebase_result_at_current(phase_custody, execution, world, facts) {
        RebasedSourceFacts::Exact(facts) => facts,
        other => panic!("fresh native facts must be exact: {other:?}"),
    }
}

fn current(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    fact: &WorthQueryApplicationObservedFact,
) -> bool {
    currentness(phase_custody, execution, world, fact)
        .unwrap()
        .movement()
        == Movement::Unmoved
}

fn output_selection(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    facts: &(impl FactSequence + ?Sized),
) -> OutputDependencySelection {
    let selected = world.selected_product(&(phase_custody).test_host_request());
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_runtime(execution, |runtime| {
            compare_retained_output_dependencies(
                execution,
                runtime,
                selected.application_basis().snapshot_handle(),
                true,
                Some(facts),
            )
            .unwrap()
        })
}

fn currentness(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    fact: &WorthQueryApplicationObservedFact,
) -> Result<FactMovement, WorthQuerySourceCurrentnessFailure> {
    let selected = world.selected_product(&(phase_custody).test_host_request());
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_runtime(execution, |runtime| {
            fact.source_currentness_in(
                runtime,
                execution,
                selected.application_basis().snapshot_handle(),
            )
            .unwrap()
        })
}

fn set_note(
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

#[path = "tests/counted_read_stops.rs"]
mod counted_read_stops;

pub(super) fn rebase_result_at_current(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    facts: Vec<WorthQueryApplicationObservedFact>,
) -> RebasedSourceFacts {
    let selected = world.selected_product(&(phase_custody).test_host_request());

    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_runtime(execution, |runtime| {
            rebase(execution, runtime, selected.application_basis().snapshot_handle(), PreparedSourceFactRebase::admit(facts, [].into(), crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None)).unwrap(), &BTreeSet::new(), true)
        })
}
