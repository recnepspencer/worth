use super::*;
use worth_execution::{CancellationSource, ExecutionRequest};

struct CancelAfterRead {
    source: Arc<CountedSource>,
    cancellation: Arc<CancellationSource>,
    first: AtomicBool,
}
impl std::fmt::Debug for CancelAfterRead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CancelAfterRead")
    }
}
impl RelationalDurableColdSource for CancelAfterRead {
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        let bytes = self.source.read_member(locator, limit)?;
        if self.first.swap(false, Ordering::SeqCst) {
            self.cancellation.cancel();
        }
        Ok(bytes)
    }
    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        self.source.read_commit(position, limit)
    }
}

#[test]
fn cancellation_during_readmission_preserves_the_request_and_does_not_damage_the_cell() {
    let fixture = region_with_history();
    let cancellation = Arc::new(CancellationSource::new());
    let request =
        crate::test_execution_policy::serial_request().with_cancellation(cancellation.token());
    let mut cold = fixture.cold;
    cold.source = Arc::new(CancelAfterRead {
        source: fixture.source.clone(),
        cancellation,
        first: AtomicBool::new(true),
    });
    let cell = RelationalRegionResidency::cold(cold);
    let failure = cell
        .demand_head(&ExecutionRequest::serial(&request), 1 << 24)
        .unwrap_err();
    assert_eq!(
        failure,
        RelationalResidencyDemandFailure::Request(
            crate::execution::RelationalExecutionDenialCause::Cancelled
        )
    );
    assert_eq!(cell.leases.load(Ordering::SeqCst), 0);
    let read_before_retry = fixture.source.reads.load(Ordering::SeqCst);
    assert!(
        read_before_retry > 0,
        "the request must stop after entering readmission"
    );
    let accepted = head_demand(&cell, 1 << 24).unwrap();
    assert!(accepted.head_view().entity(fixture.entity).is_some());
    assert!(
        fixture.source.reads.load(Ordering::SeqCst) > read_before_retry,
        "a request stop must not remain cached as a storage refusal"
    );
    assert!(fixture.source.reads.load(Ordering::SeqCst) < 4);
}
