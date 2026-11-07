//! A borrowed Head remains readable after later damage is proved.
use super::*;

#[test]
fn a_cached_head_guard_refuses_after_full_demand_proves_region_damage() {
    let fixture = region_with_history();
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let mut cold = fixture.cold.clone();
    let mut members = fixture.source.members.clone();
    members.remove(&cold.history);
    let source = Arc::new(CountedSource {
        members,
        reads: AtomicUsize::new(0),
        history_reads: AtomicUsize::new(0),
        history: cold.history,
        corrupt: AtomicBool::new(false),
    });
    cold.source = Some(source.clone());
    let cell = RelationalRegionResidency::cold(cold).unwrap();
    let retained = RelationalRegionRetention::new(cell, None);
    retained.demand_head_cached(execution, 1 << 24).unwrap();
    let accepted = retained.admitted_head_view().unwrap();
    let super::super::RelationalResidencyDemandFailure::StorageUnavailable(failure) =
        retained.demand_full(execution, 1 << 24).unwrap_err()
    else {
        panic!("missing named history must be a storage refusal")
    };
    assert_eq!(failure.cause, Cause::MissingMember);
    let reads = source.reads.load(Ordering::SeqCst);
    assert!(
        accepted.entity(fixture.entity).is_some(),
        "an already borrowed immutable view remains valid"
    );
    assert!(matches!(retained.demand_head_cached(execution, 1 << 24),
        Err(super::super::RelationalResidencyDemandFailure::StorageUnavailable(retry)) if retry == failure));
    assert!(
        retained.admitted_head_view().is_none(),
        "a probe must miss rather than accept a new damaged view"
    );
    assert!(matches!(retained.cell().capture_checkpoint(execution,
        &fixture.cold.schema, fixture.cold.schema_carrier_digest,
        &fixture.cold.symbols.interner_snapshot(), None,
    ), Err(super::super::RelationalResidencyDemandFailure::StorageUnavailable(retry)) if retry == failure));
    assert_eq!(source.reads.load(Ordering::SeqCst), reads);
}
