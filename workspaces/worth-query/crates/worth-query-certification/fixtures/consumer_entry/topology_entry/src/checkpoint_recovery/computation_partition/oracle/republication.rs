//! A real suspended producer output republishes facts without computation state.

use super::*;
use worth_query_host::facade::application_contribution::{
    WorthQueryPartitionedComputationFullCause as Cause, WorthQueryPartitionedComputationRun as Run,
};

fn seed(graph: &mut Graph) {
    differential::prefix::model().seed(graph);
}

#[test]
fn a_republished_output_runs_in_full_with_the_republication_cause() {
    let _guard = checkpoint_recovery_test_guard();
    let application =
        installation::install_variant::<false, TOTALS_WORK, 1, 2>(None, Default::default(), seed);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (model, initial, mut history) = differential::prefix::run_with_history(&application);
    let total = initial.last().unwrap().outcome.as_ref().unwrap().0;
    republish(&application, total);
    let change = differential::prefix::value_edit(&model);
    history.edit(&differential::alphabet::Change::Entry(change.clone()));
    edit(&request, &application, change.clone(), 9903);
    let (_, kept) = demand(&request, &application);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].runs, [Run::Full(Cause::Republished)]);
    assert_eq!(kept[0].calls, model.expected_calls(None));
    let fresh = history
        .install::<false, TOTALS_WORK, 1, 2>(Default::default(), |graph, model| model.seed(graph));
    let (fresh_scope, fresh_principal) = authenticate(&fresh);
    let fresh_request = fresh.request(&fresh_principal, &fresh_scope);
    let (_, reference) = demand(&fresh_request, &fresh);
    assert_eq!(kept[0].outcome, reference[0].outcome);
    assert_eq!(kept[0].calls, reference[0].calls);
    assert_published_state(&kept, &reference);
    history.demanded(None);
    differential::prefix::work_boundary(&application, model, history);
}

pub(super) fn republish(application: &Application<false, TOTALS_WORK, 1, 2>, total: u64) {
    let (scope, principal) = authenticate(application);
    let request = application.request(&principal, &scope);
    let source = request
        .query(PlanarRead {
            body_key: SCOPE.to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let value = request
        .query(PlanarOutputRead {
            body_key: SCOPE.to_owned(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .value;
    let suspended = application
        .on_branch(application.current_world())
        .select()
        .unwrap()
        .suspend_current_generated_output::<RegionOutputProducer<CheckpointSchema, false, 2>>(
            &scope, source,
        )
        .unwrap_or_else(|_| panic!("the generated region output suspends"));
    let mut reconstruction = application
        .reconstruct_generated_output::<RegionOutputProducer<CheckpointSchema, false, 2>>(suspended)
        .unwrap_or_else(|_| panic!("the region output reconstructs"));
    use worth_query_host::facade::primary_graph::WorthQueryReconstructedOutputEntity;
    let keys = [
        format!("region:{SCOPE}:{total}:a"),
        format!("region:{SCOPE}:{total}:b"),
        format!("region:{SCOPE}:{total}:c"),
    ];
    let mut entities = Vec::new();
    for (key, (x, y)) in keys.iter().zip([(1, 1), (2, 1), (1, 2)]) {
        let WorthQueryReconstructedOutputEntity::Generated(entity) = reconstruction
            .output_member::<PlanarCreatedOutputs<CheckpointSchema>>(key)
            .unwrap()
        else {
            panic!("the ring is generated");
        };
        reconstruction
            .field(&entity, BodyKey::reference(), key.clone())
            .unwrap();
        reconstruction
            .field(&entity, PositionX::reference(), length(x))
            .unwrap();
        reconstruction
            .field(&entity, PositionY::reference(), length(y))
            .unwrap();
        reconstruction
            .field(&entity, Length::reference(), value)
            .unwrap();
        entities.push(entity);
    }
    for index in 0..3 {
        reconstruction
            .relation(
                PlanarSuccessor::reference(),
                &entities[index],
                &entities[(index + 1) % 3],
            )
            .unwrap();
    }
    let completed = reconstruction
        .finish()
        .unwrap_or_else(|_| panic!("the ring reconstruction contains every generated record"));
    application
        .restore_generated_output(completed, &scope)
        .unwrap_or_else(|_| panic!("the reconstructed output restores"));
}
