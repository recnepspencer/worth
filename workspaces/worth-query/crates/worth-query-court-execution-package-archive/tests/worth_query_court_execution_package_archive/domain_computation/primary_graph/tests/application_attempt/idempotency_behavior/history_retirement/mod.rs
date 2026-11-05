//! Sealed correction metadata survives real optional history retirement.
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyResolution;
use worth_query_execution::facade::domain_computation::WorthQueryCorrectionSelectionDenial;
use worth_runtime_world::facade::CompositeAncestorSelectionDenial;
use worth_runtime_world::facade::ProductBranchAncestorSelectionDenial;

#[test]
fn a_causal_replay_after_its_history_retires_answers_with_its_sealed_causality() {
    let world = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::install(worth_execution::MonotonicClock::system());
    let source = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::edit(&world, "retirement-source", "edited-label");
    let prepared = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::prepare(&world, &source, "retirement-correction");
    let binding = prepared.binding;
    let original = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::committed(worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::publish(&world, prepared));
    let causality = original.aftermath_causality().unwrap().clone();
    let causal_commit = original
        .committed_product_publication()
        .composite_commit()
        .clone();
    let coordinates = original.correction_source().coordinates();
    // Publication consumed the preparation. Release every remaining selected
    // history/receipt handle before making retirement, rather than just age, observable.
    drop(original);
    drop(source);
    drop(worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::edit(
        &world,
        "retirement-later-1",
        "later-label-1",
    ));
    drop(worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::edit(
        &world,
        "retirement-later-2",
        "later-label-2",
    ));
    world
        .application
        .with_host_advancement(|phase| {
            let history = world
                .application
                .branches()
                .history(
                    &phase,
                    world.application.current_world(),
                    std::num::NonZeroUsize::new(64),
                )
                .unwrap();
            assert!(
                history
                    .entries()
                    .all(|entry| entry.selected_commit() != &causal_commit),
                "later real publications must retire the correction commit"
            );
            drop(history);
            let scope = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
            let principal = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authentication::authenticate_external(
                world.application.installed_schema(),
                "alice",
                std::time::Duration::from_secs(120),
                &scope,
            );
            let target = world.selected_product(&phase.test_host_request_for_test());
            let denial = target
                .select_correction_source_coordinates(
                    &phase,
                    &principal,
                    &scope,
                    &coordinates,
                    None,
                    std::num::NonZeroUsize::MIN,
                )
                .err()
                .expect("a copied coordinate cannot pin expired history");
            assert!(
                matches!(
                    denial,
                    WorthQueryCorrectionSelectionDenial::Ancestry(
                        worth_runtime_world::facade::RuntimeWorldServiceDenial::Denied(
                            ProductBranchAncestorSelectionDenial::History(
                                CompositeAncestorSelectionDenial::SourceUnavailable
                            )
                        )
                    )
                ),
                "fresh expired selection must fail at native history: {denial:?}"
            );
        })
        .unwrap();
    let WorthQueryApplicationIdempotencyResolution::RecoveredCommitted(recovered) =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::replay(&world, binding)
    else {
        panic!("fresh admitted idempotency resolution recovers the original causal receipt");
    };
    crate::domain_computation::primary_graph::tests::fixture::recorded_receipt::assert_causality(
        recovered.aftermath_causality().unwrap(),
        &causality,
    );
    assert_eq!(
        recovered
            .new_product_head()
            .memory()
            .unwrap()
            .selected_commit(),
        causal_commit.ordinal()
    );
}
