//! Sealed correction metadata survives real optional history retirement.
use crate::domain_computation::primary_graph::tests::fixture::correction_source;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationIdempotencyResolution, WorthQueryCorrectionSelectionDenial,
};
use worth_runtime_world::facade::{
    CompositeAncestorSelectionDenial, ProductBranchAncestorSelectionDenial,
};

#[test]
fn a_causal_replay_after_its_history_retires_answers_with_its_sealed_causality() {
    let world = correction_source::install();
    let source = correction_source::edit(&world, "retirement-source", "edited-label");
    let prepared = correction_source::prepare(&world, &source, "retirement-correction");
    let binding = prepared.binding;
    let original = correction_source::committed(correction_source::publish(&world, prepared));
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
    drop(correction_source::edit(
        &world,
        "retirement-later-1",
        "later-label-1",
    ));
    drop(correction_source::edit(
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
            let scope = super::super::super::fixture::live_scope();
            let principal = super::super::super::fixture::authenticate_external(
                world.application.installed_schema(),
                "alice",
                std::time::Duration::from_secs(120),
                &scope,
            );
            let target = world.selected_product(&phase.test_host_request());
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
    let WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(recovered) =
        correction_source::replay(&world, binding)
    else {
        panic!("fresh admitted idempotency resolution recovers the original causal receipt");
    };
    assert_eq!(recovered.aftermath_causality(), Some(&causality));
    assert_eq!(
        recovered.committed_product_publication().composite_commit(),
        &causal_commit
    );
}
