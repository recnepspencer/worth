use worth_store_formal_models::{map_compaction_observation, CompactionVisibilityAction};
use worth_store_physical_isolation::CompactionReadPlanCompletion;
use worth_store_test_support::harness::physical_isolation::compaction::{
    admitted_compaction_plan, published_compaction,
};

#[test]
fn completed_local_plans_map_to_cutover_validation_not_recovery_visibility() {
    let plan = admitted_compaction_plan();
    let pre = plan.source_integrity().stable_read_receipt().unwrap();
    let publication = published_compaction(plan);
    let post = publication
        .plan_post_cutover_read()
        .unwrap()
        .into_execution_ready_handle()
        .complete_plan();
    let completion =
        CompactionReadPlanCompletion::from_publication(publication, pre, post).unwrap();
    assert_eq!(
        map_compaction_observation(completion.owner_case_observation()),
        CompactionVisibilityAction::ValidateReadPlanCutover
    );
}
