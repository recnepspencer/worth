use crate::domain_computation::primary_graph::{
    application_attempt::{Movement, WorthQueryApplicationObservedFact as Fact},
    tests::{
        application_attempt::{authenticated_principal, resolved_account},
        fixture::{
            installed_authorization_world, live_scope, publish_relational_mutation, AccountLabel,
            AccountStatus, AuthorizationWorld, TouchAccountOperation,
        },
    },
};
use crate::domain_computation::primary_graph::{
    WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding,
};
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

mod selection_denies_invalid_work_and_overflow_budgets_without_retai;

#[test]
fn distinct_absent_predicates_survive_sealing_and_compare_only_their_own_matches() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let actor = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .authorize_operation(
                    &phase,
                    &actor,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let (_, projection, work) = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, _| {
                        for value in ["missing-a", "missing-b"] {
                            assert!(reader
                                .decision_select_entities(
                                    AccountStatus::reference(),
                                    value.to_owned(),
                                    2,
                                )
                                .unwrap()
                                .is_empty());
                        }
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(work.equality_lookups(), 2);
    assert_eq!(work.index_candidates_examined(), 0);
    let reads = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .complete_projected_dependencies(
                    &phase,
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let predicates = reads
        .facts
        .iter()
        .filter(|fact| matches!(fact, Fact::IndexedEntitySelection { .. }))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        predicates.len(),
        2,
        "the same index must retain both predicates"
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_eq!(
                currentness(
                    &phase,
                    &host_custody.borrowed_request_for_test(),
                    &world,
                    &predicates[0]
                ),
                (true, 1)
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_eq!(
                currentness(
                    &phase,
                    &host_custody.borrowed_request_for_test(),
                    &world,
                    &predicates[1]
                ),
                (true, 1)
            )
        })
        .expect("the fixture read owns its installed host call");

    world
        .application
        .with_host_advancement(|phase| set_unrelated_status(&phase, &world, "missing-a"))
        .expect("the fixture read owns its installed host call");
    let results = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            predicates
                .iter()
                .map(|fact| {
                    currentness(
                        &phase,
                        &host_custody.borrowed_request_for_test(),
                        &world,
                        fact,
                    )
                })
                .collect::<Vec<_>>()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        results,
        vec![(false, 2), (true, 1)],
        "one predicate changes while the other survives the same index generation update"
    );
}

fn currentness(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    fact: &Fact,
) -> (bool, usize) {
    let selected = world.selected_product(&(phase_custody).test_host_request());
    world
        .application
        .primary_provider
        .graph
        .with_runtime(execution, |runtime| {
            let (movement, work) = fact
                .source_currentness_within(
                    runtime,
                    execution,
                    selected.application_basis().snapshot_handle(),
                    3,
                )
                .unwrap();
            (movement.movement() == Movement::Unmoved, work)
        })
}

fn set_unrelated_status(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    status: &str,
) {
    let field = AccountStatus::reference();
    set_unrelated_value(
        phase_custody,
        world,
        status,
        (field.entity(), field.aspect(), field.field()),
    );
}

fn set_unrelated_value(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    status: &str,
    field: (&str, &str, &str),
) {
    let account = resolved_account(
        &(phase_custody).test_host_request(),
        world,
        "unrelated",
        &live_scope(),
    );
    let installed = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout
        .field_locator(field.0, field.1, field.2)
        .unwrap();
    let locator = worth_foundational::facade::AspectFieldLocator::new(
        worth_foundational::facade::LocatorAuthority::Authoritative,
        installed.aspect().aspect_key().clone(),
        installed.field_path().clone(),
    );
    let fields = AspectFieldPatch::from(std::collections::BTreeMap::from([(
        locator,
        StringApplicationValueBinding::encode(&status.to_owned()).unwrap(),
    )]));
    publish_relational_mutation(
        &phase_custody.borrowed_request_for_test(),
        world,
        WorkerIntentBatch::new("indexed-selection-currentness").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: account.entity_id(),
                fields,
            }),
        )),
    );
}
