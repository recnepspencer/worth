use super::super::super::QueryReadExecutionStop;
use super::*;
use crate::execution::RelationalExecutionDenialCause as Cause;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use worth_execution::ExecutionResourceLease;

struct Payload<'scope, 'authority> {
    lease: &'scope ExecutionResourceLease<'authority>,
    observed: &'scope AtomicU64,
    pointer: &'scope AtomicUsize,
    bytes: Vec<u8>,
    panic_during_disposal: bool,
}
impl Drop for Payload<'_, '_> {
    fn drop(&mut self) {
        self.observed
            .store(self.lease.charged_memory_bytes(), Ordering::SeqCst);
        self.pointer
            .store(self.bytes.as_ptr() as usize, Ordering::SeqCst);
        if self.panic_during_disposal {
            panic!("discarded fragment disposal");
        }
    }
}

#[test]
fn rejected_fragment_disposes_under_its_original_ticket_and_preserves_stop() {
    for panic_during_disposal in [false, true] {
        let lease = crate::tests::support::test_execution_lease();
        let sibling = lease.reserve_memory(23).unwrap();
        let observed = AtomicU64::new(0);
        let pointer = AtomicUsize::new(0);
        let expected_pointer;
        let result = {
            let memory = QueryPacketMemory::tracked(Some(&lease)).unwrap();
            memory.scratch(64).unwrap();
            memory.capture_result(128).unwrap();
            let slot = FragmentSlot::new(memory);
            let bytes = vec![7; 128];
            expected_pointer = bytes.as_ptr() as usize;
            *slot.value.lock().unwrap() = Some(Payload {
                lease: &lease,
                observed: &observed,
                pointer: &pointer,
                bytes,
                panic_during_disposal,
            });
            // Like schedule's stopped return, the actual value holder leaves
            // scope with the original refusal unchanged and no result handoff.
            Err::<(), _>(QueryReadExecutionStop::ReadRequest(Cause::Cancelled))
        };
        assert_eq!(observed.load(Ordering::SeqCst), 23 + 64 + 128);
        assert_eq!(pointer.load(Ordering::SeqCst), expected_pointer);
        assert!(matches!(
            result,
            Err(QueryReadExecutionStop::ReadRequest(Cause::Cancelled))
        ));
        assert_eq!(lease.charged_memory_bytes(), 23);
        drop(sibling);
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}
