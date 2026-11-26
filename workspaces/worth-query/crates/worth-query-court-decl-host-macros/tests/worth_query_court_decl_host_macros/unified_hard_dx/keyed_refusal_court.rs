//! A missing key is a typed `NotFound` and a held unique key a typed
//! `AlreadyExists`, each naming its entity, field and key, instead of a
//! generic broken-rule refusal.
use worth_foundational::facade::AspectValue;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_entry::WorthQueryApplicationRequestMutationDenial,
    primary_graph::{
        HandlerExecutionDenial, MutationHandlerExecutionDenial, WorthQueryEntityResolutionDenial,
        WorthQueryEntityResolutionDenialKind, WorthQueryEntityResolutionRefusalKind,
        WorthQueryKeyedEntityOutcome as Outcome, WorthQueryKeyedEntityRefusal,
    },
    WorthQueryRefusalCategory as Category, WorthQuerySessionRunDenial, WorthQueryUserFacingRefusal,
};

use super::actions::SetTaskState;
use super::body_links::NewProject;
use super::program::owner;
use super::sol_round2_policy_and_custody_courts::open_memory_home_with_policy;

fn assert_keyed(
    keyed: Option<&WorthQueryKeyedEntityRefusal>,
    outcome: Outcome,
    entity: &str,
    field: &str,
    key: AspectValue,
) {
    let keyed = keyed.expect("the refusal names its entity key");
    assert_eq!(keyed.outcome(), outcome);
    assert_eq!(keyed.entity(), entity);
    assert_eq!(keyed.field(), field);
    assert_eq!(keyed.key(), Some(&key));
}

#[test]
fn a_held_unique_key_is_already_exists_and_a_missing_key_is_not_found() {
    let clock = WorthQueryAdmissionClock::system();
    let home = open_memory_home_with_policy(&clock, None);
    let me = owner(&home, &clock);
    let session = home.session(&me);

    let duplicate = session
        .run(NewProject {
            parent: 1,
            id: 2,
            label: "Second backlog".to_owned(),
        })
        .unwrap_err();
    assert_eq!(duplicate.refusal_category(), Category::Conflict);
    assert!(duplicate.user_message().contains("already exists"));
    assert_keyed(
        duplicate.keyed_entity(),
        Outcome::AlreadyExists,
        "Project",
        "ProjectKey",
        AspectValue::UInt64(2),
    );

    let missing = session
        .run(SetTaskState {
            task: 99,
            state: "DONE".to_owned(),
        })
        .unwrap_err();
    assert_eq!(missing.refusal_category(), Category::NotFound);
    assert_keyed(
        missing.keyed_entity(),
        Outcome::NotFound,
        "Task",
        "TaskKey",
        AspectValue::UInt64(99),
    );

    // A fresh key still creates.
    session
        .run(NewProject {
            parent: 1,
            id: 3,
            label: "Third".to_owned(),
        })
        .unwrap();
}

/// An application's own handler error that wraps a `resolve_entity` miss.
#[derive(Debug)]
struct StockDenial(WorthQueryEntityResolutionDenial);

impl std::fmt::Display for StockDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "stock lookup failed: {}", self.0)
    }
}

impl std::error::Error for StockDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[test]
fn a_handler_resolve_miss_is_not_found_even_through_an_application_error() {
    let miss = || {
        WorthQueryEntityResolutionDenial::new(
            WorthQueryEntityResolutionDenialKind::Refused(
                WorthQueryEntityResolutionRefusalKind::UnknownEntity,
            ),
            "Sku",
        )
        .with_keyed_entity(WorthQueryKeyedEntityRefusal::not_found(
            "Item",
            "Sku",
            Some(AspectValue::UInt64(7)),
        ))
    };
    for handler in [
        HandlerExecutionDenial::new(miss()),
        HandlerExecutionDenial::new(StockDenial(miss())),
    ] {
        let run: WorthQuerySessionRunDenial<std::convert::Infallible> =
            WorthQuerySessionRunDenial::Request(
                WorthQueryApplicationRequestMutationDenial::Handler(
                    MutationHandlerExecutionDenial::Handler(handler),
                ),
            );
        assert_eq!(run.refusal_category(), Category::NotFound);
        assert_keyed(
            run.keyed_entity(),
            Outcome::NotFound,
            "Item",
            "Sku",
            AspectValue::UInt64(7),
        );
    }

    // A handler stop without keyed evidence is still the application's rule.
    let plain =
        MutationHandlerExecutionDenial::Handler(HandlerExecutionDenial::new(std::fmt::Error));
    assert_eq!(plain.refusal_category(), Category::Validation);
    assert!(plain.keyed_entity().is_none());
}
