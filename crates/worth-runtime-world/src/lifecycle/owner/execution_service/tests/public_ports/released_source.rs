//! Original release preparation is fallible only while actual custody remains.
use super::*;
use crate::facade::RuntimeWorldRecoveryDenial as Denial;

#[test]
fn terminal_preparation_refusals_preserve_original_custody_without_publishing_slots() {
    let (fixture, owner, expected) = public_world();
    let host_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(
            &owner.root.state.execution.request_policy(),
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&host_request);
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = owner
        .publication_port()
        .prepare_with_signal(
            expected,
            CompositePublicationIntent::with_signal(Some(RelationalTransactionIntent::ordinary()))
                .with_prepared_relational_candidate(
                    fixture.prepare_relational_owner_candidate(&execution, "stopped-release"),
                ),
            &cancellation.token(),
            None,
        )
        .unwrap();
    let effects = match owner.publication_port().execute_with_signal(
        execution,
        prepared,
        &mut (),
        &cancellation.token(),
        |_| {
            cancellation.cancel();
            Ok(())
        },
    ) {
        RuntimeWorldPublicationOutcome::ProductUnpublished(effects) => effects,
        other => panic!("actual Native original must precede cancelled Signal: {other:?}"),
    };
    let receipt = effects
        .component_results()
        .relational_settlement()
        .unwrap()
        .clone();
    let handle = effects.recovery_handle();
    drop(effects);
    assert!(matches!(
        owner
            .recovery_port()
            .release_effects(&execution, &handle, 0, None),
        Err(Denial::OriginalSourceUnavailable)
    ));
    let foreign = crate::branch::reference_test_fixture::real_fixture(12, 12);
    let performed = foreign.perform_relational_owner_change(&execution);
    let settled = foreign
        .relational_runtime()
        .settlement_port()
        .settle_performed_publication(performed)
        .unwrap();
    let foreign_original = foreign
        .relational_runtime()
        .durability_authority()
        .observe_memory_commit(&execution, &settled.outcome().commit)
        .unwrap();
    assert!(matches!(
        owner
            .recovery_port()
            .release_effects(&execution, &handle, 0, Some(foreign_original)),
        Err(Denial::OriginalSourceMismatch)
    ));
    let record = owner.root.state.recovery.inspect_record(&handle).unwrap();
    let stop = worth_execution::CancellationSource::new();
    stop.cancel();
    let stopped = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(
            &owner.root.state.execution.request_policy(),
        ),
        stop.token(),
        None,
    );
    assert!(matches!(
        owner.root.state.recovery.prepare_memory_original_release(
            &worth_execution::ExecutionRequest::serial(&stopped),
            &record
        ),
        Err(crate::durability::WorldDurableMediumDenial::Request(_))
    ));
    let prepared = owner
        .root
        .state
        .recovery
        .prepare_memory_original_release(&execution, &record)
        .unwrap();
    assert!(owner
        .recovery_port()
        .released_original_page(&execution, None, std::num::NonZeroUsize::MIN)
        .unwrap()
        .rows()
        .is_empty());
    drop(prepared);
    assert!(owner
        .recovery_port()
        .released_original_page(&execution, None, std::num::NonZeroUsize::MIN)
        .unwrap()
        .rows()
        .is_empty());
    drop(record);
    let native = fixture
        .relational_runtime()
        .durability_authority()
        .observe_memory_commit(&execution, &receipt)
        .unwrap();
    let released = owner
        .recovery_port()
        .release_effects(&execution, &handle, 0, Some(native))
        .unwrap();
    assert_eq!(released.original().native_receipt(), Some(&receipt));
    assert_eq!(
        owner
            .recovery_port()
            .released_original_page(&execution, None, std::num::NonZeroUsize::MIN)
            .unwrap()
            .rows(),
        &[released.original().clone()]
    );
}
