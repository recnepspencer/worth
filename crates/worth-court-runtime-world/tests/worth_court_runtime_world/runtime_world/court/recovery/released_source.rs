//! Original release refusals preserve custody at the public World receiver.
use super::*;
use crate::court::ordinary_context::CompositeSupplyChainCourt;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

#[test]
fn terminal_preparation_refusals_preserve_original_custody_without_publishing_slots() {
    let policy = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&policy);
    let court = CompositeSupplyChainCourt::compile();
    let foreign = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let foreign_root = foreign.bootstrap();
    let effects = court.partial_after_relational(&execution, &root);
    let receipt = effects
        .component_results()
        .relational_settlement()
        .unwrap()
        .clone();
    let handle = effects.recovery_handle();
    drop(effects);
    let recovery = court.world.recovery_port();
    assert!(matches!(
        recovery.release_effects(&execution, &handle, 0, None),
        Err(RuntimeWorldRecoveryDenial::OriginalSourceUnavailable)
    ));
    let foreign_effects = foreign.partial_after_relational(&execution, &foreign_root);
    let foreign_handle = foreign_effects.recovery_handle();
    drop(foreign_effects);
    assert!(matches!(
        recovery.release_effects(
            &execution,
            &handle,
            0,
            foreign.observe_recovery_original(&execution, &foreign_handle)
        ),
        Err(RuntimeWorldRecoveryDenial::OriginalSourceMismatch)
    ));
    let stop = CancellationSource::new();
    let stopped =
        SerialRequest::from_memory(SerialMemoryBudget::new(Some(1 << 20)), stop.token(), None);
    let native = court.observe_recovery_original(&execution, &handle);
    let reached = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let observed = reached.clone();
    // The existing SDK observer cancels only inside the actual terminal
    // preparation, after the live Native original-source check has succeeded.
    let (refused, _) = worth_execution::with_request_consult_observer_for_test(
        move |_| {
            let trace = std::backtrace::Backtrace::force_capture().to_string();
            if trace.contains("prepare_memory_original_release") {
                observed.store(true, std::sync::atomic::Ordering::SeqCst);
                stop.cancel();
            }
        },
        || recovery.release_effects(&ExecutionRequest::serial(&stopped), &handle, 0, native),
    );
    assert!(
        reached.load(std::sync::atomic::Ordering::SeqCst),
        "the actual terminal preparation must be observed"
    );
    assert!(matches!(
        refused,
        Err(RuntimeWorldRecoveryDenial::OutputCapacityExhausted)
    ));
    // Age admission happens after successful terminal preparation. Refusal
    // drops that unfinished destination, which must publish no released slot.
    assert!(matches!(
        recovery.release_effects(
            &execution,
            &handle,
            1,
            court.observe_recovery_original(&execution, &handle)
        ),
        Err(RuntimeWorldRecoveryDenial::TooYoung)
    ));
    assert_eq!(
        recovery
            .inspect_effects(&handle)
            .unwrap()
            .component_results()
            .relational_settlement(),
        Some(&receipt)
    );
    assert!(recovery
        .released_original_page(&execution, None, std::num::NonZeroUsize::MIN)
        .unwrap()
        .rows()
        .is_empty());
    let released = recovery
        .release_effects(
            &execution,
            &handle,
            0,
            court.observe_recovery_original(&execution, &handle),
        )
        .unwrap();
    assert_eq!(released.original().native_receipt(), Some(&receipt));
    assert_eq!(
        recovery
            .released_original_page(&execution, None, std::num::NonZeroUsize::MIN)
            .unwrap()
            .rows(),
        &[released.original().clone()]
    );
    assert_eq!(
        court.observe(&root).selected_commit(),
        root.selected_commit()
    );
    foreign
        .world
        .recovery_port()
        .release_effects(
            &execution,
            &foreign_handle,
            0,
            foreign.observe_recovery_original(&execution, &foreign_handle),
        )
        .unwrap();
    drop((root, foreign_root, released));
    court.finish();
    foreign.finish();
}
