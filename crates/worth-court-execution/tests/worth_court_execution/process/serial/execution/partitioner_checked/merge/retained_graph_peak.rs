use super::*;

#[test]
fn report_peak_includes_two_simultaneously_retained_graphs() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

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
