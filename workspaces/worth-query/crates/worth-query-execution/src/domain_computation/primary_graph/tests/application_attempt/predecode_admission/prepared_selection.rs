use super::projection;
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::StoreDenial,
    tests::fixture::{installed_authorization_world, live_scope, AccountLabel, AccountStatus},
    WorthQueryInvariantDecisionPlanDenial, WorthQueryInvariantDecisionPlanDenialKind,
};
use std::cell::Cell;
use std::time::{Duration, Instant};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestInterruption, WorthQueryRequestScope,
};

#[test]
fn prepared_decision_selection_preserves_scalar_results_and_rejects_foreign_projection() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let prepared = world
        .application
        .with_host_advancement(|phase| {
            projection::with_reader(&phase, &world, &request, |reader| {
                let denied = reader
                    .prepare_entity_selection(AccountLabel::reference())
                    .err()
                    .unwrap();
                assert_eq!(
                    denied
                        .downcast::<WorthQueryInvariantDecisionPlanDenial>()
                        .unwrap()
                        .kind(),
                    WorthQueryInvariantDecisionPlanDenialKind::UndeclaredDecisionTarget
                );
                let prepared = reader
                    .prepare_entity_selection(AccountStatus::reference())
                    .unwrap();
                for value in ["open", "absent"] {
                    let selected = reader
                        .select_entities_prepared(&prepared, value.to_owned(), 2)
                        .unwrap();
                    let scalar = reader
                        .select_entities(AccountStatus::reference(), value.to_owned(), 2)
                        .unwrap();
                    assert_eq!(
                        selected
                            .iter()
                            .map(|row| row.entity_id())
                            .collect::<Vec<_>>(),
                        scalar.iter().map(|row| row.entity_id()).collect::<Vec<_>>()
                    );
                    assert_eq!(selected.len(), usize::from(value == "open"));
                }
                prepared
            })
        })
        .expect("the fixture read owns its installed host call");
    let denied = world
        .application
        .with_host_advancement(|phase| {
            projection::with_reader(&phase, &world, &request, |reader| {
                reader.select_entities_prepared(&prepared, "open".to_owned(), 2)
            })
            .err()
            .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denied
            .downcast::<WorthQueryInvariantDecisionPlanDenial>()
            .unwrap()
            .kind(),
        WorthQueryInvariantDecisionPlanDenialKind::ForeignIdentity
    );
}

#[test]
fn prepared_decision_selection_observes_cancellation_after_preparation() {
    let world = installed_authorization_world(true);
    let source = WorthQueryCancellationSource::new();
    let request =
        WorthQueryRequestScope::new(Instant::now() + Duration::from_secs(60), source.token());
    let observed = Cell::new(false);
    let result = world
        .application
        .with_host_advancement(|phase| {
            projection::try_with_reader(&phase, &world, &request, |reader| {
                let prepared = reader
                    .prepare_entity_selection(AccountStatus::reference())
                    .unwrap();
                assert_eq!(
                    reader
                        .select_entities_prepared(&prepared, "open".to_owned(), 2)
                        .unwrap()
                        .len(),
                    1
                );
                source.cancel();
                let denied = reader
                    .select_entities_prepared(&prepared, "absent".to_owned(), 2)
                    .err()
                    .unwrap();
                assert_eq!(
                    denied.downcast::<StoreDenial>().unwrap(),
                    StoreDenial::RequestInterruption(WorthQueryRequestInterruption::Cancelled)
                );
                observed.set(true);
            })
        })
        .expect("the fixture read owns its installed host call");
    assert!(observed.get());
    assert_eq!(
        result
            .unwrap_err()
            .invariant_denial()
            .unwrap()
            .source_retention_interruption(),
        Some(WorthQueryRequestInterruption::Cancelled)
    );
}
