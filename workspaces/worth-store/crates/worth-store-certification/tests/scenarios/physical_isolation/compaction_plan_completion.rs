//! Local cutover correlation: these fixtures do not execute Store byte I/O.
use worth_store_physical_isolation::{
    CompactionOwnerCaseId, CompactionReadInterlockDenial, CompactionReadPlanCompletion,
    PhysicalReadPlanCompletionReceipt,
};
use worth_store_test_support::harness::physical_isolation::{
    compaction::{
        admitted_compaction_plan, admitted_compaction_plan_for_seed, published_compaction,
    },
    epoch_scope::current_generation_page_reference,
    read_plan::{admit_plan, protected_set},
};

#[test]
fn completion_binds_both_roots_and_emits_only_local_plan_evidence() {
    let plan = admitted_compaction_plan();
    let pre = plan.source_integrity().stable_read_receipt().unwrap();
    let publication = published_compaction(plan);
    let post = publication
        .plan_post_cutover_read()
        .unwrap()
        .into_execution_ready_handle()
        .complete_plan();
    let completion =
        CompactionReadPlanCompletion::from_publication(publication.clone(), pre, post).unwrap();
    assert_eq!(
        completion.pre_cutover_root(),
        publication.publication().old_root()
    );
    assert_eq!(
        completion.post_cutover_root(),
        publication.publication().new_root()
    );
    assert_eq!(completion.pre_cutover_plan_completion(), pre);
    assert_eq!(completion.post_cutover_plan_completion(), post);
    assert_eq!(
        completion.owner_case_observation().id(),
        CompactionOwnerCaseId::ValidateReadPlanCutover
    );

    let foreign_pre = admitted_compaction_plan_for_seed(18)
        .source_integrity()
        .stable_read_receipt()
        .unwrap();
    assert_eq!(
        CompactionReadPlanCompletion::from_publication(publication.clone(), foreign_pre, post)
            .unwrap_err(),
        CompactionReadInterlockDenial::PreCutoverReadReceiptMismatch
    );
    assert_eq!(
        CompactionReadPlanCompletion::from_publication(publication, pre, pre).unwrap_err(),
        CompactionReadInterlockDenial::PostCutoverReadReceiptMismatch
    );
}

#[test]
fn matching_new_root_cannot_hide_an_unrelated_completed_footprint() {
    let plan = admitted_compaction_plan();
    let pre = plan.source_integrity().stable_read_receipt().unwrap();
    let publication = published_compaction(plan);
    let authority =
        worth_store_physical_isolation::admit_post_publication_read_stability_authority(
            publication.publication(),
        )
        .unwrap();
    let wrong = admit_plan(
        &authority,
        publication.publication().new_root(),
        protected_set([current_generation_page_reference(999)], 1),
        8,
        1,
    )
    .into_execution_ready_handle()
    .complete_plan();
    assert_eq!(
        wrong.read_plan_release().root(),
        publication.publication().new_root()
    );
    assert_ne!(
        wrong.read_plan_release().footprint_basis(),
        publication.delta().plan().candidates().footprint_basis()
    );
    assert_eq!(
        CompactionReadPlanCompletion::from_publication(publication.clone(), pre, wrong)
            .unwrap_err(),
        CompactionReadInterlockDenial::PostCutoverReadReceiptMismatch
    );

    let post = publication
        .plan_post_cutover_read()
        .unwrap()
        .into_execution_ready_handle()
        .complete_plan();
    assert!(CompactionReadPlanCompletion::from_publication(publication, pre, post).is_ok());
}

#[test]
fn matching_old_root_cannot_hide_an_unrelated_completed_footprint() {
    let plan = admitted_compaction_plan();
    let root = plan.protected().root();
    let identity = root.store_authority_identity();
    let authority =
        worth_store_physical_isolation::physical_read_stability_authority_for_certification_test(
            17, identity,
        );
    let wrong: PhysicalReadPlanCompletionReceipt = admit_plan(
        &authority,
        root,
        protected_set([current_generation_page_reference(999)], 1),
        8,
        1,
    )
    .into_execution_ready_handle()
    .complete_plan();
    let pre = plan.source_integrity().stable_read_receipt().unwrap();
    let publication = published_compaction(plan);
    let post = publication
        .plan_post_cutover_read()
        .unwrap()
        .into_execution_ready_handle()
        .complete_plan();
    assert_eq!(
        wrong.read_plan_release().root(),
        pre.read_plan_release().root()
    );
    assert_ne!(
        wrong.read_plan_release().footprint_basis(),
        pre.read_plan_release().footprint_basis()
    );
    assert_eq!(
        CompactionReadPlanCompletion::from_publication(publication.clone(), wrong, post)
            .unwrap_err(),
        CompactionReadInterlockDenial::PreCutoverReadReceiptMismatch
    );
    assert!(CompactionReadPlanCompletion::from_publication(publication, pre, post).is_ok());
}

#[test]
fn combined_observation_rejects_reclaim_from_another_plan_or_publication() {
    use worth_store_physical_isolation::{
        CompactionDeferredReclaimQueue, CompactionInterlockFoundationalEvidence,
    };
    use worth_store_test_support::harness::physical_isolation::compaction::published_compaction_at_manifest;
    let plan = admitted_compaction_plan();
    let pre = plan.source_integrity().stable_read_receipt().unwrap();
    let publication = published_compaction(plan.clone());
    let post = publication
        .plan_post_cutover_read()
        .unwrap()
        .into_execution_ready_handle()
        .complete_plan();
    let completion =
        CompactionReadPlanCompletion::from_publication(publication.clone(), pre, post).unwrap();
    let foreign_plan = admitted_compaction_plan();
    assert_eq!(foreign_plan.protected(), plan.protected());
    assert_ne!(foreign_plan, plan);
    let different_manifest = published_compaction_at_manifest(
        plan,
        publication.publication().new_root().manifest_epoch().get() + 1,
    );
    for foreign in [published_compaction(foreign_plan), different_manifest] {
        let drain = CompactionDeferredReclaimQueue::admit(foreign)
            .unwrap()
            .drain_after_release(pre.read_plan_release())
            .unwrap();
        assert_eq!(
            CompactionInterlockFoundationalEvidence::after_completed_plans_and_reclaim(
                &completion,
                &drain
            ),
            Err(CompactionReadInterlockDenial::ReclaimPublicationMismatch)
        );
    }
    let drain = CompactionDeferredReclaimQueue::admit(publication)
        .unwrap()
        .drain_after_release(pre.read_plan_release())
        .unwrap();
    let evidence = CompactionInterlockFoundationalEvidence::after_completed_plans_and_reclaim(
        &completion,
        &drain,
    )
    .unwrap();
    assert!(evidence.blocked_reclaim_until_release());
}
