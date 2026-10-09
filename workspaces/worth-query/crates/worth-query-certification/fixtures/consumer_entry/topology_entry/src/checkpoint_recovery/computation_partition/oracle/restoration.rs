//! Restoration follows a seeded edit prefix and compares actual published state.
use super::*;
use worth_query_host::facade::application_contribution::{
    WorthQueryPartitionedComputationFullCause as Cause, WorthQueryPartitionedComputationRun as Run,
};

#[test]
fn restoration_after_seeded_edits_rebuilds_fresh_state_and_the_same_work_boundary() {
    let _guard = checkpoint_recovery_test_guard();
    let seed = |graph: &mut Graph| differential::prefix::model().seed(graph);
    let original = install(seed);
    let (model, _, mut history) = differential::prefix::run_with_history(&original);
    let checkpoint = original.capture_application_checkpoint().unwrap();
    drop(original);
    let application = installation::install_configured::<false, TOTALS_WORK, 1>(
        Some(checkpoint),
        Default::default(),
        seed,
    );
    let fresh = history
        .install::<false, TOTALS_WORK, 1, 0>(Default::default(), |graph, model| model.seed(graph));
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (fresh_scope, fresh_principal) = authenticate(&fresh);
    let fresh_request = fresh.request(&fresh_principal, &fresh_scope);
    let change = differential::prefix::value_edit(&model);
    history.edit(&differential::alphabet::Change::Entry(change.clone()));
    edit(&request, &application, change.clone(), 0x69_9901);
    edit(&fresh_request, &fresh, change, 0x69_9901);
    let kept = demand(&request, &application).1;
    let reference = demand(&fresh_request, &fresh).1;
    assert_eq!(kept[0].runs, [Run::Full(Cause::Restored)]);
    assert_eq!(reference[0].runs, [Run::Full(Cause::FirstRun)]);
    assert_eq!(kept[0].calls.plans, 1);
    assert_eq!(kept[0].calls.keys, model.len());
    assert_eq!(kept[0].calls, reference[0].calls);
    assert_eq!(kept[0].outcome, reference[0].outcome);
    assert_published_state(&kept, &reference);
    history.demanded(None);
    differential::prefix::work_boundary(&application, model, history);
}
