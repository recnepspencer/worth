//! A busy provider cut is transient: an original read waits under its own request.
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    branch::RelationalBranchTransitionSequence, durability::*, publication::PatchStreamPosition,
    storage::RelationalStorageUnavailableCause as Cause,
};
use RelationalDurableOwnedReadRefusal as Refusal;

/// Refuses `Busy` for a fixed number of captures, as a contended provider does.
#[derive(Debug)]
struct ContendedCut {
    busy: AtomicUsize,
    captures: AtomicUsize,
}
impl RelationalDurableColdSource for ContendedCut {
    fn checkpoint_read_protection(
        &self,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<(RelationalDurableReadBounds, RelationalDurableReadPayload), Refusal> {
        self.captures.fetch_add(1, Ordering::SeqCst);
        let contended = self
            .busy
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |left| {
                left.checked_sub(1)
            });
        if contended.is_ok() {
            return Err(Refusal::Busy);
        }
        Ok((
            RelationalDurableReadBounds {
                selected_checkpoint: false,
                canonical: PatchStreamPosition(4),
                transitions: RelationalBranchTransitionSequence(0),
            },
            Box::new(()),
        ))
    }
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        panic!("a cut capture retains no members")
    }
    fn read_member(&self, _: RelationalDurableMemberLocator, _: u64) -> Result<Vec<u8>, Cause> {
        panic!("a cut capture reads no member")
    }
    fn read_commit(&self, _: PatchStreamPosition, _: u64) -> Result<Vec<u8>, Cause> {
        panic!("a cut capture reads no commit")
    }
}
fn contended(busy: usize) -> (Arc<ContendedCut>, Arc<dyn RelationalDurableColdSource>) {
    let cut = Arc::new(ContendedCut {
        busy: AtomicUsize::new(busy),
        captures: AtomicUsize::new(0),
    });
    (cut.clone(), cut)
}
fn serial(cancellation: &CancellationSource) -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 20)),
        cancellation.token(),
        None,
    )
}

#[test]
fn a_busy_cut_is_recaptured_until_the_provider_admits_the_reader() {
    let cancellation = CancellationSource::new();
    let host = serial(&cancellation);
    let request = ExecutionRequest::serial(&host);
    let (cut, source) = contended(3);
    assert!(matches!(
        RelationalDurableReadProtection::capture(&source, &request),
        Err(Refusal::Busy)
    ));
    let admitted =
        RelationalDurableReadProtection::capture_when_available(&source, &request).unwrap();
    assert_eq!(admitted.bounds().canonical, PatchStreamPosition(4));
    assert_eq!(cut.captures.load(Ordering::SeqCst), 4);
}

#[test]
fn a_stopped_request_ends_the_wait_for_a_busy_cut_as_its_own_stop() {
    let cancellation = CancellationSource::new();
    let host = serial(&cancellation);
    let request = ExecutionRequest::serial(&host);
    let (cut, source) = contended(usize::MAX);
    cancellation.cancel();
    assert!(matches!(
        RelationalDurableReadProtection::capture_when_available(&source, &request),
        Err(Refusal::Request(_))
    ));
    assert_eq!(
        cut.captures.load(Ordering::SeqCst),
        0,
        "liveness is checked before the provider is asked again"
    );
}
