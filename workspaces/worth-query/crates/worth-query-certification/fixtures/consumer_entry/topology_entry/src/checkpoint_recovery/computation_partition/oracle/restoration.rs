//! Restoration follows a seeded edit prefix and compares actual published state.
use super::*;
use worth_query_host::facade::application_contribution::{
    WorthQueryPartitionedComputationFullCause as Cause,
    WorthQueryPartitionedComputationRunForTest as Run,
};

#[cfg(test)]
mod serial {
    use super::*;
    #[test]
    fn restoration_after_seeded_edits_rebuilds_fresh_state() {
        let _guard = checkpoint_recovery_test_guard();
        let seed = |graph: &mut Graph| differential::prefix::model().seed(graph);
        let policy = worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, None),
        );
        let original = installation::install_variant_with_observations::<false, TOTALS_WORK, 1, 0>(
            None,
            Default::default(),
            0,
            policy,
            seed,
        );
        let (model, _, mut history) = differential::prefix::run_with_history(&original);
        let checkpoint =
            worth_query_testkit_host::facade::host::checkpoint_capture::capture(&original);
        drop(original);
        let application = installation::install_variant_with_observations::<false, TOTALS_WORK, 1, 0>(
            Some(checkpoint),
            Default::default(),
            0,
            policy,
            seed,
        );
        let fresh = history
            .install::<false, TOTALS_WORK, 1, 0>(Default::default(), |graph, model| {
                model.seed(graph)
            });
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
        assert_eq!(kept[0].calls, model.expected_calls(None));
        assert_eq!(kept[0].calls, reference[0].calls);
        assert_eq!(kept[0].outcome, reference[0].outcome);
        assert_published_state(&kept, &reference);
    }
}
