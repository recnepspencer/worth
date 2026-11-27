//! Round 12 hardening court for Store M4.3f (`[P2-1]` and `[P2-2]`):
//! - Sibling merge (`sibling2` forked at C2 into `sibling1` forked at C1) applies
//!   each ancestor branch's own cutoff (`main <= C2`), skipping `main` event pages
//!   wholly after C2 even though `sibling2` itself has bound `u64::MAX`.

use std::collections::BTreeSet;

use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalDurableCheckpointSource, RelationalDurableRecoverySource,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::merge::{MergeExecutionRequest, MergeIntent};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::transactions::{
    CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch,
};
use worth_testkit_relational::aspect_storage::publications::release_test_commit_snapshot;
use worth_testkit_relational::cold_recovery::{
    capture_external_carrying, capture_external_checkpoint,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::field_patch;

use super::bounded_naming_fixture::observe_branch;
use super::bounded_naming_r2::fork_retained_at;
use super::bounded_naming_r3::durable_mixed_runtime;

fn commit_unnamed(
    request: &ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    branch: &BranchId,
    label: &str,
) {
    let basis = observe_branch(runtime, branch);
    let mut tx = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new(label).push(MutationIntent::Create(CreateIntent::Entity(
            EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(3),
                client_key: label.into(),
                fields: field_patch("name", "name", label),
            },
        ))),
        request.allocation_policy(),
    )
    .unwrap();
    let committed = tx.commit(request, runtime).unwrap();
    release_test_commit_snapshot(runtime, &committed);
}

#[test]
fn sibling_merge_skips_ancestor_event_pages_past_source_fork_cutoff() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_mixed_runtime(&medium);
    let main = producer.main_branch_identity().branch_id().clone();

    // C1 on main -> fork sibling1.
    commit_unnamed(request, &producer, &main, "r12-main-c1");
    let sibling1 = BranchId("r12-sibling-1".into());
    let obs_c1 = observe_branch(&producer, &main);
    fork_retained_at(request, &producer, &obs_c1, &sibling1).unwrap();

    // C2 on main -> fork sibling2.
    commit_unnamed(request, &producer, &main, "r12-main-c2");
    let sibling2 = BranchId("r12-sibling-2".into());
    let obs_c2 = observe_branch(&producer, &main);
    fork_retained_at(request, &producer, &obs_c2, &sibling2).unwrap();

    // Seal C1..C2 in checkpoint 1 and reopen so main's later C3 event lands in a
    // distinct post-C2 event page carried separately in checkpoint 2.
    let (ck1, src1) = capture_external_checkpoint(request, &producer);
    let ck1_event_members: BTreeSet<_> = super::selected_directory::read_native(&ck1)
        .lineage_events
        .iter()
        .map(|p| p.member)
        .collect();
    let mut mid = durable_mixed_runtime(&medium);
    let mid_in = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            ck1.covered_position().unwrap(),
            ck1.covered_branch_transitions().unwrap(),
            ck1,
        )),
        Vec::new(),
        Vec::new(),
        src1.clone(),
    );
    let mid_plan = mid
        .durability()
        .recovery_plan_from_source(
            request,
            mid_in,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    mid.durability_recovery()
        .recover(request, mid_plan)
        .unwrap();

    // C3 on main (strictly past sibling2's fork cutoff C2).
    commit_unnamed(request, &mid, &main, "r12-main-c3-beyond-sibling2");

    let (ck2, src2) = capture_external_carrying(request, &mid, &src1);
    let ck2_dir = super::selected_directory::read_native(&ck2);
    let post_c2_main_members: BTreeSet<_> = ck2_dir
        .lineage_events
        .iter()
        .map(|p| p.member)
        .filter(|m| !ck1_event_members.contains(m))
        .collect();
    assert!(
        !post_c2_main_members.is_empty(),
        "C2 must emit a distinct main event page wholly after sibling2's fork cutoff"
    );

    let mut reopened = durable_mixed_runtime(&medium);
    let reopen_in = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            ck2.covered_position().unwrap(),
            ck2.covered_branch_transitions().unwrap(),
            ck2,
        )),
        Vec::new(),
        Vec::new(),
        src2.clone(),
    );
    let reopen_plan = reopened
        .durability()
        .recovery_plan_from_source(
            request,
            reopen_in,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    reopened
        .durability_recovery()
        .recover(request, reopen_plan)
        .unwrap();

    // Fail any read of the post-C2 main event page: merge preparation of sibling2
    // into sibling1 must neither read nor fail on it.
    let failed_target = *post_c2_main_members.iter().next().unwrap();
    *src2.failed_member.lock().unwrap() = Some(failed_target);
    let reads_before = src2.member_reads.lock().unwrap().len();

    let bound = reopened
        .bind_merge_execution_request(MergeExecutionRequest::new(
            sibling1,
            sibling2,
            MergeIntent::ReconcileIntoTarget,
        ))
        .unwrap();
    reopened
        .prepare_merge_execution(request, bound)
        .expect("sibling merge must skip main event pages past source's fork cutoff");

    let post_c2_reads = src2.member_reads.lock().unwrap()[reads_before..]
        .iter()
        .filter(|m| post_c2_main_members.contains(m))
        .count();
    assert_eq!(
        post_c2_reads, 0,
        "merge preparation must not read main event pages wholly after sibling2's fork"
    );
}
