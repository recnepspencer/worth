use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact as Fact,
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

#[test]
fn selection_denies_invalid_work_and_overflow_budgets_without_retaining_partial_results() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let label = AccountLabel::reference();
    set_unrelated_value(
        &world,
        "primary",
        (label.entity(), label.aspect(), label.field()),
    );
    let account = resolved_account(&world, "open", &request);
    let actor = authenticated_principal(&world, &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(&actor, &account, &operation, Default::default(), &request)
        .unwrap();
    let over_budget = world
        .invariant
        .project_admitted_operation(&admission, |reader, _| {
            let denial = reader
                .decision_select_entities(AccountLabel::reference(), "primary".to_owned(), 32)
                .unwrap_err()
                .downcast::<WorthQueryEntityResolutionDenial>()
                .unwrap();
            assert_eq!(
                denial.kind(),
                WorthQueryEntityResolutionDenialKind::ProjectionWorkBudgetExceeded
            );
        });
    assert!(
        over_budget.is_err(),
        "an exhausted projection cannot be sealed after a handler ignores its denial"
    );
    let (_, projection, work) = world
        .invariant
        .project_admitted_operation(&admission, |reader, _| {
            for (limit, expected) in [
                (
                    0,
                    WorthQueryEntityResolutionDenialKind::InvalidCandidateLimit,
                ),
                (
                    usize::MAX,
                    WorthQueryEntityResolutionDenialKind::InvalidCandidateLimit,
                ),
                (
                    1,
                    WorthQueryEntityResolutionDenialKind::CandidateLimitExceeded { maximum: 1 },
                ),
            ] {
                let denial = reader
                    .decision_select_entities(
                        AccountLabel::reference(),
                        "primary".to_owned(),
                        limit,
                    )
                    .unwrap_err()
                    .downcast::<WorthQueryEntityResolutionDenial>()
                    .unwrap();
                assert_eq!(denial.kind(), expected);
            }
        })
        .unwrap()
        .into_parts();
    assert_eq!(
        work.equality_lookups(),
        1,
        "invalid budgets deny before index work"
    );
    assert_eq!(work.index_candidates_examined(), 1);
    let reads = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap()
        .complete_projected_dependencies()
        .unwrap();
    assert!(
        !reads
            .facts
            .iter()
            .any(|fact| matches!(fact, Fact::IndexedEntitySelection { .. })),
        "an overflowing prefix cannot become a complete source fact"
    );
}

#[test]
fn distinct_absent_predicates_survive_sealing_and_compare_only_their_own_matches() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let actor = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(&actor, &account, &operation, Default::default(), &request)
        .unwrap();
    let (_, projection, work) = world
        .invariant
        .project_admitted_operation(&admission, |reader, _| {
            for value in ["missing-a", "missing-b"] {
                assert!(reader
                    .decision_select_entities(AccountStatus::reference(), value.to_owned(), 2,)
                    .unwrap()
                    .is_empty());
            }
        })
        .unwrap()
        .into_parts();
    assert_eq!(work.equality_lookups(), 2);
    assert_eq!(work.index_candidates_examined(), 0);
    let reads = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap()
        .complete_projected_dependencies()
        .unwrap();
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
    assert_eq!(currentness(&world, &predicates[0]), (true, 1));
    assert_eq!(currentness(&world, &predicates[1]), (true, 1));

    set_unrelated_status(&world, "missing-a");
    let results = predicates
        .iter()
        .map(|fact| currentness(&world, fact))
        .collect::<Vec<_>>();
    assert_eq!(
        results,
        vec![(false, 2), (true, 1)],
        "one predicate changes while the other survives the same index generation update"
    );
}

fn currentness(world: &AuthorizationWorld, fact: &Fact) -> (bool, usize) {
    let selected = world.selected_product();
    world
        .application
        .primary_provider
        .graph
        .with_runtime(|runtime| {
            fact.source_currentness_in(runtime, selected.application_basis().snapshot_handle(), 3)
                .unwrap()
        })
}

fn set_unrelated_status(world: &AuthorizationWorld, status: &str) {
    let field = AccountStatus::reference();
    set_unrelated_value(
        world,
        status,
        (field.entity(), field.aspect(), field.field()),
    );
}

fn set_unrelated_value(world: &AuthorizationWorld, status: &str, field: (&str, &str, &str)) {
    let account = resolved_account(world, "unrelated", &live_scope());
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
        world,
        WorkerIntentBatch::new("indexed-selection-currentness").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: account.entity_id(),
                fields,
            }),
        )),
    );
}
