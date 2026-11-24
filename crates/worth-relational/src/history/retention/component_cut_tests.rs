//! Deterministic distinct-lane membership changes cross one real owner cut.
use crate::tests::support::*;
use std::sync::{mpsc::sync_channel, Arc};

#[test]
fn native_custody_correction_cross_lane_acquire_release_has_one_actual_capture_cut() {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "lane-a-original");
    let (_, basis_a) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let lease_a = runtime.retain_component_basis(&basis_a).unwrap();
    drop(basis_a);
    let other = create_branch_from_main(&runtime, "cut-lane-b");
    let (_, basis_b) = runtime
        .observe_branch(&runtime.branch_identity(&other).unwrap())
        .unwrap();
    create_entity(execution, &runtime, "lane-a-advanced");
    let (entered_tx, entered_rx) = sync_channel(0);
    let (resume_tx, resume_rx) = sync_channel(0);
    let capture_cut =
        super::arm_component_capture_for_test(&runtime.history, entered_tx, resume_rx);
    let selected = std::thread::scope(|scope| {
        let capture = scope.spawn(|| runtime.history.external_checkpoint_components());
        entered_rx.recv().unwrap();
        // This receiver seam is after retired lane collection and before the
        // current inventory and all lane observations, inside the same cut.
        // It observes the real owner lock, not elapsed time or a synthetic set.
        assert!(
            capture_cut.external_cut_held_for_test(),
            "the actual capture owns the complete cut while paused between lanes"
        );
        let (started_tx, started_rx) = sync_channel(0);
        let changing_runtime = &runtime;
        let changing_basis = &basis_b;
        let changed = scope.spawn(move || {
            started_tx.send(()).unwrap();
            changing_runtime.release_component_basis(lease_a).unwrap();
            changing_runtime
                .retain_component_basis(changing_basis)
                .unwrap()
        });
        started_rx.recv().unwrap();
        resume_tx.send(()).unwrap();
        let selected = capture.join().unwrap();
        let lease_b = changed.join().unwrap();
        drop(capture_cut);
        assert!(selected.iter().any(
            |basis| basis.identity().branch_id() == runtime.main_branch_identity().branch_id()
        ));
        assert!(
            !selected
                .iter()
                .any(|basis| basis.identity().branch_id() == &other),
            "A and B were never simultaneous external obligations"
        );
        let after = runtime.history.external_checkpoint_components();
        assert!(!after.iter().any(
            |basis| basis.identity().branch_id() == runtime.main_branch_identity().branch_id()
        ));
        assert!(after
            .iter()
            .any(|basis| basis.identity().branch_id() == &other));
        drop(after);
        runtime.release_component_basis(lease_b).unwrap();
        selected
    });
    drop(selected);
    drop(basis_b);
    runtime.prepare_close().unwrap().finish();
}

#[test]
fn native_custody_correction_current_lane_inventory_is_collected_inside_actual_cut() {
    let runtime = runtime_with_test_schema();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let lease = runtime.retain_component_basis(&basis).unwrap();
    let retention = runtime.history.retention_owner_for_test();
    let selected = retention.checkpoint_components(|| {
        assert!(
            retention.external_cut_held_for_test(),
            "current lane discovery must share the external retention cut"
        );
        let cell = runtime
            .history
            .branch_cell(runtime.main_branch_identity().branch_id())
            .unwrap();
        vec![(
            cell.identity().branch_id().clone(),
            cell.head_retention().binding().unwrap(),
        )]
    });
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].descriptor(), basis.descriptor());
    drop(selected);
    runtime.release_component_basis(lease).unwrap();
    drop(basis);
    runtime.prepare_close().unwrap().finish();
}

#[test]
fn owner_reclamation_overflow_preserves_prior_removals_and_requeues_overflowing_root() {
    use crate::storage::substrate::StorageAllocationOverflow;
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let mut config = runtime_with_test_schema().config();
    config.storage.retention.reclaim_batch_size = 0;
    let runtime = crate::runtime::RelationalRuntime::new(config).unwrap();

    // Two commits retire the initial root and commit 1's root while automatic reclamation is 0.
    create_entity(execution, &runtime, "reclaim-overflow-root-1");
    create_entity(execution, &runtime, "reclaim-overflow-root-2");
    assert_eq!(runtime.retired_branch_root_count(), 2);

    let owner = runtime.history.retention_owner_for_test();
    let queued_bytes = owner.retired_root_unique_bytes_for_test();
    assert_eq!(queued_bytes.len(), 2);
    let min_bytes = *queued_bytes.iter().min().unwrap();
    let max_bytes = *queued_bytes.iter().max().unwrap();
    assert!(min_bytes > 0);
    // Prime maintenance counters so the first retired root fits (regardless of
    // queue pop order) and the second retired root overflows by >= 1 byte.
    let prime_bytes = u64::MAX - max_bytes;
    owner.set_reclaimed_unique_bytes_for_test(prime_bytes);

    // Strict pass: reclaims root 1, commits root 1 into maintenance counters,
    // refuses root 2 with StorageAllocationOverflow, and restores root 2 to the queue.
    assert_eq!(
        runtime.history.try_reclaim_retired_branch_roots(4),
        Err(StorageAllocationOverflow)
    );
    assert_eq!(
        runtime.retired_branch_root_count(),
        1,
        "root 1 was reclaimed while overflowing root 2 remained retained"
    );
    let after_first_pass = runtime.retention_cost_counters();
    assert_eq!(after_first_pass.reclamation_roots_examined, 1);
    assert_eq!(after_first_pass.reclamation_roots_reclaimed, 1);
    assert!(
        after_first_pass.reclamation_unique_authoritative_bytes > prime_bytes,
        "root 1's reclaimed bytes were committed before root 2 overflowed"
    );

    // Legacy pass on the still-overflowing root 2 preserves accurate retained count,
    // reports overflow through the checked accessor, and keeps root 2 queued.
    let legacy = runtime.history.reclaim_retired_branch_roots(4);
    assert_eq!(legacy.roots_examined(), 0);
    assert_eq!(legacy.roots_reclaimed(), 0);
    assert_eq!(legacy.roots_still_retained(), 1);
    assert_eq!(
        legacy.checked_unique_authoritative_bytes_reclaimed(),
        Err(StorageAllocationOverflow)
    );
    assert_eq!(runtime.retired_branch_root_count(), 1);

    // Clearing the ceiling proves the second root can still be reclaimed.
    owner.set_reclaimed_unique_bytes_for_test(0);
    let final_pass = runtime.history.try_reclaim_retired_branch_roots(4).unwrap();
    assert_eq!(final_pass.roots_examined(), 1);
    assert_eq!(final_pass.roots_reclaimed(), 1);
    assert_eq!(final_pass.roots_still_retained(), 0);
    assert!(final_pass.unique_authoritative_bytes_reclaimed() >= min_bytes);
    assert_eq!(runtime.retired_branch_root_count(), 0);
}

#[test]
fn concurrent_reclamation_passes_account_shared_backing_exactly_across_drop_boundary() {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let mut config = runtime_with_test_schema().config();
    config.storage.retention.reclaim_batch_size = 0;
    let runtime = crate::runtime::RelationalRuntime::new(config).unwrap();

    // Commit 1 establishes root_a; fork sibling shares root_a's region.
    create_entity(execution, &runtime, "shared-reclaim-p1-v1");
    let sibling = create_branch_from_main(&runtime, "shared-reclaim-sibling");
    // Advance sibling on a second partition so sibling's root_b shares main's
    // partition-1 region with root_a while having its own distinct root object.
    create_entity_in_partition_on_branch(
        execution,
        &runtime,
        "shared-reclaim-p2-v1",
        crate::identity::data::PartitionId(2),
        sibling.clone(),
    );
    // Advance both main and sibling on partition 1 so ONLY the retired roots
    // still hold partition 1 v1 (strong_count == 2 across the retired pair).
    create_entity(execution, &runtime, "shared-reclaim-p1-v2");
    create_entity_in_partition_on_branch(
        execution,
        &runtime,
        "shared-reclaim-p1-v3",
        crate::identity::data::PartitionId::main(),
        sibling,
    );

    let owner = runtime.history.retention_owner_for_test();
    // Reclaim any initial empty root(s) in one bounded pass so only the two shared-region roots remain.
    let extra = runtime.retired_branch_root_count().saturating_sub(2);
    if extra > 0 {
        let drained = runtime
            .history
            .try_reclaim_retired_branch_roots(extra)
            .unwrap();
        assert_eq!(drained.roots_reclaimed(), extra as u64);
    }
    assert_eq!(runtime.retired_branch_root_count(), 2);

    // Independent live-inspection allocation-union oracle before either drop:
    // `reclaimable_only_authoritative_bytes` is A(reclaimable) \ A(current U held).
    let oracle = runtime.observe_retained_storage().unwrap();
    assert_eq!(oracle.reclaimable_roots, 2);
    let expected_union_bytes = oracle.reclaimable_only_authoritative_bytes;
    assert!(expected_union_bytes > 0);
    let shared_region_root_bytes: u64 =
        owner.retired_root_unique_bytes_for_test().into_iter().sum();
    assert!(
        expected_union_bytes > shared_region_root_bytes,
        "shared partition-1 region is excluded from each root alone but present in the pair union"
    );
    let baseline_bytes = runtime
        .retention_cost_counters()
        .reclamation_unique_authoritative_bytes;

    // Pause Pass A right after removing root 1 and BEFORE `drop(root 1)`.
    let (entered_tx, entered_rx) = sync_channel(0);
    let (resume_tx, resume_rx) = sync_channel(0);
    let (contended_tx, contended_rx) = sync_channel(0);
    let pass_b_done = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let reclamation = owner.pause_reclamation_drop_for_test(entered_tx, resume_rx);
    let runtime_ref = &runtime;
    let (out_a, out_b) = std::thread::scope(|scope| {
        // Owned inside `scope`: dropping `resume_tx` (normal or panic unwind) is
        // nonblocking and wakes `CapturePause::wait` BEFORE `thread::scope` joins.
        let resume_tx = resume_tx;
        let pass_a = scope.spawn(move || {
            runtime_ref
                .history
                .try_reclaim_retired_branch_roots(1)
                .unwrap()
        });
        entered_rx
            .recv()
            .expect("Pass A must reach reclamation_drop_pause before exiting");
        assert!(
            reclamation.reclamation_gate_held_for_test(),
            "Pass A holds gate across drop(root)"
        );
        reclamation.observe_contention_for_test(contended_tx);
        let done_flag = Arc::clone(&pass_b_done);
        let pass_b = scope.spawn(move || {
            let out = runtime_ref
                .history
                .try_reclaim_retired_branch_roots(1)
                .unwrap();
            done_flag.store(true, std::sync::atomic::Ordering::Release);
            out
        });
        // Deterministic receiver-local contention observation: Pass B actually hit WouldBlock.
        contended_rx
            .recv()
            .expect("Pass B must contend on reclamation_gate while Pass A holds it");
        assert!(
            !pass_b_done.load(std::sync::atomic::Ordering::Acquire),
            "Pass B remains blocked"
        );
        drop(resume_tx);
        (pass_a.join().unwrap(), pass_b.join().unwrap())
    });

    assert_eq!(
        (
            out_a.roots_reclaimed(),
            out_b.roots_reclaimed(),
            runtime.retired_branch_root_count()
        ),
        (1, 1, 0)
    );
    let combined =
        out_a.unique_authoritative_bytes_reclaimed() + out_b.unique_authoritative_bytes_reclaimed();
    assert_eq!(
        combined, expected_union_bytes,
        "concurrent passes must match live-inspection union"
    );
    let delta = runtime
        .retention_cost_counters()
        .reclamation_unique_authoritative_bytes
        - baseline_bytes;
    assert_eq!(delta, expected_union_bytes);
}
