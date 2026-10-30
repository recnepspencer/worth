use crate::partitioner_checked::isolated_context::{
    authority, checked_test_guard, item, lease, map,
};
use worth_execution::MapKernelFailure;

pub(super) fn growth_stops_then_removal_admits_insertions() {
    let _serial = checked_test_guard();
    let authority = authority();
    let tight = lease(
        authority,
        4_096,
        100_000,
        worth_execution::CancellationToken::new(),
    );
    let components = std::sync::Arc::new(std::sync::Mutex::new(
        worth_execution::ComponentPartitioner::new(),
    ));
    let outcome = map().run(Some(&tight), |_, context| {
        let mut count = 0_u64;
        for id in 1..=1_000 {
            match components
                .lock()
                .unwrap()
                .upsert_item_checked(&tight, context, item(id))
            {
                Ok(_) => count += 1,
                Err(worth_execution::PartitionUpdateDenial::Admission(
                    worth_execution::LeaseDenial::MemoryExhausted(_),
                )) => break,
                Err(other) => panic!("unexpected checked denial: {other:?}"),
            }
        }
        assert!(count > 0 && count < 1_000);
        assert!(components.lock().unwrap().route(item(1_000)).is_none());
        Ok::<_, MapKernelFailure<()>>(count)
    });
    assert!(matches!(
        outcome,
        worth_execution::MapOutcome::Stopped { .. }
    ));

    let roomy = lease(
        authority,
        100_000,
        100_000,
        worth_execution::CancellationToken::new(),
    );
    let outcome = map().run(Some(&roomy), |_, context| {
        let mut components = components.lock().unwrap();
        for id in 1..=6 {
            components
                .remove_item_checked(&roomy, context, item(id))
                .unwrap();
        }
        for id in 20..=30 {
            components
                .upsert_item_checked(&roomy, context, item(id))
                .unwrap();
        }
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(
        outcome,
        worth_execution::MapOutcome::Complete { .. }
    ));
    assert!(components.lock().unwrap().route(item(30)).is_some());
}

pub(super) fn peak_includes_simultaneously_retained_graphs() {
    let _serial = checked_test_guard();
    let authority = authority();
    let admitted = lease(
        authority,
        100_000,
        100_000,
        worth_execution::CancellationToken::new(),
    );
    let first = std::sync::Arc::new(std::sync::Mutex::new(
        worth_execution::ComponentPartitioner::new(),
    ));
    let second = std::sync::Arc::new(std::sync::Mutex::new(
        worth_execution::ComponentPartitioner::new(),
    ));
    let outcome = map().run(Some(&admitted), |_, context| {
        for id in 1..=8 {
            first
                .lock()
                .unwrap()
                .upsert_item_checked(&admitted, context, item(id))
                .unwrap();
            second
                .lock()
                .unwrap()
                .upsert_item_checked(&admitted, context, item(id + 100))
                .unwrap();
        }
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    match outcome {
        worth_execution::MapOutcome::Complete { report, .. } => {
            // Each eight-item graph retains at least 8 * 384 charged bytes.
            assert!(report.physical().peak_charged_memory_bytes() >= 2 * 8 * 384);
        }
        worth_execution::MapOutcome::Stopped { reason, .. } => {
            panic!("two retained graphs stopped: {reason:?}")
        }
    }
}
