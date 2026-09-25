use std::num::NonZeroUsize;
use std::time::Duration;

use worth_foundational::facade::{AspectFieldLocator, CanonicalFieldPath, LocatorAuthority};
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding,
};
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

use super::super::fixture::{
    installed_authorization_world, live_scope, AccountIdentity, AccountLabel, AccountNote,
    AuthorizationWorld, OptionalAccountFieldQuery, OptionalAccountFieldResult,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationObservedFact, WorthQueryApplicationQueryAccessContext,
    WorthQueryPrincipalResolutionMode,
};

#[test]
fn optional_result_field_distinguishes_present_value_from_lawful_absence() {
    let world = installed_authorization_world(true);

    let present = execute(&world, "account-1");
    assert_eq!(present.note(), Some("reviewed"));
    assert_eq!(present.score(), None);
    assert_eq!(present.annotation(), None);
    let absent = execute(&world, "account-2");
    assert_eq!(absent.note(), None);
    assert_eq!(absent.score(), None);
    assert_eq!(absent.annotation(), None);
}

#[test]
fn absent_optional_source_stays_current_across_sibling_edit_but_not_presence_change() {
    let world = installed_authorization_world(true);
    let (_, absent_fact, account) = execute_with_note_fact(&world, "account-2");
    let WorthQueryApplicationObservedFact::SourceFieldRevision {
        native_revision: Some(absent),
        ..
    } = &absent_fact
    else {
        panic!("the query must record a native revision for its absent field")
    };
    assert_eq!(
        absent.presence(),
        worth_relational::facade::runtime::RelationalFieldPresence::Absent
    );
    assert!(current(&world, &absent_fact));

    let graph = world.application.runtime.primary_graph().unwrap();
    let label = AccountLabel::reference();
    let label_locator = graph
        .layout
        .field_locator(label.entity(), label.aspect(), label.field())
        .unwrap()
        .clone();
    set_string_field(&world, account, label_locator, "renamed");
    assert!(
        current(&world, &absent_fact),
        "another field of the same aspect cannot stale native absence"
    );

    let note = AccountNote::reference();
    let note_locator = graph
        .layout
        .field_locator(note.entity(), note.aspect(), note.field())
        .unwrap()
        .clone();
    set_string_field(&world, account, note_locator, "temporary");
    assert!(
        !current(&world, &absent_fact),
        "an absent-to-present edit must stale the source"
    );
}

#[test]
fn present_optional_source_denies_equal_value_after_aba() {
    let world = installed_authorization_world(true);
    let (before, fact, account) = execute_with_note_fact(&world, "account-1");
    assert_eq!(before.note(), Some("reviewed"));
    let graph = world.application.runtime.primary_graph().unwrap();
    let note = AccountNote::reference();
    let locator = graph
        .layout
        .field_locator(note.entity(), note.aspect(), note.field())
        .unwrap()
        .clone();
    set_string_field(&world, account, locator.clone(), "temporary");
    set_string_field(&world, account, locator, "reviewed");
    assert_eq!(execute(&world, "account-1").note(), Some("reviewed"));
    assert!(
        !current(&world, &fact),
        "returning to the same value cannot restore the old native revision"
    );
}

fn current(world: &AuthorizationWorld, fact: &WorthQueryApplicationObservedFact) -> bool {
    let selected = world.selected_product();
    let graph = world.application.runtime.primary_graph().unwrap();
    graph.integration_handle().with_runtime(|runtime| {
        fact.source_currentness_in(runtime, selected.application_basis().snapshot_handle(), 1)
            .unwrap()
            .0
    })
}

fn set_string_field(
    world: &AuthorizationWorld,
    entity: worth_relational::facade::identity::EntityId,
    locator: AspectFieldLocator,
    value: &str,
) {
    let fields = AspectFieldPatch::from(std::collections::BTreeMap::from([(
        locator,
        StringApplicationValueBinding::encode(&value.to_owned()).unwrap(),
    )]));
    super::super::fixture::publish_relational_mutation(
        world,
        WorkerIntentBatch::new("optional-field-revision").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields,
            }),
        )),
    );
}

fn execute(world: &AuthorizationWorld, account: &str) -> OptionalAccountFieldResult {
    execute_with_note_fact(world, account).0
}

fn execute_with_note_fact(
    world: &AuthorizationWorld,
    account: &str,
) -> (
    OptionalAccountFieldResult,
    WorthQueryApplicationObservedFact,
    worth_relational::facade::identity::EntityId,
) {
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let scope = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountIdentity::reference(),
            account.to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .certification_query(OptionalAccountFieldQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let plan = world
        .selected_product()
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::<OptionalAccountFieldQuery>::new(),
            crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
                NonZeroUsize::new(1).unwrap(),
                NonZeroUsize::new(256).unwrap(),
                &request,
            ),
        )
        .unwrap();
    let result = world
        .application
        .execute_application_query_one_shot(plan)
        .unwrap();
    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].account(), account);
    assert_eq!(result.receipt().projected_field_count(), 4);
    assert!(result.receipt().disclosure().omitted().is_empty());
    let footprint = result.observed_sources()[0].footprint_for_test();
    assert_eq!(footprint.aspects.len(), 4);
    assert_eq!(
        footprint
            .aspects
            .iter()
            .filter(|aspect| aspect
                .native_revision
                .is_some_and(|revision| revision.presence()
                    == worth_relational::facade::runtime::RelationalFieldPresence::Absent))
            .count(),
        if account == "account-1" { 2 } else { 3 },
        "each absent optional field must retain an explicit native absence revision"
    );
    let note = footprint
        .aspects
        .iter()
        .find(|observed| observed.field.as_str() == AccountNote::reference().field())
        .expect("the optional note is a tracked source field");
    let fact = WorthQueryApplicationObservedFact::SourceFieldRevision {
        entity_id: note.entity,
        locator: AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            note.aspect.clone(),
            CanonicalFieldPath::single(note.field.clone()),
        ),
        native_revision: note.native_revision,
    };
    (result.rows()[0].clone(), fact, footprint.root)
}
