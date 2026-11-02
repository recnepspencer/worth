//! Slot handoff evidence; physical Home completion is qualified by Store courts.
use super::*;
use crate::durability::medium::RelationalDurableMemberLocator;
use std::{
    any::Any,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
struct Payload {
    released: bool,
    drops: Arc<AtomicUsize>,
}
impl Drop for Payload {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
#[derive(Debug)]
struct Source {
    callbacks: Arc<AtomicUsize>,
}
impl RelationalDurableColdSource for Source {
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<
        RelationalDurableMemberCustody,
        crate::storage::data::RelationalStorageUnavailableCause,
    > {
        Err(crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable)
    }
    fn read_member(
        &self,
        _: RelationalDurableMemberLocator,
        _: u64,
    ) -> Result<Vec<u8>, crate::storage::data::RelationalStorageUnavailableCause> {
        Err(crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable)
    }
    fn read_commit(
        &self,
        _: crate::publication::patch::data::PatchStreamPosition,
        _: u64,
    ) -> Result<Vec<u8>, crate::storage::data::RelationalStorageUnavailableCause> {
        Err(crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable)
    }
    fn validate_member_custody_retirement(
        &self,
        original: &(dyn Any + Send + Sync),
    ) -> Result<(), RelationalDurableMemberRetirementRefusal> {
        if original.is::<Payload>() {
            Ok(())
        } else {
            Err(RelationalDurableMemberRetirementRefusal::ForeignCustody)
        }
    }
    fn advance_member_custody_retirement(
        &self,
        _: &ExecutionRequest<'_, '_>,
        original: &mut (dyn Any + Send + Sync),
    ) -> RelationalDurableMemberRetirementObservation {
        let original = original
            .downcast_mut::<Payload>()
            .expect("validated original payload");
        assert!(
            !original.released,
            "the owner callback must never replay a completed effect"
        );
        self.callbacks.fetch_add(1, Ordering::SeqCst);
        original.released = true;
        RelationalDurableMemberRetirementObservation::Complete
    }
}
fn request(cancellation: &CancellationSource) -> SerialRequest {
    SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None)
}
#[test]
fn completed_member_callback_survives_a_busy_original_return_slot_without_replay() {
    let callbacks = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let source: Arc<dyn RelationalDurableColdSource> = Arc::new(Source {
        callbacks: callbacks.clone(),
    });
    let token = RelationalDurableMemberCustody::new_from_source(
        Payload {
            released: false,
            drops: drops.clone(),
        },
        Arc::downgrade(&source),
    );
    let slot = Mutex::new(NativeSourceMemberState::new(source.clone(), token));
    let live = request(&CancellationSource::new());
    let execution = ExecutionRequest::serial(&live);
    let mut handoff = None;
    for _ in 0..2 {
        assert!(!slot.lock().unwrap().loan(&mut handoff).unwrap());
        handoff
            .as_mut()
            .unwrap()
            .observe(&execution, source.as_ref());
        // The actual slot is unavailable after the callback, not before it.
        let held = slot.lock().unwrap();
        assert!(slot.try_lock().is_err());
        handoff
            .as_mut()
            .unwrap()
            .observe(&execution, source.as_ref());
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        drop(held);
        let observation = slot
            .lock()
            .unwrap()
            .return_handoff(&mut handoff, source.as_ref())
            .unwrap();
        assert!(matches!(
            observation,
            RelationalDurableMemberRetirementObservation::Advanced
                | RelationalDurableMemberRetirementObservation::Complete
        ));
    }
    assert_eq!(callbacks.load(Ordering::SeqCst), 1);
    assert!(slot.lock().unwrap().loan(&mut handoff).unwrap());
    assert!(slot.lock().unwrap().source().is_none());
    assert_eq!(
        drops.load(Ordering::SeqCst),
        0,
        "the same retired payload remains in its original slot"
    );
    drop(slot); // Explicit unbudgeted reclaim.
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}
#[test]
fn cancelled_member_handoff_returns_original_custody_and_a_live_request_continues_it() {
    let callbacks = Arc::new(AtomicUsize::new(0));
    let source: Arc<dyn RelationalDurableColdSource> = Arc::new(Source {
        callbacks: callbacks.clone(),
    });
    let token = RelationalDurableMemberCustody::new_from_source(
        Payload {
            released: false,
            drops: Arc::new(AtomicUsize::new(0)),
        },
        Arc::downgrade(&source),
    );
    let mut slot = NativeSourceMemberState::new(source.clone(), token);
    let cancellation = CancellationSource::new();
    let stopped = request(&cancellation);
    cancellation.cancel();
    let mut handoff = None;
    assert!(!slot.loan(&mut handoff).unwrap());
    handoff
        .as_mut()
        .unwrap()
        .observe(&ExecutionRequest::serial(&stopped), source.as_ref());
    assert!(matches!(
        slot.return_handoff(&mut handoff, source.as_ref()).unwrap(),
        RelationalDurableMemberRetirementObservation::Stopped(_)
    ));
    assert_eq!(callbacks.load(Ordering::SeqCst), 0);
    assert!(Arc::ptr_eq(slot.source().unwrap(), &source));
    let live = request(&CancellationSource::new());
    loop {
        if slot.loan(&mut handoff).unwrap() {
            break;
        }
        handoff
            .as_mut()
            .unwrap()
            .observe(&ExecutionRequest::serial(&live), source.as_ref());
        match slot.return_handoff(&mut handoff, source.as_ref()).unwrap() {
            RelationalDurableMemberRetirementObservation::Advanced
            | RelationalDurableMemberRetirementObservation::Complete => {}
            other => panic!("same original task resumes: {other:?}"),
        }
    }
    assert_eq!(callbacks.load(Ordering::SeqCst), 1);
}
