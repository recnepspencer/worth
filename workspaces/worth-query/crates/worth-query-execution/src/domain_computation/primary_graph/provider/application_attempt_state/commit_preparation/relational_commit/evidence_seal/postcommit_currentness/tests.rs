use worth_foundational::facade::{
    AspectFieldLocator, CanonicalFieldPath, FieldKey, LocatorAuthority,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding,
};
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQuerySourceCurrentnessFailure,
    output_reuse::{compare_retained_output_dependencies, OutputDependencySelection},
    tests::fixture::{
        installed_authorization_world, live_scope, publish_relational_mutation, AccountIdentity,
        AccountNote, AuthorizationWorld,
    },
    WorthQueryPrincipalResolutionMode,
};

#[test]
fn producer_decision_field_uses_native_revision_after_value_aba() {
    let world = installed_authorization_world(true);
    let (entity, locator) = account_note(&world, "account-1");
    let field = WorthQueryApplicationObservedFact::Field {
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
        value: StringApplicationValueBinding::encode(&"reviewed".to_owned()).unwrap(),
    };
    let facts = rebase_at_current(&world, vec![field.clone()]);
    let durable = crate::domain_computation::primary_graph::application_checkpoint::decode_producer_facts(
        &crate::domain_computation::primary_graph::application_checkpoint::encode_producer_facts(&facts)
            .expect("the rebased non-query producer decision field is checkpoint-comparable"),
    )
    .expect("the complete producer fact set round-trips");
    assert_eq!(durable.as_ref(), facts.as_ref());
    assert!(matches!(
        facts[0],
        WorthQueryApplicationObservedFact::SourceFieldRevision {
            native_revision: Some(_),
            ..
        }
    ));
    assert!(current(&world, &facts[0]));
    assert!(matches!(
        output_selection(&world, &durable),
        OutputDependencySelection::Reuse
    ));
    assert_eq!(
        currentness(&world, &field),
        Err(WorthQuerySourceCurrentnessFailure::Unavailable),
        "a value-only fact cannot authorize output reuse"
    );

    set_note(&world, entity, locator.clone(), "temporary");
    set_note(&world, entity, locator, "reviewed");
    assert!(!current(&world, &facts[0]));
    assert!(matches!(
        output_selection(&world, &durable),
        OutputDependencySelection::FreshRequired
    ));
}

#[test]
fn producer_decision_absence_uses_native_presence_revision() {
    let world = installed_authorization_world(true);
    let (entity, locator) = account_note(&world, "account-2");
    let fact = WorthQueryApplicationObservedFact::AbsentField {
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
    };
    let facts = rebase_at_current(&world, vec![fact]);
    assert!(matches!(
        facts[0],
        WorthQueryApplicationObservedFact::SourceFieldRevision {
            native_revision: Some(revision),
            ..
        } if revision.presence() == worth_relational::facade::runtime::RelationalFieldPresence::Absent
    ));
    assert!(current(&world, &facts[0]));
    assert!(matches!(
        output_selection(&world, &facts),
        OutputDependencySelection::Reuse
    ));
    set_note(&world, entity, locator, "now-present");
    assert!(!current(&world, &facts[0]));
    assert!(matches!(
        output_selection(&world, &facts),
        OutputDependencySelection::FreshRequired
    ));
}

#[test]
fn unavailable_native_revision_never_authorizes_output_reuse() {
    let world = installed_authorization_world(true);
    let (entity, note) = account_note(&world, "account-1");
    let undeclared = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        note.aspect().aspect_key().clone(),
        CanonicalFieldPath::single(FieldKey::new("undeclared").unwrap()),
    );
    let facts = rebase_at_current(
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
    );
    assert!(matches!(
        facts[0],
        WorthQueryApplicationObservedFact::SourceFieldRevision {
            native_revision: None,
            ..
        }
    ));
    assert!(matches!(
        output_selection(&world, &facts),
        OutputDependencySelection::FreshRequired
    ));
}

fn account_note(
    world: &AuthorizationWorld,
    key: &str,
) -> (
    worth_relational::facade::identity::EntityId,
    AspectFieldLocator,
) {
    let entity = world
        .selected_product()
        .resolve_entity(
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

fn planned(locator: &AspectFieldLocator) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        locator.aspect().aspect_key().clone(),
        locator.field_path().clone(),
    )
}

fn rebase_at_current(
    world: &AuthorizationWorld,
    facts: Vec<WorthQueryApplicationObservedFact>,
) -> Arc<[WorthQueryApplicationObservedFact]> {
    let selected = world.selected_product();
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_runtime(|runtime| {
            rebase(
                runtime,
                selected.application_basis().snapshot_handle(),
                facts,
                true,
                64,
            )
        })
}

fn current(world: &AuthorizationWorld, fact: &WorthQueryApplicationObservedFact) -> bool {
    currentness(world, fact).unwrap().0
}

fn output_selection(
    world: &AuthorizationWorld,
    facts: &[WorthQueryApplicationObservedFact],
) -> OutputDependencySelection {
    let selected = world.selected_product();
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_runtime(|runtime| {
            compare_retained_output_dependencies(
                runtime,
                selected.application_basis().snapshot_handle(),
                true,
                Some(facts),
                &mut 16,
            )
            .unwrap()
        })
}

fn currentness(
    world: &AuthorizationWorld,
    fact: &WorthQueryApplicationObservedFact,
) -> Result<(bool, usize), WorthQuerySourceCurrentnessFailure> {
    let selected = world.selected_product();
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_runtime(|runtime| {
            fact.source_currentness_in(runtime, selected.application_basis().snapshot_handle(), 1)
        })
}

fn set_note(
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
        world,
        WorkerIntentBatch::new("decision-field-currentness").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields,
            }),
        )),
    );
}
