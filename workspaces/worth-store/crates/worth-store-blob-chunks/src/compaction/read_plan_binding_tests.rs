//! Local Blob/physical-plan correlation, not an executed Store rewrite.
use super::test_support::{
    authority, compacted_rewritten_publication, intent, verified_read_for_rewritten,
};
use crate::{BlobCompactionDenial, BlobCompactionEquivalence};
use worth_store_physical_isolation::{CompactionReadInterlockPlan, CompactionReadPlanCompletion};
use worth_store_test_support::harness::physical_isolation::compaction::{
    admitted_compaction_plan, published_compaction_at_manifest,
};

#[test]
fn rewrite_consumes_matching_local_plan_completion_without_recovery_evidence() {
    let owner = authority("local-cutover");
    let plan = owner.plan_compaction(intent("local-cutover")).unwrap();
    let rewritten = compacted_rewritten_publication("local-cutover");
    let read = verified_read_for_rewritten(&plan, &rewritten);
    let equivalence =
        BlobCompactionEquivalence::from_rewritten_root_and_verified_read(&plan, &rewritten, &read)
            .unwrap();
    let manifest = super::rewrite_binding::physical_rewrite_manifest_epoch_for_root(
        equivalence.new_root(),
        plan.physical().protected().root().manifest_epoch().get(),
    );
    let completion = complete(plan.physical().clone(), manifest);
    let execution = owner
        .execute_rewrite(plan, equivalence, completion)
        .unwrap();
    assert_eq!(
        execution
            .read_plan_completion()
            .post_cutover_root()
            .manifest_epoch()
            .get(),
        manifest
    );
}

#[test]
fn rewrite_rejects_foreign_plan_identity_and_wrong_manifest_despite_local_completion() {
    for case in ["foreign-plan", "wrong-manifest", "valid"] {
        let owner = authority("local-cutover-mismatch");
        let plan = owner
            .plan_compaction(intent("local-cutover-mismatch"))
            .unwrap();
        let rewritten = compacted_rewritten_publication("local-cutover-mismatch");
        let read = verified_read_for_rewritten(&plan, &rewritten);
        let equivalence = BlobCompactionEquivalence::from_rewritten_root_and_verified_read(
            &plan, &rewritten, &read,
        )
        .unwrap();
        let manifest = super::rewrite_binding::physical_rewrite_manifest_epoch_for_root(
            equivalence.new_root(),
            plan.physical().protected().root().manifest_epoch().get(),
        );
        let completion = match case {
            "foreign-plan" => {
                let foreign = admitted_compaction_plan();
                assert_eq!(foreign.protected(), plan.physical().protected());
                assert_ne!(&foreign, plan.physical());
                complete(foreign, manifest)
            }
            "wrong-manifest" => complete(plan.physical().clone(), manifest + 1),
            _ => complete(plan.physical().clone(), manifest),
        };
        let result = owner.execute_rewrite(plan, equivalence, completion);
        if case == "valid" {
            assert!(result.is_ok(), "{result:?}");
        } else {
            assert!(
                matches!(
                    result,
                    Err(BlobCompactionDenial::MixedChunkTreePublication { .. })
                ),
                "{case}: {result:?}"
            );
        }
    }
}

fn complete(plan: CompactionReadInterlockPlan, manifest: u64) -> CompactionReadPlanCompletion {
    let pre = plan.source_integrity().stable_read_receipt().unwrap();
    let publication = published_compaction_at_manifest(plan, manifest);
    let post = publication
        .plan_post_cutover_read()
        .unwrap()
        .into_execution_ready_handle()
        .complete_plan();
    CompactionReadPlanCompletion::from_publication(publication, pre, post).unwrap()
}
