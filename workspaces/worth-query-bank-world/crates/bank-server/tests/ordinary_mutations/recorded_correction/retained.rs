//! The installed retained action owns correction delivery and exact replay close.
use super::*;
use worth_query_host::facade::primary_graph::{
    CompositeAncestorSelectionDenial, ProductBranchAncestorSelectionDenial,
    RuntimeWorldServiceDenial,
};

#[test]
fn closed_source_correction_and_replay_keep_the_exact_retained_commit() {
    let world = fixture::World::new(9);
    let (source, _) = world.send(30, "retained-original");
    let source = source.coordinates();
    world.send(7, "retained-intervening");
    let scope = request_scope();
    let request = world.native.runtime.request(&world.teller, &scope);
    // Retain the optional source history independently of either operation's close custody.
    let selected_source = request
        .corrections()
        .select_coordinates(&source, limits())
        .unwrap();
    let prepared = world
        .native
        .runtime
        .prepare_send_money_correction(
            &world.teller,
            &scope,
            &source,
            "retained-reversal",
            limits(),
        )
        .unwrap();
    let WorthQueryOperationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("first inverse must prepare without publication");
    };
    assert_eq!(world.balances(), (63, 37));
    let WorthQuerySourceOnlyAdvance::Completed(completed) = prepared.publish().unwrap() else {
        panic!("fresh correction completes with its retained observation");
    };
    let WorthQuerySourceCompletion::Fresh {
        receipt,
        retained: Some(retained),
        ..
    } = completed.evidence()
    else {
        panic!("installed delivery retains the actual performed commit");
    };
    let commit = receipt.clone();
    assert_eq!(
        retained.selected_commit(),
        commit.committed_product_publication().composite_commit()
    );
    assert_eq!(world.balances(), (93, 7));
    let activity = world.activity(world.source);
    assert!(completed.close(&request).is_ok());

    world.send(3, "retained-after-correction");
    let source_activity = world.activity(world.source);
    let destination_activity = world.activity(world.destination);
    let replay = world
        .native
        .runtime
        .prepare_send_money_correction(
            &world.teller,
            &scope,
            &source,
            "retained-reversal",
            limits(),
        )
        .unwrap();
    let WorthQueryOperationCorrectionPreparation::Continued(WorthQuerySourceOnlyAdvance::Progress(
        replay,
    )) = replay
    else {
        panic!("exact retry retains the original native history delivery");
    };
    let WorthQuerySourceOnlyAdvance::Completed(replayed) = replay.advance(&request, None) else {
        panic!("original correction observation is still retained");
    };
    let WorthQuerySourceCompletion::Recovered {
        receipt,
        retained: Some(retained),
        ..
    } = replayed.evidence()
    else {
        panic!("retry delivers original evidence rather than a new result");
    };
    assert_eq!(receipt.correction_source(), commit.correction_source());
    assert_eq!(
        retained.selected_commit(),
        commit.committed_product_publication().composite_commit()
    );
    assert_eq!(world.balances(), (90, 10));
    assert_eq!(world.activity(world.source).len(), activity.len() + 1);
    assert_eq!(world.activity(world.source), source_activity);
    assert_eq!(world.activity(world.destination), destination_activity);
    assert!(replayed.close(&request).is_ok());

    drop(selected_source);
    world.send(1, "retained-source-released");
    assert!(matches!(
        request.corrections().select_coordinates(&source, limits()),
        Err(WorthQueryApplicationCorrectionDenial::Source(
            WorthQueryCorrectionSelectionDenial::Ancestry(RuntimeWorldServiceDenial::Denied(
                ProductBranchAncestorSelectionDenial::History(
                    CompositeAncestorSelectionDenial::SourceUnavailable
                )
            ))
        ))
    ));
    assert_eq!(world.balances(), (89, 11));
}
