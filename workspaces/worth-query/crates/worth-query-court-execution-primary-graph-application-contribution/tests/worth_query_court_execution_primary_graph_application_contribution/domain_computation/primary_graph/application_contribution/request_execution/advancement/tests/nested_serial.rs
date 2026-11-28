use super::*;

#[test]
fn nested_distinct_child_serial_advancement_respects_enclosing_parent_room_and_deduplicates() {
    let parent_budget = worth_execution::SerialMemoryBudget::new(Some(16 * 1024));
    let parent_req = worth_execution::SerialRequest::from_memory(
        parent_budget.clone(),
        worth_execution::CancellationToken::new(),
        None,
    );
    let parent_hold = parent_budget
        .reserve(10 * 1024)
        .expect("parent holds 10 KiB");
    let installed_policy = serial_policy(Some(32 * 1024));

    // Also verify an advancement opened BEFORE entering the parent scope still
    // enforces the enclosing parent serial scope when executed inside it.
    let preopened_budget = worth_execution::SerialMemoryBudget::new(Some(24 * 1024));
    let preopened_req = worth_execution::SerialRequest::from_memory(
        preopened_budget.clone(),
        worth_execution::CancellationToken::new(),
        None,
    );
    let preopened_exec = open_serial(
        installed_policy,
        worth_execution::ExecutionRequest::serial(&preopened_req),
    );

    worth_execution::ExecutionWorkCeiling::new(None)
        .run_serial(&parent_req, || {
            // 1. Distinct child SerialRequest opened inside active finite parent scope.
            let child_budget = worth_execution::SerialMemoryBudget::new(Some(24 * 1024));
            let child_req = worth_execution::SerialRequest::from_memory(
                child_budget.clone(),
                worth_execution::CancellationToken::new(),
                None,
            );
            let exec = open_serial(
                installed_policy,
                worth_execution::ExecutionRequest::serial(&child_req),
            );
            exec.run_advancement(None, None, |phase| {
                // Only 6 KiB remain on the parent (16 - 10), minus small framework overhead.
                assert!(reserve_phase_memory(&phase, 8 * 1024).is_err());
                assert!(matches!(
                    exec.reserve(8 * 1024),
                    Err(Resource::MemoryLimit { .. })
                ));
                let raw = reserve_phase_memory(&phase, 2 * 1024)
                    .expect("2 KiB raw phase hold fits in parent remaining room")
                    .expect("leased phase returns reservation");
                assert!(parent_budget.charged_memory_bytes() >= 12 * 1024);
                assert!(child_budget.charged_memory_bytes() >= 2 * 1024);
                drop(raw);
            })
            .expect("distinct nested child advancement completes");
            assert_eq!(child_budget.charged_memory_bytes(), 0);

            // 2. Pre-opened child advancement run inside parent scope still obeys parent room
            //    for both phase-borrowed and direct `preopened_exec.reserve(...)` calls.
            assert!(matches!(
                preopened_exec.reserve(8 * 1024),
                Err(Resource::MemoryLimit { .. })
            ));
            preopened_exec
                .run_advancement(None, None, |phase| {
                    assert!(reserve_phase_memory(&phase, 8 * 1024).is_err());
                    assert!(matches!(
                        preopened_exec.reserve(8 * 1024),
                        Err(Resource::MemoryLimit { .. })
                    ));
                    let direct = preopened_exec
                        .reserve(2 * 1024)
                        .expect("2 KiB direct reserve fits in parent remaining room");
                    assert!(parent_budget.charged_memory_bytes() >= 12 * 1024);
                    assert!(preopened_budget.charged_memory_bytes() >= 2 * 1024);
                    drop(direct);
                    let raw = reserve_phase_memory(&phase, 2 * 1024)
                        .expect("2 KiB fits in parent remaining room")
                        .expect("leased phase returns reservation");
                    drop(raw);
                })
                .expect("preopened nested child advancement completes");
            assert_eq!(preopened_budget.charged_memory_bytes(), 0);

            // 3. Shared budget with parent must be counted ONCE, not twice.
            let shared_req = worth_execution::SerialRequest::from_memory(
                parent_budget.clone(),
                worth_execution::CancellationToken::new(),
                None,
            );
            let shared_exec = open_serial(
                installed_policy,
                worth_execution::ExecutionRequest::serial(&shared_req),
            );
            shared_exec
                .run_advancement(None, None, |phase| {
                    let before = parent_budget.charged_memory_bytes();
                    let raw = reserve_phase_memory(&phase, 3 * 1024)
                        .expect("3 KiB fits once in remaining parent room")
                        .expect("leased phase returns reservation");
                    // Charged exactly once (+3 KiB), NOT twice (+6 KiB).
                    assert_eq!(parent_budget.charged_memory_bytes() - before, 3 * 1024);
                    drop(raw);
                })
                .expect("shared-budget nested advancement completes");

            // 4. Worker-only child SerialRequest (memory = None) inside finite parent
            //    serial scope must still inherit and enforce parent's remaining room.
            let worker_only_policy = worth_foundational::ExecutionRequestPolicy::for_workers(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                std::num::NonZeroUsize::MIN,
            );
            let unbounded_req = worth_execution::SerialRequest::from_policy(
                &worker_only_policy,
                worth_execution::CancellationToken::new(),
                None,
            );
            assert!(unbounded_req.memory().is_none());
            let unbounded_exec = open_serial(
                worker_only_policy,
                worth_execution::ExecutionRequest::serial(&unbounded_req),
            );
            unbounded_exec
                .run_advancement(None, None, |phase| {
                    assert!(reserve_phase_memory(&phase, 8 * 1024).is_err());
                    assert!(matches!(
                        unbounded_exec.reserve(8 * 1024),
                        Err(Resource::MemoryLimit { .. })
                    ));
                    let fit = reserve_phase_memory(&phase, 2 * 1024)
                        .expect("2 KiB fits in parent remaining room")
                        .expect("leased phase returns reservation");
                    assert!(parent_budget.charged_memory_bytes() >= 12 * 1024);
                    drop(fit);
                })
                .expect("worker-only nested advancement completes");
        })
        .expect("parent serial scope completes");

    assert_eq!(parent_budget.charged_memory_bytes(), 10 * 1024);
    drop(parent_hold);
    assert_eq!(parent_budget.charged_memory_bytes(), 0);
}

#[test]
fn serial_caller_is_capped_to_one_worker_on_multiworker_leased_placement() {
    let four = NonZeroUsize::new(4).unwrap();
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Automatic,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(four, Some(1024 * 1024), Some(10_000)),
    );
    let authority = worth_execution::ExecutionAuthority::try_construct(
        worth_execution::ExecutionAuthorityConfig {
            max_workers: four,
            charged_memory_bytes: Some(4 * 1024 * 1024),
        },
    )
    .unwrap();
    let serial = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        worth_execution::CancellationToken::new(),
        None,
    );
    let caller = worth_execution::ExecutionRequest::serial(&serial);
    let exec = QueryRequestExecution::open_serial_request(
        worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Leased {
            authority: &authority,
            policy,
        },
        caller,
        Some(policy),
        worth_execution::CancellationToken::new(),
        None,
    );
    let req = exec.execution_request().expect("admitted serial lease");
    assert_eq!(req.max_workers(), 1);
}
