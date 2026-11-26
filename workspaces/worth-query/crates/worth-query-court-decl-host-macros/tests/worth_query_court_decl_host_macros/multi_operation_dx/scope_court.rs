//! Scope posture: a query scoped on a mutable, non-unique field of its own
//! root is refused at declaration, naming the field; an action whose scope key
//! matches two entities says so rather than reporting a generic defect; and a
//! keyed refusal labels an entity-prefixed field once.
use worth_query_decl::facade::application_query::ApplicationQueryBinding;
use worth_query_decl::facade::application_schema::fragment::declaration_builder;
use worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial as Denial;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    primary_graph::WorthQueryKeyedEntityOutcome as Outcome, WorthQueryRefusalCategory as Category,
    WorthQueryUserFacingRefusal,
};

use super::adders::AddOne;
use super::counter;
use super::program::{open, owner};
use super::schema::*;

#[test]
fn a_query_scoped_on_a_mutable_non_unique_field_of_its_root_is_refused_at_declaration() {
    let refused = declaration_builder(BoardSchema)
        .expect("the board declares")
        .bound_application_query::<CountersByLabelBinding>(CountersByLabelQuery::definition())
        .build()
        .expect_err("a shared, mutable label cannot select one counter");
    let Denial::RootScopedQueryRequiresKeyField {
        binding,
        query,
        field,
    } = &refused
    else {
        panic!("refused for the root scope's uniqueness: {refused:?}");
    };
    assert_eq!(binding, CountersByLabelBinding::IDENTITY);
    assert_eq!(query, "counters_by_label");
    assert_eq!((field.entity(), field.field()), ("Counter", "Label"));
    // The explanation travels in the denial's rendering.
    assert!(refused
        .to_string()
        .contains("RootScopedQueryRequiresKeyField"));

    // Its unique-keyed twin is the board's own read, which builds. (A
    // `read_only` identity key is admitted too: the macros task board scopes
    // `correction_task` on its read-only `TaskKey`.)
    declaration_builder(BoardSchema)
        .expect("the board declares")
        .build()
        .expect("a root scope on a unique key is admitted");
}

#[test]
fn an_action_whose_scope_key_matches_two_entities_explains_the_ambiguity() {
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&clock);
    let session = home.session(owner(&home, &clock));

    let denial = session
        .run(SetByLabel {
            label: "pair".to_owned(),
            value: 5,
        })
        .expect_err("two counters share the label");
    let summary = denial.user_summary();
    assert_eq!(summary.category(), Category::Internal);
    assert!(
        summary
            .user_message()
            .contains("More than one item matches"),
        "{}",
        summary.user_message()
    );
    assert_ne!(summary.user_message(), Category::Internal.default_message());
    assert_eq!(counter(&home, &clock, 2).value, 10);

    // A label held by one counter still selects it.
    session
        .run(SetByLabel {
            label: "solo".to_owned(),
            value: 5,
        })
        .expect("a unique label value selects its counter");
    assert_eq!(counter(&home, &clock, 4).value, 5);
}

#[test]
fn a_keyed_refusal_names_an_entity_prefixed_field_once() {
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&clock);
    let session = home.session(owner(&home, &clock));

    let missing = session
        .run(AddOne { counter: 99 })
        .expect_err("no counter 99");
    let keyed = missing.keyed_entity().expect("the refusal names its key");
    assert_eq!(keyed.outcome(), Outcome::NotFound);
    assert_eq!(
        (keyed.entity(), keyed.field()),
        ("Counter", "CounterCounterId")
    );
    assert_eq!(keyed.field_label(), "counter id");
}
